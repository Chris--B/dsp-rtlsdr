#![allow(nonstandard_style)]

use crate::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct rtlsdr_dev {
    _unused: [u8; 0],
}

pub type rtlsdr_tuner = c_uint;
pub const RTLSDR_TUNER_UNKNOWN: rtlsdr_tuner = 0;
pub const RTLSDR_TUNER_E4000: rtlsdr_tuner = 1;
pub const RTLSDR_TUNER_FC0012: rtlsdr_tuner = 2;
pub const RTLSDR_TUNER_FC0013: rtlsdr_tuner = 3;
pub const RTLSDR_TUNER_FC2580: rtlsdr_tuner = 4;
pub const RTLSDR_TUNER_R820T: rtlsdr_tuner = 5;
pub const RTLSDR_TUNER_R828D: rtlsdr_tuner = 6;

pub const EINVAL: c_int = 22;

#[rustfmt::skip]
pub type rtlsdr_read_async_cb_t = Option<
    unsafe extern "C" fn(
        buf: *mut u8,
        len: u32,
        ctx: *mut c_void
    )
>;

unsafe extern "C" {

    pub safe fn rtlsdr_get_device_count() -> u32;

    pub fn rtlsdr_get_device_name(index: u32) -> *const c_char;

    /// Get USB device strings.
    ///
    /// NOTE: The string arguments must provide space for up to 256 bytes.
    ///
    /// ## Params
    /// - `index`: the device index
    /// - `manufact`: manufacturer name, may be `NULL`
    /// - `product`: product name, may be `NULL`
    /// - `serial`: serial number, may be `NULL`
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_get_device_usb_strings(
        index: u32,
        manufact: *mut c_char,
        product: *mut c_char,
        serial: *mut c_char,
    ) -> c_int;

    /// Get device index by USB serial string descriptor.
    ///
    /// ## Params
    /// - `serial`: serial string of the device
    ///
    /// ## Return Value
    /// - device index of first device where the name matched
    /// - `-1` if name is `NULL`
    /// - `-2` if no devices were found at all
    /// - `-3` if devices were found, but none with matching name
    pub fn rtlsdr_get_index_by_serial(serial: *const c_char) -> c_int;

    /// Opens a new RTLSDR device with the given index.
    ///
    /// To enumerate devices available, use [`rtlsdr_get_device_count()`] and [`rtlsdr_get_device_name()`]
    ///
    /// ## Params
    /// - `dev`: pointer to receive the opened device
    /// - `index`: index of device to open.
    pub fn rtlsdr_open(dev: *mut rtlsdr_dev_t, index: u32) -> c_int;

    /// Close the RTLSDR device.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_close(dev: rtlsdr_dev_t) -> c_int;

    /// Set crystal oscillator frequencies used for the RTL2832 and the tuner IC.
    ///
    /// Usually both ICs use the same clock. Changing the clock may make sense if
    /// you are applying an external clock to the tuner or to compensate the
    /// frequency (and samplerate) error caused by the original (cheap) crystal.
    ///
    /// NOTE: Call this function only if you fully understand the implications.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `rtl_freq`: frequency value used to clock the RTL2832 in Hz
    /// - `tuner_freq`: frequency value used to clock the tuner IC in Hz
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_xtal_freq(dev: rtlsdr_dev_t, rtl_freq: u32, tuner_freq: u32) -> c_int;

    /// Get crystal oscillator frequencies used for the RTL2832 and the tuner IC.
    ///
    /// Usually both ICs use the same clock.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `rtl_freq`: frequency value used to clock the RTL2832 in Hz
    /// - `tuner_freq`: frequency value used to clock the tuner IC in Hz
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_get_xtal_freq(
        dev: rtlsdr_dev_t,
        rtl_freq: *mut u32,
        tuner_freq: *mut u32,
    ) -> c_int;

    /// Get USB device strings.
    ///
    /// NOTE: The string arguments must provide space for up to 256 bytes.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `manufact`: manufacturer name, may be `NULL`
    /// - `product`: product name, may be `NULL`
    /// - `serial`: serial number, may be `NULL`
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_get_usb_strings(
        dev: rtlsdr_dev_t,
        manufact: *mut c_char,
        product: *mut c_char,
        serial: *mut c_char,
    ) -> c_int;

    /// Write the device EEPROM.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `data`: buffer of data to be written
    /// - `offset`: address where the data should be written
    /// - `len`: length of the data
    ///
    /// ## Return Value
    /// - `0` on success
    /// - `-1` if device handle is invalid
    /// - `-2` if EEPROM size is exceeded
    /// - `-3` if no EEPROM was found
    pub fn rtlsdr_write_eeprom(dev: rtlsdr_dev_t, data: *mut u8, offset: u8, len: u16) -> c_int;

    /// Read the device EEPROM.
    ///
    /// ## Params
    /// - `dev` the device handle given by [`rtlsdr_open()`]
    /// - `data` buffer where the data should be written
    /// - `offset` address where the data should be read from
    /// - `len` length of the data
    ///
    /// ## Return Value
    /// - `0` on success
    /// - `-1` if device handle is invalid
    /// - `-2` if EEPROM size is exceeded
    /// - `-3` if no EEPROM was found
    pub fn rtlsdr_read_eeprom(dev: rtlsdr_dev_t, data: *mut u8, offset: u8, len: u16) -> c_int;

    /// Set the frequency the device is tuned to.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `freq`: frequency in Hz
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_center_freq(dev: rtlsdr_dev_t, freq: u32) -> c_int;

    /// Get actual frequency the device is tuned to.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// return 0 on error, frequency in Hz otherwise
    pub fn rtlsdr_get_center_freq(dev: rtlsdr_dev_t) -> u32;

    /// Set the frequency correction value for the device.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `ppm`: correction value in parts per million (ppm)
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_freq_correction(dev: rtlsdr_dev_t, ppm: c_int) -> c_int;

    /// Get actual frequency correction value of the device.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// return correction value in parts per million (ppm)
    pub fn rtlsdr_get_freq_correction(dev: rtlsdr_dev_t) -> c_int;

    /// Get the tuner type.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// return [RTLSDR_TUNER_UNKNOWN] on error, tuner type otherwise
    pub fn rtlsdr_get_tuner_type(dev: rtlsdr_dev_t) -> rtlsdr_tuner;

    /// Get a list of gains supported by the tuner.
    ///
    /// NOTE: The gains argument must be preallocated by the caller. If `NULL` is
    /// being given instead, the number of available gain values will be returned.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `gains`: array of gain values. In tenths of a dB, 115 means 11.5 dB.
    ///
    /// ## Return Value
    /// return <= `0` on error, number of available (returned) gain values otherwise
    pub fn rtlsdr_get_tuner_gains(dev: rtlsdr_dev_t, gains: *mut c_int) -> c_int;

    /// Set the gain for the device.
    ///
    /// Manual gain mode must be enabled for this to work.
    ///
    /// Valid gain values (in tenths of a dB) for the E4000 tuner:
    /// - -10, 15, 40, 65, 90, 115, 140, 165, 190,
    /// - 215, 240, 290, 340, 420, 430, 450, 470, 490
    ///
    /// Valid gain values may be queried with [rtlsdr_get_tuner_gains()] function.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `gain`: in tenths of a dB, 115 means 11.5 dB.
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_tuner_gain(dev: rtlsdr_dev_t, gain: c_int) -> c_int;

    /// Set the bandwidth for the device.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `bw`: bandwidth in Hz. Zero means automatic BW selection.
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_tuner_bandwidth(dev: rtlsdr_dev_t, bw: u32) -> c_int;

    /// Get actual gain the device is configured to.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// return `0` on error, gain in tenths of a dB, 115 means 11.5 dB.
    pub fn rtlsdr_get_tuner_gain(dev: rtlsdr_dev_t) -> c_int;

    /// Set the intermediate frequency gain for the device.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `stage`: intermediate frequency gain stage number (1 to 6 for E4000)
    /// - `gain`: in tenths of a dB, -30 means -3.0 dB.
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_tuner_if_gain(dev: rtlsdr_dev_t, stage: c_int, gain: c_int) -> c_int;

    /// Set the gain mode (automatic/manual) for the device.
    ///
    /// Manual gain mode must be enabled for the gain setter function to work.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `manual`: gain mode, `1` means manual gain mode shall be enabled.
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_tuner_gain_mode(dev: rtlsdr_dev_t, manual: c_int) -> c_int;

    /// Set the sample rate for the device.
    ///
    /// Also selects the baseband filters according to the requested sample rate for tuners where this is possible.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `samp_rate`: the sample rate to be set, possible values are:
    ///     - `225_001` - `300_000` Hz
    ///     - `900_001` - `320_0000` Hz
    ///     - sample loss is to be expected for rates > `2_400_000`
    ///
    /// ## Return Value
    /// return `0` on success, `-EINVAL` on invalid rate
    pub fn rtlsdr_set_sample_rate(dev: rtlsdr_dev_t, rate: u32) -> c_int;

    /// Get actual sample rate the device is configured to.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// return `0` on error, sample rate in Hz otherwise
    pub fn rtlsdr_get_sample_rate(dev: rtlsdr_dev_t) -> u32;

    /// Enable test mode that returns an 8 bit counter instead of the samples.
    ///
    /// The counter is generated inside the RTL2832.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `on`: test mode
    ///     - `1` means enabled
    ///     - `0` disabled
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_testmode(dev: rtlsdr_dev_t, on: c_int) -> c_int;

    /// Enable or disable the internal digital AGC of the RTL2832.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `on`: digital AGC mode
    ///     - `1` means enabled
    ///     - `0` disabled
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_agc_mode(dev: rtlsdr_dev_t, on: c_int) -> c_int;

    /// Enable or disable the direct sampling mode.
    ///
    /// When enabled, the IF mode
    /// of the RTL2832 is activated, and rtlsdr_set_center_freq() will control
    /// the IF-frequency of the DDC, which can be used to tune from 0 to 28.8 MHz
    /// (xtal frequency of the RTL2832).
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `on`:
    ///     - `0` means disabled
    ///     - `1` I-ADC input enabled
    ///     - `2` Q-ADC input enabled
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_direct_sampling(dev: rtlsdr_dev_t, on: c_int) -> c_int;

    /// Get state of the direct sampling mode.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// - `-1` on error,
    /// - `0` means disabled
    /// - `1` I-ADC input enabled
    /// - `2` Q-ADC input enabled
    pub fn rtlsdr_get_direct_sampling(dev: rtlsdr_dev_t) -> c_int;

    /// Enable or disable offset tuning for zero-IF tuners.
    ///
    /// This allows to avoid problems caused by the DC offset of the ADCs and 1/f noise.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `on`:
    ///     - `0` means disabled
    ///     - `1` enabled
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_set_offset_tuning(dev: rtlsdr_dev_t, on: c_int) -> c_int;

    /// Get state of the offset tuning mode.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// - `-1` on error
    /// - `0` means disabled
    /// - `1` enabled
    pub fn rtlsdr_get_offset_tuning(dev: rtlsdr_dev_t) -> c_int;

    pub fn rtlsdr_reset_buffer(dev: rtlsdr_dev_t) -> c_int;

    pub fn rtlsdr_read_sync(
        dev: rtlsdr_dev_t,
        buf: *mut u8,
        len: c_int,
        n_read: *mut c_int,
    ) -> c_int;

    /// Read samples from the device asynchronously.
    ///
    /// This function will block until
    /// it is being canceled using rtlsdr_cancel_async()
    ///
    /// NOTE: This function is deprecated and is subject for removal.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `cb` callback function to return received samples
    /// - `ctx` user specific context to pass via the callback function
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_wait_async(
        dev: rtlsdr_dev_t,
        cb: rtlsdr_read_async_cb_t,
        ctx: *mut c_void,
    ) -> c_int;

    /// Read samples from the device asynchronously.
    ///
    /// This function will block until
    /// it is being canceled using rtlsdr_cancel_async()
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `cb:` callback function to return received samples
    /// - `ctx:` user specific context to pass via the callback function
    /// - `buf_num:` optional buffer count, `buf_num * buf_len = overall buffer size`.
    ///     - set to `0` for default buffer count (`15`)
    /// - `buf_len:` optional buffer length
    ///     - must be multiple of `512`
    ///     - should be a multiple of `16384` ([URB](https://docs.kernel.org/driver-api/usb/URB.html) size)
    ///     - set to `0` for default buffer length (`16 * 32 * 512`)
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_read_async(
        dev: rtlsdr_dev_t,
        cb: rtlsdr_read_async_cb_t,
        ctx: *mut c_void,
        buf_num: u32,
        buf_len: u32,
    ) -> c_int;

    /// Cancel all pending asynchronous operations on the device.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    ///
    /// ## Return Value
    /// return `0` on success
    pub fn rtlsdr_cancel_async(dev: rtlsdr_dev_t) -> c_int;

    /// Enable or disable the bias tee on `GPIO PIN 0`.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `on`:
    ///     - `1` for Bias T on
    ///     - `0` for Bias T off
    ///
    /// ## Return Value
    /// return `-1` if device is not initialized. `0` otherwise.
    pub fn rtlsdr_set_bias_tee(dev: rtlsdr_dev_t, on: c_int) -> c_int;

    /// Enable or disable the bias tee on the given GPIO pin.
    ///
    /// ## Params
    /// - `dev`: the device handle given by [`rtlsdr_open()`]
    /// - `gpio`: the gpio pin to configure as a Bias T control.
    /// - `on`:
    ///     - `1` for Bias T on
    ///     - `0` for Bias T off
    ///
    /// ## Return Value
    /// return `-1` if device is not initialized. `0` otherwise.
    pub fn rtlsdr_set_bias_tee_gpio(dev: rtlsdr_dev_t, gpio: c_int, on: c_int) -> c_int;
}
