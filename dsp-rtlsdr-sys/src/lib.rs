#![no_std]

mod sys;
pub use sys::*;

pub use libusb_sys;

use ::core::ffi::c_char;
use ::core::ffi::c_void;
use ::core::ffi::{c_int, c_uint};

#[allow(nonstandard_style)]
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct rtlsdr_dev_t(*mut rtlsdr_dev);

impl rtlsdr_dev_t {
    pub fn null() -> Self {
        Self(core::ptr::null_mut())
    }

    /// Gets the inner pointer-handle that the rtlsdr C API expects
    pub fn raw(self) -> *mut rtlsdr_dev {
        self.0
    }
}

pub fn strerror(err: c_int) -> *const c_char {
    unsafe { libusb_sys::libusb_strerror(err) }
}
