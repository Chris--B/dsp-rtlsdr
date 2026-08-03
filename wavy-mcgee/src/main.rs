#![allow(non_snake_case, clippy::unused_enumerate_index)]

use clap::Parser;
use dsp_rtlsdr_rs::RtlSdrDevice;
use just_sdl3::*;

use core::ffi::*;

mod sampler_thread;
use sampler_thread::*;

#[allow(non_camel_case_types)]
pub type cf32 = rustfft::num_complex::Complex<f32>;

const fn SDL_NULL<T>() -> *mut T {
    core::ptr::null_mut()
}

#[derive(Parser, Debug)]
#[command(disable_help_flag = true)]
pub struct Opts {
    /// Sample rate in Hz
    #[arg(short, long, default_value_t = 900_001)]
    pub sample_rate: u32,

    /// Center frequency in Hz
    #[arg(short = 'f', long, default_value_t = 99_500_000)]
    pub center_freq: u32,

    #[arg(short, long, default_value_t = false)]
    pub test: bool,

    #[arg(short, long, default_value = "wave.png")]
    pub output: String,

    /// Number of samples used in each row of the phase raster
    #[arg(long, default_value = "1024")]
    pub fft_window: usize,

    /// Width in pixels of viewer window (ignored with --fullscreen)
    #[arg(short, long, default_value = "1024")]
    pub width: u32,

    /// Height in pixels of viewer window (ignored with --fullscreen)
    #[arg(short, long, default_value = "1024")]
    pub height: u32,

    /// Opens the window in fullscreen, ignoring --width or --height
    #[arg(long, default_value = "false")]
    pub fullscreen: bool,

    // We use -h for height, which conflicts with helper.
    // This lets us preserve --help while also taking -h for ourselves.
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,
}

// See: https://wiki.libsdl.org/SDL3/NonstandardStartup
fn main() {
    unsafe {
        SDL_EnterAppMainCallbacks(
            c_args::argc(),
            c_args::argv(),
            Some(SDL_AppInit),
            Some(SDL_AppIterate),
            Some(SDL_AppEvent),
            Some(SDL_AppQuit),
        );
    }
}

fn SDL_LogError(label: &CStr) {
    unsafe {
        SDL_Log(c"%s: %s".as_ptr(), label.as_ptr(), SDL_GetError());
    }
}

pub struct App {
    #[allow(unused)]
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,

    sampler_thread: SamplerThread,
    opts: Opts,
    paused: bool,
}

impl App {
    /// Borrow back out of a callback's `appstate` pointer
    fn get(appstate: &mut *mut c_void) -> Option<&mut Self> {
        if !appstate.is_null() {
            Some(unsafe { &mut *(*appstate as *mut Self) })
        } else {
            None
        }
    }

    /// Borrow back out of a callback's `appstate` pointer
    unsafe fn read(appstate: &mut *mut c_void) -> Option<Self> {
        if !appstate.is_null() {
            unsafe {
                let t = Some((*appstate as *mut Self).read());
                *appstate = SDL_NULL();
                t
            }
        } else {
            None
        }
    }

    /// Allocate and store a new object into an `*appstate`
    fn new_into(app: App, appstate: *mut *mut c_void) {
        unsafe {
            let example = Box::new(app);
            *appstate = Box::into_raw(example) as *mut c_void
        };
    }
}

// TODO: Wrap in panic-catching code
#[unsafe(no_mangle)]
unsafe extern "C" fn SDL_AppInit(
    appstate: *mut *mut c_void,
    _argc: c_int,
    _argv: *mut *mut c_char,
) -> SDL_AppResult {
    unsafe {
        let opts = Opts::parse();

        // Init SDR
        let mut sdr = match RtlSdrDevice::open(0) {
            Ok(sdr) => sdr,
            Err(err) => {
                eprintln!();
                eprintln!("Failed to open device. Are you sure it's plugged in and not in use?");
                eprintln!("{err:#?}");
                return SDL_APP_FAILURE;
            }
        };

        // Configure device
        {
            if let Err(err) = sdr.set_sample_rate(opts.sample_rate) {
                eprint!("set_sample_rate(): {err:#?}")
            };
            if let Err(err) = sdr.set_center_freq(opts.center_freq) {
                eprint!("set_center_freq(): {err:#?}")
            };

            if opts.test {
                if let Err(err) = sdr.set_testmode_enabled(true) {
                    eprint!("set_testmode_enabled(): {err:#?}")
                };
                println!("Test mode is enabled");
            }
        }

        // Start background thread
        let sampler_thread = spawn_sampler_thread(sdr, &opts);

        let mut window = SDL_NULL();
        let mut renderer = SDL_NULL();

        SDL_SetAppMetadata(
            c"Wavy McGee".as_ptr(),
            c"1.0".as_ptr(),
            c"com.example.wavy-mcgee".as_ptr(),
        );

        if !SDL_Init(SDL_INIT_VIDEO) {
            SDL_Log(c"Couldn't initialize SDL: %s".as_ptr(), SDL_GetError());
            return SDL_APP_FAILURE;
        }

        if !SDL_CreateWindowAndRenderer(
            c"Wavy McGee".as_ptr(),
            opts.width as i32,
            opts.height as i32,
            0,
            &mut window,
            &mut renderer,
        ) {
            SDL_LogError(c"SDL_CreateWindowAndRenderer()");
            return SDL_APP_FAILURE;
        }

        SDL_SetWindowResizable(window, true);
        SDL_SetWindowFullscreen(window, opts.fullscreen);

        let texture = SDL_CreateTexture(
            renderer,
            SDL_PIXELFORMAT_RGB24,
            SDL_TEXTUREACCESS_STREAMING,
            opts.width as i32,
            opts.height as i32,
        );
        if texture.is_null() {
            SDL_LogError(c"Failed to create texture with format=SDL_PIXELFORMAT_RGB24\n%s");
        }

        let app = App {
            window,
            renderer,
            texture,

            sampler_thread,
            opts,
            paused: false,
        };
        App::new_into(app, appstate);

        SDL_APP_CONTINUE
    }
}

// TODO: Wrap in panic-catching code
#[unsafe(no_mangle)]
unsafe extern "C" fn SDL_AppEvent(
    mut appstate: *mut c_void,
    event: *mut SDL_Event,
) -> SDL_AppResult {
    unsafe {
        let Some(appstate) = App::get(&mut appstate) else {
            return SDL_APP_FAILURE;
        };

        #[allow(clippy::single_match)]
        match (*event).r#type {
            SDL_EVENT_QUIT => return SDL_APP_SUCCESS,

            SDL_EVENT_KEY_UP => {
                // sorry
                #[repr(C)]
                struct SDL_KeyboardEvent {
                    /// SDL_EVENT_KEY_DOWN or SDL_EVENT_KEY_UP
                    r#type: SDL_EventType,
                    reserved: Uint32,
                    /// In nanoseconds, populated using [`SDL_GetTicksNS()`]
                    timestamp: Uint64,
                    /// The window with keyboard focus, if any
                    windowID: SDL_WindowID,
                    /// The keyboard instance id, or 0 if unknown or virtual
                    which: u32,
                    /// SDL physical key code
                    scancode: u32,
                    /// SDL virtual key code
                    key: u32,
                    /// current key modifiers
                    r#mod: u32,
                    /// The platform dependent scancode for this event
                    raw: Uint16,
                    /// true if the key is pressed
                    down: bool,
                    /// true if this is a key repeat
                    repeat: bool,
                }
                let event = core::ptr::read(event as *const SDL_KeyboardEvent);

                match event.key {
                    0x20 /*SDLK_SPACE*/ => {
                        appstate.paused = !appstate.paused;
                    }
                    0x71 /*SDLK_Q*/ => return SDL_APP_SUCCESS,
                    0x74 /*SDLK_T*/ => {
                        appstate.opts.test = !appstate.opts.test;
                        let _ = appstate.sampler_thread.ask_tx.send(SetTestMode(appstate.opts.test));
                    }
                    0x63 /*SDLK_C*/ => {
                        // I guess just recreate it to clear it?
                        let mut w = 0.;
                        let mut h = 0.;
                        SDL_GetTextureSize(appstate.texture, &mut w, &mut h);
                        let new_texture = SDL_CreateTexture(
                            appstate.renderer,
                            SDL_PIXELFORMAT_RGB24,
                            SDL_TEXTUREACCESS_STREAMING,
                            w as i32,
                            h as i32,
                        );
                        if new_texture.is_null() {
                            SDL_Log(
                                c"[NEW] SDL_CreateTexture() failed: %s\n".as_ptr(),
                                SDL_GetError(),
                            );
                            return SDL_APP_CONTINUE;
                        }

                        SDL_DestroyTexture(appstate.texture);
                        appstate.texture = new_texture;

                        // Make sure we start drawing at the top tho
                        let _ = appstate.sampler_thread.ask_tx.send(SetRow(0));
                    }
                    _ => {}
                }
            }

            SDL_EVENT_WINDOW_RESIZED => {
                SDL_ClearError();

                let _new_width = (*event).window.data1;
                let new_height = (*event).window.data2;

                let mut old_w = 0.;
                let mut old_h = 0.;
                SDL_GetTextureSize(appstate.texture, &mut old_w, &mut old_h);
                if (old_h as i32) != new_height {
                    let new_texture = SDL_CreateTexture(
                        appstate.renderer,
                        SDL_PIXELFORMAT_RGB24,
                        SDL_TEXTUREACCESS_STREAMING,
                        old_w as i32, // Don't scale width
                        new_height,
                    );
                    if new_texture.is_null() {
                        SDL_Log(
                            c"[NEW] SDL_CreateTexture() failed: %s\n".as_ptr(),
                            SDL_GetError(),
                        );
                        return SDL_APP_CONTINUE;
                    }

                    SDL_DestroyTexture(appstate.texture);
                    appstate.texture = new_texture;
                }

                let _ = appstate
                    .sampler_thread
                    .ask_tx
                    .send(SetHeight(new_height as usize));
            }
            _ => {}
        }

        SDL_APP_CONTINUE
    }
}

// TODO: Wrap in panic-catching code
#[unsafe(no_mangle)]
unsafe extern "C" fn SDL_AppIterate(mut appstate: *mut c_void) -> SDL_AppResult {
    unsafe {
        let Some(appstate) = App::get(&mut appstate) else {
            return SDL_APP_FAILURE;
        };

        // Update our SDL window
        {
            SDL_ClearError();

            {
                let mut pitch = 0_i32;
                let mut p_pixels: *mut c_void = SDL_NULL();

                // TODO: Lock only sub-regions that are getting updated
                if !SDL_LockTexture(appstate.texture, SDL_NULL(), &mut p_pixels, &mut pitch) {
                    SDL_Log(c"SDL_LockTexture() failed: %s\n".as_ptr(), SDL_GetError());
                    return SDL_APP_FAILURE;
                }

                let pitch = pitch as usize;
                let mut width = 0.0;
                let mut height = 0.0;
                if !SDL_GetTextureSize(appstate.texture, &mut width, &mut height) {
                    SDL_Log(
                        c"SDL_GetTextureSize() failed, somehow? %s\n".as_ptr(),
                        SDL_GetError(),
                    );
                }
                let pixels =
                    core::slice::from_raw_parts_mut(p_pixels as *mut u8, pitch * height as usize);

                while let Ok(update) = appstate.sampler_thread.update_rx.try_recv() {
                    if appstate.paused {
                        // While paused, discard samples
                        continue;
                    }
                    // Resizing events can result in the sampler thread "rendering" rows that are now out of bounds
                    // No-Op out of bounds pixels
                    if update.row < height as usize {
                        let begin = update.row * pitch;
                        let end = begin + pitch;
                        pixels[begin..end].copy_from_slice(&update.pixels);
                    }
                }

                SDL_UnlockTexture(appstate.texture);
            }

            if !SDL_SetRenderDrawColor(appstate.renderer, 0, 0, 0, SDL_ALPHA_OPAQUE as u8) {
                SDL_Log(
                    c"SDL_SetRenderDrawColorFloat() failed: %s\n".as_ptr(),
                    SDL_GetError(),
                );
                return SDL_APP_FAILURE;
            }

            if !SDL_RenderClear(appstate.renderer) {
                SDL_Log(c"SDL_RenderClear() failed: %s\n".as_ptr(), SDL_GetError());
                return SDL_APP_FAILURE;
            }

            if !SDL_RenderTexture(appstate.renderer, appstate.texture, SDL_NULL(), SDL_NULL()) {
                SDL_Log(c"SDL_RenderTexture() failed: %s\n".as_ptr(), SDL_GetError());
                return SDL_APP_FAILURE;
            }

            if !SDL_RenderPresent(appstate.renderer) {
                SDL_Log(c"SDL_RenderPresent() failed: %s\n".as_ptr(), SDL_GetError());
                return SDL_APP_FAILURE;
            }
        }

        SDL_APP_CONTINUE
    }
}

// TODO: Wrap in panic-catching code
#[unsafe(no_mangle)]
unsafe extern "C" fn SDL_AppQuit(mut appstate: *mut c_void, _result: SDL_AppResult) {
    unsafe {
        let Some(appstate) = App::read(&mut appstate) else {
            return;
        };

        // Tell the background thread to exit and wait a little for it to do so.
        let _ = appstate.sampler_thread.ask_tx.send(Exit);
        #[allow(deprecated)]
        std::thread::sleep_ms(250);

        drop(appstate);
    }
}
