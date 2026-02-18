use dsp_fftw_sys::*;

use core::ffi::CStr;
use core::ffi::c_char;

fn main() {
    unsafe {
        let p_version = fftwf_version.as_ptr() as *const c_char;
        let version = CStr::from_ptr(p_version).to_string_lossy().to_string();
        println!("version = {version}");

        let p_cc = fftwf_cc.as_ptr() as *const c_char;
        let cc = CStr::from_ptr(p_cc).to_string_lossy().to_string();
        println!("cc = {cc}");

        let p_codelet_optim = fftwf_codelet_optim.as_ptr() as *const c_char;
        let codelet_optim = CStr::from_ptr(p_codelet_optim)
            .to_string_lossy()
            .to_string();
        println!("codelet_optim = {codelet_optim}");
    }
}
