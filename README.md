
This repo houses Rust bindings to `librtlsdr`. They aim to make using your RTL-SDR dongle a little bit easier to work with.

For more info on RTL-SDR hardware, see:
- https://www.rtl-sdr.com/about-rtl-sdr/
- https://www.rtl-sdr.com/rtl-sdr-quick-start-guide/
- https://osmocom.org/projects/rtl-sdr/wiki

## What's Included?

#### Documentation
For comprehensive documentation on each crate, it's recommended to clone the repo and generate the Rust docs directly:
```sh
cargo doc --open
```

#### `dsp-rtlsdr-rs`
This crate offers high level bindings built on top of ``dsp-rtlsdr-sys`. They aim to make using this from Rust feel natural. Note that this crate is not feature-complete yet.

#### `dsp-rtlsdr-sys`
This crate mirrors the `rtl-sdr.h` header as close as possible. It includes the comments from the header with light formatting tweaks to make reading `lib.rs` or the generated rustdoc pages easy. They can be used without or in junction with `dsp-rtlsdr-rs`.

## Building
All of the crates here build with standard `cargo` usage and can be included in a project directly from GitHub.
```sh
$ cargo add --git https://github.com/Chris--B/dsp-rtlsdr.git dsp-rtlsdr-rs
```
Because the repo has multiple crates, you'll need to specifiy which crate(s) you want to use. Note that the `-rs` crate re-exports the `-sys` crate.

### Installing `librtlsdr` and `libusb-1.0`
The `-sys` crate depends on both a locally installed `librtlsdr` and `libusb-1.0`. The most reliable way to use these is to install them with your local package manager.

```sh
# macOS
$ brew install librtlsdr libusb

# Fedora
$ dnf install rtl-sdr-devel libusb1-devel

# Ubuntu
$ apt install librtlsdr-dev libusb-1.0-0

# Windows is untested. Consult the next section for details on how to control how libs for `librtlsdr` and `libusb-1.0` are found.
```

Verify that `pkg-config` can find your installed library with:
```sh
pkg-config --libs librtlsdr
```

### Notes on linking
`dsp-rtlsdr-sys` depends on [`libusb-sys`](https://crates.io/crates/libusb-sys) and both use [`pkg-config`](https://crates.io/crates/pkg-config) by default to locate the libraries to link against. Consult `libusb-sys`'s docs for options on controlling how it locates the library to link against. `dsp-rtlsdr-sys` follows the same conventions, since they're from `pkg-config`. By default, the `-sys` crate will attempt to static link `librtlsdr`.

`dsp-rtlsdr-sys` also reads the `DSP_RTLSDR_LIB` environment variable to find a library to link against. This takes precedence over `pkg-config`. `DSP_RTLSDR_LIB` supports both static and dynamic libraries.

For example, if you have a local build you want to use:
```sh
$ DSP_RTLSDR_LIB=$PWD/_build/librtlsdr.a cargo build
```
You can set this in your shells, or with with `[env]` in your [`config.toml`](https://doc.rust-lang.org/cargo/reference/config.html).

## Samples
Once everything builds, checkout the example code in `dsp-rtlsdr-sys/examples/`
- `hello-rtlsdr-sys `demonstrates basic does-this-even-work usage of the `-sys` crate that can be used to make sure your device is functioning properly. They use the test mode of the dongle to output incrementing bytes.
```sh
cargo run --example hello-rtlsdr-sys
```

Note that only one application can hold the device at a time, so if it fails with device access errors check that something else isn't using it:
```
Found 1 device(s)
  + "Generic RTL2832U OEM"
  + Manufact: "RTLSDRBlog"
  + Product: "Blog V4"
  + Serial: "00000001"

Kernel driver is active, or device is claimed by second instance of librtlsdr.
In the first case, please either detach or blacklist the kernel module
(dvb_usb_rtl28xxu), or enable automatic detaching at compile time.

usb_claim_interface error -3
rtlsdr_open: "Access denied (insufficient permissions)" (-3)
```
