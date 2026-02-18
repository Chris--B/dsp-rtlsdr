use clap::Parser;
use image::{Rgb, RgbImage};
use itertools::Itertools;
use rustfft::FftPlanner;

use dsp_rtlsdr_rs::RtlSdrDevice;

#[allow(non_camel_case_types)]
type cf32 = rustfft::num_complex::Complex<f32>;

#[derive(Parser, Debug)]
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
}

fn main() -> dsp_rtlsdr_rs::Result<()> {
    let opts = Opts::parse();

    let mut dev = match RtlSdrDevice::open(0) {
        Ok(dev) => dev,
        Err(err) => {
            eprintln!();
            eprintln!("Failed to open device. Are you sure it's plugged in and not in use?");
            return Err(err);
        }
    };

    // Configure device
    {
        dev.set_sample_rate(opts.sample_rate)?;
        dev.set_center_freq(opts.center_freq)?;

        if opts.test {
            dev.set_testmode_enabled(true)?;
            println!("Test mode is enabled");
        }
    }

    let w = 1024;
    let h = 1024;
    let mut samples8: Vec<u8> = vec![0; 2 * w * h];

    // Discard first few samples
    let _ = dev.read_samples(&mut samples8[..2048])?;
    let _n_read = dev.read_samples(&mut samples8)?;

    let mut samples: Vec<cf32> = vec![];
    for (i, q) in samples8.drain(..).tuples() {
        samples.push(cf32::new(i as f32, q as f32));
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(w);
    for chunk in samples.chunks_exact_mut(w) {
        fft.process(chunk);
        let (pos, neg) = chunk.split_at_mut(chunk.len() / 2);
        // TODO: running mean to subtrack off DC spike
        pos[0] = cf32::new(1., 1.);
        pos.swap_with_slice(neg);
    }

    let mut img = RgbImage::new(w as u32, h as u32);
    let (min, max) = samples
        .iter()
        .map(|iq| (iq.norm() + 1e-10).log10())
        .minmax_by_key(|f| f.to_bits() as i32)
        .into_option()
        .unwrap();

    for (y, chunk) in samples.chunks_exact(w).enumerate() {
        for (x, iq) in chunk.iter().enumerate() {
            let r = (iq.norm().log10() - min) / (max - min);
            let r = r * r;
            let r = (255.0 * r) as u8;
            img.put_pixel(x as u32, y as u32, Rgb([r, r, r]));
        }
    }
    img.save(opts.output).unwrap();

    Ok(())
}
