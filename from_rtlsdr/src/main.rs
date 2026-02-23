use clap::Parser;

#[allow(non_camel_case_types)]
type ci8 = num_complex::Complex<i8>;

use dsp_rtlsdr_rs::*;
use dsp_tools::cli_wrapper::o_wrapper_dsp_data_file;
use dsp_tools::io::dsp_data_file::*;

/// Attach to first found BladeRF.
#[derive(Parser, Debug)]
struct Args {
    /// Samplerate
    /// possible values are:
    ///  * 225_001 -   300_000 Hz
    ///  * 900_001 - 3_200_000 Hz
    ///  * sample loss is to be expected for rates > 2_400_000
    #[arg(short, long, default_value_t = 1_000_001)]
    samplerate: u32,

    /// Frequency
    #[arg(short, long, default_value_t = 95_700_000)]
    frequency: u32,

    /// Bias Tee Enable
    #[arg(short, long, default_value_t = false)]
    bias_tee_enable: bool,

    /// Intermediate buffer size in KiBs
    #[arg(long, default_value_t = 16)]
    buf_size: usize,
}

#[allow(unused)]
struct State {
    dev: RtlSdrDevice,
    samplerate: f64,
    frequency: f64,

    buf: Vec<ci8>,
}

fn init_state(args: Args) -> Result<State, RtlSdrError> {
    let mut dev = RtlSdrDevice::open(0)
        .expect("Failed to open first RTL-SDR device. Is it plugged in and available?");

    dev.set_sample_rate(args.samplerate)?;
    let samplerate = dev.get_sample_rate()?;

    dev.set_center_freq(args.frequency)?;
    let frequency = dev.get_center_freq()?;

    // dev.set_bias_tee_enable(args.bias_tee_enable, args.bias_tee_enable);
    unsafe {
        sys::rtlsdr_set_bias_tee(dev.raw(), args.bias_tee_enable as _);
    }

    let buf = vec![ci8::new(0, 0); args.buf_size * 1024 / 2];

    Ok(State {
        dev,
        samplerate: samplerate as f64,
        frequency: frequency as f64,
        buf,
    })
}

fn allocate_func() -> State {
    let args = Args::parse();

    // TODO: Any center_frequency OK...?

    init_state(args).expect("Failed to allocate device")
}

fn processing_func(state: &mut State) -> anyhow::Result<DspDataFile> {
    let iq: &mut [u8] = bytemuck::cast_slice_mut(&mut state.buf);
    state.dev.read_samples(iq).unwrap();

    Ok(DspDataFile::from_data_bytes(
        iq,
        DspDataType::Ci8,
        DspDataFormat::Array1d,
        Some(state.samplerate),
        None,
    ))
}

fn main() -> anyhow::Result<()> {
    o_wrapper_dsp_data_file(allocate_func, processing_func)
}
