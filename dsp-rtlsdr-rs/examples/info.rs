use dsp_rtlsdr_rs::*;

fn main() -> dsp_rtlsdr_rs::Result<()> {
    let all_devices = all_rtlsdr_devices();

    if all_devices.is_empty() {
        println!("No devices");
    }

    for (i, dev) in all_devices.into_iter().enumerate() {
        match dev {
            Ok(mut dev) => {
                println!();
                println!("[{i}] {}", dev.name());
                println!("    + Manufacturer: {}", dev.maufacturer()?);
                println!("    + Product:      {}", dev.product()?);
                println!("    + Serial:       {}", dev.serial()?);
                println!();

                let xtal = dev.get_xtal_freq()?;
                println!("    + RTL Freq:     {} Hz", xtal.rtl);
                println!("    + Tuner Freq:   {} Hz", xtal.tuner);
                println!();

                println!("    + Tuner:        {:?}", dev.get_tuner_type()?);
                println!(
                    "    + Gain:         {:>4.1}",
                    dev.get_tuner_gain()? as f32 / 10.
                );
                println!("    + Gains:");
                for gain in dev.get_tuner_gains() {
                    println!("        + {:>4.1}", gain as f32 / 10.);
                }
                println!();

                println!();
            }
            Err(err) => {
                eprintln!("Failed to open device-{i}: {}", err.desc());
            }
        }
    }

    Ok(())
}
