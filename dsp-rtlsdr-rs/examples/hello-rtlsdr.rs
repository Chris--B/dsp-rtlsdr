// This demonstrates the API without using unsafe
#![forbid(unsafe_code)]
#![allow(unused)]

use dsp_rtlsdr_rs::{RtlSdrDevice, all_rtlsdr_devices};

fn main() -> dsp_rtlsdr_rs::Result<()> {
    {
        let all_devices = all_rtlsdr_devices();
        for (i, dev) in all_devices.into_iter().enumerate() {
            match dev {
                Ok(dev) => {
                    println!("  + {}", dev.name());
                    println!("    + {}", dev.maufacture()?);
                    println!("    + {}", dev.product()?);
                    println!("    + {}", dev.serial()?);
                }
                Err(err) => {
                    eprintln!("Failed to open device-{i}: {}", err.desc());
                }
            }
        }

        let mut dev = RtlSdrDevice::open(0)?;

        dev.set_testmode_enabled(true)?;

        dev.set_sample_rate(900_001 /*Hz*/)?;
        dev.set_center_freq(99_500_000 /*Hz*/);
        // err = rtlsdr_set_freq_correction(dev, 60 /*PPM*/);
        // err = rtlsdr_set_tuner_gain_mode(dev, 0 /*auto*/);

        let mut buf = [0_u8; 1024];
        let num_samples = dev.read_samples(&mut buf)?;
        println!("Sync: Read {num_samples} samples",);

        for i in 0..8 {
            print!("    ");
            for j in 0..32 {
                print!("0x{:.02x} ", buf[32 * i + j]);
            }
            println!();
        }

        dev.close();
    }

    Ok(())
}
