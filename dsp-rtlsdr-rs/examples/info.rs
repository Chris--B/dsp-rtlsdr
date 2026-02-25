use dsp_rtlsdr_rs::*;

fn main() -> dsp_rtlsdr_rs::Result<()> {
    let all_devices = all_rtlsdr_devices();
    for (i, dev) in all_devices.into_iter().enumerate() {
        match dev {
            Ok(mut dev) => {
                println!();
                println!("[{i}] {}", dev.name());
                println!("    + Manufacturer: {}", dev.maufacturer()?);
                println!("    + Product:      {}", dev.product()?);
                println!("    + Serial:       {}", dev.serial()?);
                let xtal = dev.get_xtal_freq()?;
                println!("    + RTL Freq:     {} Hz", xtal.rtl);
                println!("    + Tuner Freq:   {} Hz", xtal.tuner);
                println!();
            }
            Err(err) => {
                eprintln!("Failed to open device-{i}: {}", err.desc());
            }
        }
    }

    Ok(())
}
