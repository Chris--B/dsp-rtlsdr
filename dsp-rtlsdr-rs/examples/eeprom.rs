use dsp_rtlsdr_rs::*;

fn main() -> dsp_rtlsdr_rs::Result<()> {
    let mut dev = RtlSdrDevice::open(0)?;

    let mut eeprom = vec![0; RtlSdrDevice::EEPROM_SIZE];
    dev.read_eeprom(&mut eeprom, 0)?;

    // TODO: parse out into config: https://github.com/osmocom/rtl-sdr/blob/master/src/rtl_eeprom.c#L132-L150

    // Optionally write out ROM to a file
    if let Some(filename) = std::env::args().nth(1) {
        std::fs::write(&filename, eeprom).map_err(|_| panic!("Failed to write to {filename}"));
    }

    Ok(())
}
