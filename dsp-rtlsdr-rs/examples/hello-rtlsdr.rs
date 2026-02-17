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

        // err = rtlsdr_reset_buffer(dev);
        // log_rtlsdr_err!(err, "rtlsdr_reset_buffer");

        // err = rtlsdr_set_sample_rate(dev, 2.048e6 /*Hz*/ as u32);
        // log_rtlsdr_err!(err, "rtlsdr_set_sample_rate");

        // err = rtlsdr_set_center_freq(dev, 99_500_000 /*Hz*/);
        // log_rtlsdr_err!(err, "rtlsdr_set_center_freq");

        // err = rtlsdr_set_freq_correction(dev, 60 /*PPM*/);
        // log_rtlsdr_err!(err, "rtlsdr_set_freq_correction");

        // err = rtlsdr_set_tuner_gain_mode(dev, 0 /*auto*/);
        // log_rtlsdr_err!(err, "rtlsdr_set_tuner_gain_mode");

        let mut buf = [0_u8; 1024];
        // let mut num_samples = 0;
        // err = rtlsdr_read_sync(dev, buf.as_mut_ptr(), buf.len() as i32, &mut num_samples);
        // log_rtlsdr_err!(err, "rtlsdr_read_sync");
        // println!("Sync: Read {} samples", num_samples);

        // for i in 0..32 {
        //     print!("    ");
        //     for j in 0..32 {
        //         print!("0x{:.02x} ", buf[32 * i + j]);
        //     }
        //     println!();
        // }

        dev.close();
    }

    Ok(())
}
