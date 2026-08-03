use dsp_rtlsdr_rs::RtlSdrDevice;
use itertools::Itertools;
use rustfft::FftPlanner;

use std::sync::mpsc;

use crate::Opts;
use crate::cf32;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum SamplerAsks {
    SetHeight(usize),
    SetFfftWindow(usize),
    SetRow(usize),
    SetTestMode(bool),
    Exit,
}
pub use SamplerAsks::*;

pub struct SamplerThread {
    pub _handle: std::thread::JoinHandle<()>,
    pub ask_tx: mpsc::Sender<SamplerAsks>,
    pub update_rx: mpsc::Receiver<PixelsUpdate>,
}

pub struct PixelsUpdate {
    pub row: usize,
    pub pixels: Vec<u8>,
}

pub fn spawn_sampler_thread(mut sdr: RtlSdrDevice, opts: &Opts) -> SamplerThread {
    let (update_tx, update_rx) = mpsc::channel::<PixelsUpdate>();
    let (ask_tx, ask_rx) = mpsc::channel::<SamplerAsks>();
    let mut fft_window = opts.fft_window;
    let mut height = opts.height as usize;

    let handle = std::thread::spawn(move || {
        let mut planner = FftPlanner::new();
        let mut row = 0;

        'main: loop {
            while let Ok(ask) = ask_rx.try_recv() {
                match ask {
                    SetHeight(h) => {
                        if height != h {
                            height = h;
                            row = 0;
                        }
                    }
                    SetFfftWindow(w) => fft_window = w,
                    SetRow(r) => row = r,
                    SetTestMode(enabled) => {
                        let _ = sdr.set_testmode_enabled(enabled);
                    }
                    Exit => break 'main,
                }
            }

            // Get Samples
            let mut samples8: Vec<u8> = vec![0; 2 * fft_window];
            let n = sdr.read_samples(&mut samples8).unwrap_or(0) as usize;

            if n == 0 {
                continue 'main;
            }

            // Get Samples as cf32
            let mut samples: Vec<cf32> = vec![];
            for (i, q) in samples8.drain(..n).tuples() {
                samples.push(cf32::new(i as f32, q as f32));
            }

            // Do FFT
            let fft = planner.plan_fft_forward(fft_window);
            {
                fft.process(&mut samples);

                let mid = samples.len() / 2;
                let (pos, neg) = samples.split_at_mut(mid);
                // TODO: running mean to subtrack off DC spike
                pos[0] = cf32::new(1., 1.);
                pos.swap_with_slice(neg);
            }

            // Map FFT to Pixels
            let mut pixels = vec![];
            let (min, max) = samples
                .iter()
                .map(|iq| (iq.norm() + 1e-10).log10())
                .minmax_by_key(|f| f.to_bits() as i32)
                .into_option()
                .unwrap();
            for iq in &samples {
                let r = (iq.norm().log10() - min) / (max - min);
                let r = r * r;
                let r = (255.0 * r) as u8;

                // TODO: Make it colorful
                let g = r;
                let b = r;
                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
            }

            // Send them off
            match update_tx.send(PixelsUpdate { row, pixels }) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("[Sampler Thread] Failed to send update for row {row}: {err:#?}")
                }
            };

            // Next row
            row += 1;
            row %= height;
        }
    });

    SamplerThread {
        _handle: handle,
        ask_tx,
        update_rx,
    }
}
