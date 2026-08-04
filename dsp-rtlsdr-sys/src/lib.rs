#![no_std]

mod sys;
pub use sys::*;

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
    unsafe { libusb_strerror(err) }
}

// It's easier to redefine these than depend on the libusb crate
#[repr(C)]
pub struct libusb_version {
    pub major: u16,
    pub minor: u16,
    pub micro: u16,
    pub nano: u16,
    pub rc: *const c_char,
    pub describe: *const c_char,
}

// libusb_error
pub const LIBUSB_SUCCESS: c_int = 0;
pub const LIBUSB_ERROR_IO: c_int = -1;
pub const LIBUSB_ERROR_INVALID_PARAM: c_int = -2;
pub const LIBUSB_ERROR_ACCESS: c_int = -3;
pub const LIBUSB_ERROR_NO_DEVICE: c_int = -4;
pub const LIBUSB_ERROR_NOT_FOUND: c_int = -5;
pub const LIBUSB_ERROR_BUSY: c_int = -6;
pub const LIBUSB_ERROR_TIMEOUT: c_int = -7;
pub const LIBUSB_ERROR_OVERFLOW: c_int = -8;
pub const LIBUSB_ERROR_PIPE: c_int = -9;
pub const LIBUSB_ERROR_INTERRUPTED: c_int = -10;
pub const LIBUSB_ERROR_NO_MEM: c_int = -11;
pub const LIBUSB_ERROR_NOT_SUPPORTED: c_int = -12;
pub const LIBUSB_ERROR_OTHER: c_int = -99;

unsafe extern "C" {
    pub fn libusb_get_version() -> *const libusb_version;
    pub fn libusb_error_name(errcode: c_int) -> *const c_char;
    pub fn libusb_strerror(errcode: c_int) -> *const c_char;
}
