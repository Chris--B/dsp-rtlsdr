pub use dsp_rtlsdr_sys as sys;

use sys::libusb_sys::*;
use sys::*;

use std::ffi::CStr;
use std::ffi::{c_char, c_int};
use std::mem::ManuallyDrop;

pub type Result<T, E = crate::RtlSdrError> = std::result::Result<T, E>;

/// An error that can be raised from `librtlsdr` functions
///
/// Note: Because `librtlsdr` uses `libusb-1.0`'s error codes exactly, these all match directly to `libusb-1.0`'s error codes.
///
/// Consult `libusb` docs for more details. <https://libusb.sourceforge.io/api-1.0/group__libusb__misc.html#gab2323aa0f04bc22038e7e1740b2f29ef>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RtlSdrError {
    /// A string describing what failed.
    ///
    /// This is typically the `librtlsdr` C function name that generated this error.
    pub what: &'static str,

    /// The `libusb-1.0` error code returned by some `librtlsdr` function call.
    pub code: ErrorCode,
}

impl RtlSdrError {
    /// A short description of the given error code.
    ///
    /// This description is intended for displaying to the end user and will be in the language set by [`libusb_setlocale()`].
    /// See also [`libusb_error_name()`].
    pub fn desc(self) -> String {
        format!("{}: {}", self.what, self.code.desc())
    }
}

/// An error that can be raised from librtlsdr functions
///
/// Note: Because `librtlsdr` uses `libusb-1.0`'s error codes exactly, these all match directly to `libusb-1.0`'s error codes.
///
/// Consult `libusb` docs for more details. <https://libusb.sourceforge.io/api-1.0/group__libusb__misc.html#gab2323aa0f04bc22038e7e1740b2f29ef>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    // Note: No LIBUSB_SUCCESS, since that's not an error
    /// See: [`LIBUSB_ERROR_IO`]
    Io = LIBUSB_ERROR_IO,

    /// See: [`LIBUSB_ERROR_INVALID_PARAM`]
    InvalidParam = LIBUSB_ERROR_INVALID_PARAM,

    /// See: [`LIBUSB_ERROR_ACCESS`]
    Access = LIBUSB_ERROR_ACCESS,

    /// See: [`LIBUSB_ERROR_NO_DEVICE`]
    NoDevice = LIBUSB_ERROR_NO_DEVICE,

    /// See: [`LIBUSB_ERROR_NOT_FOUND`]
    NotFound = LIBUSB_ERROR_NOT_FOUND,

    /// See: [`LIBUSB_ERROR_BUSY`]
    Busy = LIBUSB_ERROR_BUSY,

    /// See: [`LIBUSB_ERROR_TIMEOUT`]
    Timeout = LIBUSB_ERROR_TIMEOUT,

    /// See: [`LIBUSB_ERROR_OVERFLOW`]
    Overflow = LIBUSB_ERROR_OVERFLOW,

    /// See: [`LIBUSB_ERROR_PIPE`]
    Pipe = LIBUSB_ERROR_PIPE,

    /// See: [`LIBUSB_ERROR_INTERRUPTED`]
    Interrupted = LIBUSB_ERROR_INTERRUPTED,

    /// See: [`LIBUSB_ERROR_NO_MEM`]
    NoMem = LIBUSB_ERROR_NO_MEM,

    /// See: [`LIBUSB_ERROR_NOT_SUPPORTED`]
    NotSupported = LIBUSB_ERROR_NOT_SUPPORTED,

    /// See: [`LIBUSB_ERROR_OTHER`]
    Other = LIBUSB_ERROR_OTHER,
}

impl ErrorCode {
    /// The ASCII name of an error.
    ///
    /// See also [`libusb_error_name()`].
    pub fn name(self) -> String {
        unsafe {
            let p_str = libusb_error_name(self.to_raw());
            CStr::from_ptr(p_str).to_string_lossy().to_string()
        }
    }

    /// A short description of the given error code.
    ///
    /// This description is intended for displaying to the end user and will be in the language set by [`libusb_setlocale()`].
    /// See also [`libusb_strerror()`] and [`libusb_error_name()`].
    pub fn desc(self) -> String {
        unsafe {
            let p_str = libusb_strerror(self.to_raw());
            CStr::from_ptr(p_str).to_string_lossy().to_string()
        }
    }

    pub fn to_raw(self) -> c_int {
        match self {
            Self::Io => LIBUSB_ERROR_IO,
            Self::InvalidParam => LIBUSB_ERROR_INVALID_PARAM,
            Self::Access => LIBUSB_ERROR_ACCESS,
            Self::NoDevice => LIBUSB_ERROR_NO_DEVICE,
            Self::NotFound => LIBUSB_ERROR_NOT_FOUND,
            Self::Busy => LIBUSB_ERROR_BUSY,
            Self::Timeout => LIBUSB_ERROR_TIMEOUT,
            Self::Overflow => LIBUSB_ERROR_OVERFLOW,
            Self::Pipe => LIBUSB_ERROR_PIPE,
            Self::Interrupted => LIBUSB_ERROR_INTERRUPTED,
            Self::NoMem => LIBUSB_ERROR_NO_MEM,
            Self::NotSupported => LIBUSB_ERROR_NOT_SUPPORTED,
            Self::Other => LIBUSB_ERROR_OTHER,
        }
    }

    pub fn from_raw(raw: c_int) -> Self {
        match raw {
            LIBUSB_SUCCESS => todo!(),
            LIBUSB_ERROR_IO => Self::Io,
            LIBUSB_ERROR_INVALID_PARAM => Self::InvalidParam,
            LIBUSB_ERROR_ACCESS => Self::Access,
            LIBUSB_ERROR_NO_DEVICE => Self::NoDevice,
            LIBUSB_ERROR_NOT_FOUND => Self::NotFound,
            LIBUSB_ERROR_BUSY => Self::Busy,
            LIBUSB_ERROR_TIMEOUT => Self::Timeout,
            LIBUSB_ERROR_OVERFLOW => Self::Overflow,
            LIBUSB_ERROR_PIPE => Self::Pipe,
            LIBUSB_ERROR_INTERRUPTED => Self::Interrupted,
            LIBUSB_ERROR_NO_MEM => Self::NoMem,
            LIBUSB_ERROR_NOT_SUPPORTED => Self::NotSupported,
            LIBUSB_ERROR_OTHER => Self::Other,
            other => {
                eprintln!(
                    "{other} is not a recognized LIBUSB_ERROR code. Mapping to LIBUSB_ERROR_OTHER"
                );
                Self::Other
            }
        }
    }
}

/// Shorthand to quickly build a Result from an rtlsdr error return code
#[track_caller]
fn make_result(what: &'static str, raw: c_int) -> Result<()> {
    if raw < 0 {
        Err(RtlSdrError {
            what,
            code: ErrorCode::from_raw(raw),
        })
    } else {
        Ok(())
    }
}

pub struct RtlSdrDevice {
    dev: rtlsdr_dev_t,
    index: u32,
}

impl RtlSdrDevice {
    /// [`rtlsdr_open()`]
    pub fn open(index: u32) -> Result<Self> {
        unsafe {
            let mut dev = rtlsdr_dev_t::null();
            make_result("rtlsdr_open", rtlsdr_open(&mut dev, index))?;
            debug_assert!(!dev.raw().is_null());

            Ok(Self { dev, index })
        }
    }

    /// [`rtlsdr_close()`]
    pub fn close(self) -> Result<()> {
        unsafe {
            make_result("rtlsdr_close", rtlsdr_close(self.dev))?;

            // Cannot let Drop run after we closed the device
            let _ = ManuallyDrop::new(self);

            Ok(())
        }
    }
}

impl RtlSdrDevice {
    /// [`rtlsdr_get_device_name()`]
    pub fn name(&self) -> String {
        unsafe {
            let p_str = rtlsdr_get_device_name(self.index);
            if !p_str.is_null() {
                CStr::from_ptr(p_str).to_string_lossy().to_string()
            } else {
                "".to_string()
            }
        }
    }

    /// [`rtlsdr_get_device_usb_strings()`]
    pub fn maufacture(&self) -> Result<String> {
        let mut buf = [0 as c_char; 256];
        unsafe {
            make_result(
                "rtlsdr_get_device_usb_strings",
                rtlsdr_get_device_usb_strings(
                    self.index,
                    buf.as_mut_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ),
            )?;

            Ok(CStr::from_ptr(&buf as *const c_char)
                .to_string_lossy()
                .to_string())
        }
    }

    /// [`rtlsdr_get_device_usb_strings()`]
    pub fn product(&self) -> Result<String> {
        let mut buf = [0 as c_char; 256];
        unsafe {
            make_result(
                "rtlsdr_get_device_usb_strings",
                rtlsdr_get_device_usb_strings(
                    self.index,
                    core::ptr::null_mut(),
                    buf.as_mut_ptr(),
                    core::ptr::null_mut(),
                ),
            )?;

            Ok(CStr::from_ptr(&buf as *const c_char)
                .to_string_lossy()
                .to_string())
        }
    }

    /// [`rtlsdr_get_device_usb_strings()`]
    pub fn serial(&self) -> Result<String> {
        let mut buf = [0 as c_char; 256];
        unsafe {
            make_result(
                "rtlsdr_get_device_usb_strings",
                rtlsdr_get_device_usb_strings(
                    self.index,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    buf.as_mut_ptr(),
                ),
            )?;

            Ok(CStr::from_ptr(&buf as *const c_char)
                .to_string_lossy()
                .to_string())
        }
    }
}

/// Calls [`rtlsdr_close()`] and panics on failure.
///
/// To close the device while handling a returned error gracefully, use [`ManuallyDrop`] and [`RtlSdrDevice::close()`].
///
/// Note: If someone else calls `rtlsdr_close()` on this device, this crashes as a use-after-free.
/// `rtlsdr_close()` guards against NULL, but it cannot guard against dangling.
impl Drop for RtlSdrDevice {
    fn drop(&mut self) {
        unsafe {
            make_result("rtlsdr_close", rtlsdr_close(self.dev)).expect("Closing device failed");
        }
    }
}

/// [`rtlsdr_get_device_count()`] + [`rtlsdr_open()`]
pub fn all_rtlsdr_devices() -> Vec<Result<RtlSdrDevice>> {
    let num = rtlsdr_get_device_count();
    let mut devices = vec![];
    for i in 0..num {
        devices.push(RtlSdrDevice::open(i));
    }

    devices
}
