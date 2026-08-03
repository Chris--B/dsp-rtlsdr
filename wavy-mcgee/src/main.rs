#![allow(non_snake_case, clippy::unused_enumerate_index)]

use clap::Parser;
use itertools::Itertools;
use just_sdl3::*;
use rustfft::FftPlanner;

use core::ffi::*;

use dsp_rtlsdr_rs::RtlSdrDevice;

#[allow(non_camel_case_types)]
type cf32 = rustfft::num_complex::Complex<f32>;

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

pub struct App {
    #[allow(unused)]
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,

    sdr: RtlSdrDevice,
    opts: Opts,

    planner: FftPlanner<f32>,
    // Current row that we'll update
    // TODO: Background thread
    row: usize,
}

impl App {
    /// Borrow back out of a callback's `appstate` pointer
    fn get(appstate: &mut *mut c_void) -> &mut Self {
        unsafe { &mut *(*appstate as *mut Self) }
    }

    unsafe fn read(appstate: &mut *mut c_void) -> Self {
        unsafe { (*appstate as *mut Self).read() }
    }

    /// Allocate and store a new object into an `*appstate`
    fn new_into(app: App, appstate: *mut *mut c_void) {
        unsafe {
            let example = Box::new(app);
            *appstate = Box::into_raw(example) as *mut c_void
        };
    }
}

/* This function runs once at startup. */
#[unsafe(no_mangle)]
unsafe extern "C" fn SDL_AppInit(
    appstate: *mut *mut c_void,
    _argc: c_int,
    _argv: *mut *mut c_char,
) -> SDL_AppResult {
    unsafe {
        let opts = Opts::parse();

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
            640,
            480,
            0,
            &mut window,
            &mut renderer,
        ) {
            SDL_Log(
                c"Couldn't create window/renderer: %s".as_ptr(),
                SDL_GetError(),
            );
            return SDL_APP_FAILURE;
        }

        SDL_SetWindowResizable(window, true);

        let texture = SDL_CreateTexture(
            renderer,
            SDL_PIXELFORMAT_RGB24,
            SDL_TEXTUREACCESS_STREAMING,
            opts.width as i32,
            opts.height as i32,
        );
        if texture.is_null() {
            SDL_Log(
                c"Failed to create texture with format=%d\n%s".as_ptr(),
                SDL_PIXELFORMAT_RGB24 as c_int,
                SDL_GetError(),
            );
        }

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

        let planner = FftPlanner::new();

        let app = App {
            window,
            renderer,
            texture,

            sdr,
            opts,

            planner,
            row: 0,
        };
        App::new_into(app, appstate);

        SDL_APP_CONTINUE
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn SDL_AppEvent(
    mut appstate: *mut c_void,
    event: *mut SDL_Event,
) -> SDL_AppResult {
    unsafe {
        let appstate = App::get(&mut appstate);
        let _ = appstate;

        if (*event).r#type == SDL_EVENT_QUIT {
            return SDL_APP_SUCCESS;
        }

        SDL_APP_CONTINUE
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn SDL_AppIterate(mut appstate: *mut c_void) -> SDL_AppResult {
    unsafe {
        let appstate = App::get(&mut appstate);

        let _now_ms: f64 = (SDL_GetTicks() as f64) / 1000.0;

        // Do the FFT I guess
        let mut wavy = vec![];
        let rows_per_read = 1;
        {
            let mut samples8: Vec<u8> = vec![0; 2 * appstate.opts.fft_window * rows_per_read];
            let n = appstate.sdr.read_samples(&mut samples8).unwrap_or(0) as usize;
            let mut samples: Vec<cf32> = vec![];
            for (i, q) in samples8.drain(..n).tuples() {
                samples.push(cf32::new(i as f32, q as f32));
            }

            let fft = appstate.planner.plan_fft_forward(appstate.opts.fft_window);
            for chunk in samples.chunks_exact_mut(appstate.opts.fft_window) {
                fft.process(chunk);
                let (pos, neg) = chunk.split_at_mut(chunk.len() / 2);
                // TODO: running mean to subtrack off DC spike
                pos[0] = cf32::new(1., 1.);
                pos.swap_with_slice(neg);
            }

            let (min, max) = samples
                .iter()
                .map(|iq| (iq.norm() + 1e-10).log10())
                .minmax_by_key(|f| f.to_bits() as i32)
                .into_option()
                .unwrap();

            for (_y, chunk) in samples.chunks_exact(appstate.opts.fft_window).enumerate() {
                for (_x, iq) in chunk.iter().enumerate() {
                    let r = (iq.norm().log10() - min) / (max - min);
                    let r = r * r;
                    let r = (255.0 * r) as u8;

                    // TODO: Make it colorful
                    let g = r;
                    let b = r;
                    wavy.push(r);
                    wavy.push(g);
                    wavy.push(b);
                }
            }
        }

        // Update our SDL window
        {
            let mut ok;
            SDL_ClearError();

            {
                let mut pitch = 0_i32;
                let mut p_pixels: *mut c_void = SDL_NULL();
                ok = SDL_LockTexture(appstate.texture, SDL_NULL(), &mut p_pixels, &mut pitch);
                if !ok {
                    SDL_Log(c"SDL_LockTexture() failed: %s\n".as_ptr(), SDL_GetError());
                    return SDL_APP_FAILURE;
                }
                let pixels = core::slice::from_raw_parts_mut(
                    p_pixels as *mut u8,
                    (pitch as usize) * (appstate.opts.height as usize),
                );

                let begin = appstate.row * (pitch as usize);
                let end = begin + wavy.len();
                if end < pixels.len() {
                    pixels[begin..end].copy_from_slice(&wavy);
                    appstate.row += rows_per_read;
                    appstate.row %= appstate.opts.height as usize;
                } else {
                    let n = pixels.len() - begin;
                    let m = wavy.len() - n;
                    pixels[begin..].copy_from_slice(&wavy[..n]);
                    pixels[..m].copy_from_slice(&wavy[n..]);
                    appstate.row = m / (pitch as usize);
                }

                SDL_UnlockTexture(appstate.texture);
            }

            ok = SDL_SetRenderDrawColorFloat(
                appstate.renderer,
                0.0,
                0.0,
                0.0,
                SDL_ALPHA_OPAQUE_FLOAT,
            );
            if !ok {
                SDL_Log(
                    c"SDL_SetRenderDrawColorFloat() failed: %s\n".as_ptr(),
                    SDL_GetError(),
                );
                return SDL_APP_FAILURE;
            }

            ok = SDL_RenderClear(appstate.renderer);
            if !ok {
                SDL_Log(c"SDL_RenderClear() failed: %s\n".as_ptr(), SDL_GetError());
                return SDL_APP_FAILURE;
            }

            ok = SDL_RenderTexture(appstate.renderer, appstate.texture, SDL_NULL(), SDL_NULL());
            if !ok {
                SDL_Log(c"SDL_RenderTexture() failed: %s\n".as_ptr(), SDL_GetError());
                return SDL_APP_FAILURE;
            }

            ok = SDL_RenderPresent(appstate.renderer);
            if !ok {
                SDL_Log(c"SDL_RenderPresent() failed: %s\n".as_ptr(), SDL_GetError());
                return SDL_APP_FAILURE;
            }
        }

        SDL_APP_CONTINUE
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn SDL_AppQuit(mut appstate: *mut c_void, _result: SDL_AppResult) {
    unsafe {
        let appstate = App::read(&mut appstate);
        drop(appstate);
    }
}
