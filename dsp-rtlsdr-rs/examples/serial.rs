use dsp_rtlsdr_rs::*;

fn main() -> dsp_rtlsdr_rs::Result<()> {
    let serial = std::env::args().nth(1).unwrap_or("00000001".to_string());

    println!("Trying to open serial {serial}");
    let index = RtlSdrDevice::get_index_by_serial(serial)?;
    let name = RtlSdrDevice::name_of(index);
    println!("Found device: \"{name}\"");

    Ok(())
}
