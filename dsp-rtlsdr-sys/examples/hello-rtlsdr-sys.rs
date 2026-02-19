use dsp_rtlsdr_sys::*;

use std::ffi::CStr;
use std::ffi::c_char;

macro_rules! log_rtlsdr_err {
    ($err:expr, $($args:tt)*) => {
        let args =  format_args!($($args)*);
        let err = $err;
        if err != 0 {
            let err_text = CStr::from_ptr(dsp_rtlsdr_sys::strerror(err));
            println!("{args}: {err_text:?} ({err})");
        }
    };
}

fn main() {
    unsafe {
        let num_devices = rtlsdr_get_device_count();
        println!("Found {num_devices} device(s)");
        if num_devices == 0 {
            println!("Is your device connected?");
            return;
        }

        for i in 0..num_devices {
            let p_name = rtlsdr_get_device_name(i);
            println!("  + {:?}", CStr::from_ptr(p_name));

            let mut manufact = [0 as c_char; 256];
            let mut product = [0 as c_char; 256];
            let mut serial = [0 as c_char; 256];
            let err = rtlsdr_get_device_usb_strings(
                i,
                manufact.as_mut_ptr(),
                product.as_mut_ptr(),
                serial.as_mut_ptr(),
            );
            log_rtlsdr_err!(err, "rtlsdr_get_device_usb_strings");

            let manufact = CStr::from_ptr(manufact.as_ptr());
            if !manufact.is_empty() {
                println!("  + Manufact: {:?}", manufact);
            }

            let product = CStr::from_ptr(product.as_ptr());
            if !product.is_empty() {
                println!("  + Product: {:?}", product);
            }

            let serial = CStr::from_ptr(serial.as_ptr());
            if !serial.is_empty() {
                println!("  + Serial: {:?}", serial);
            }
        }

        let mut err;
        let mut dev = rtlsdr_dev_t::null();

        err = rtlsdr_open(&mut dev, 0);
        log_rtlsdr_err!(err, "rtlsdr_open");
        if err < 0 {
            return;
        }

        err = rtlsdr_set_testmode(dev, 1);
        log_rtlsdr_err!(err, "rtlsdr_set_testmode");

        err = rtlsdr_reset_buffer(dev);
        log_rtlsdr_err!(err, "rtlsdr_reset_buffer");

        err = rtlsdr_set_sample_rate(dev, 900_001 /*Hz*/);
        log_rtlsdr_err!(err, "rtlsdr_set_sample_rate");

        err = rtlsdr_set_center_freq(dev, 99_500_000 /*Hz*/);
        log_rtlsdr_err!(err, "rtlsdr_set_center_freq");

        err = rtlsdr_set_freq_correction(dev, 60 /*PPM*/);
        log_rtlsdr_err!(err, "rtlsdr_set_freq_correction");

        err = rtlsdr_set_tuner_gain_mode(dev, 0 /*auto*/);
        log_rtlsdr_err!(err, "rtlsdr_set_tuner_gain_mode");

        let mut buf = [0_u8; 1024];
        let mut num_samples = 0;
        err = rtlsdr_read_sync(dev, buf.as_mut_ptr(), buf.len() as i32, &mut num_samples);
        log_rtlsdr_err!(err, "rtlsdr_read_sync");
        println!("Sync: Read {} samples", num_samples);
        if err < 0 {
            return;
        }

        for (i, sample) in buf.iter().enumerate().take(304) {
            if i > 0 && i % 16 == 0 {
                println!();
            }
            print!("{sample:>3} ");
        }
        println!();

        rtlsdr_close(dev);
    }
}
