fn main() {
    if needs_binaries() {
        link_rtlsdr();
        link_libusb_1_0();
    } else {
        println!("Skipping link step for non-binary build");
    }
}

// librtlsdr is effectively a thin wrapper ontop of libusb-1.0 (not to be confused with libusb)
fn link_rtlsdr() {
    if try_find_link_paths("DSP_RTLSDR_LIB") {
        println!(
            "Found librltsdr libs with DSP_RTLSDR_LIB: {:?}",
            std::env::var("DSP_RTLSDR_LIB")
        );
        return;
    }

    if let Ok(pkg) = pkg_config::Config::new()
        .atleast_version("2.0")
        .probe("librtlsdr")
    {
        println!("Found librltsdr lib with pkg-config: {pkg:#?}");
    } else {
        println!("cargo::rustc-link-lib=rtlsdr");
        println!(
            "cargo::warning=Did NOT find librltsdr search path. You may need to set DSP_RTLSDR_LIB if linking fails."
        );
    }
}

// libusb-1.0 lets us talk to USB devices. rtlsdr is built ontop of this, but we use some of its
// functions for error handling explicitly.
fn link_libusb_1_0() {
    if try_find_link_paths("DSP_LIBUSB1_LIB") {
        println!(
            "Found libusb-1.0 libs with DSP_LIBUSB1_LIB: {:?}",
            std::env::var("DSP_LIBUSB1_LIB")
        );
        return;
    }

    if let Ok(pkg) = pkg_config::Config::new().probe("libusb-1.0") {
        println!("Found libusb-1.0 lib with pkg-config: {pkg:#?}");
    } else {
        println!("cargo::rustc-link-lib=libusb-1.0");
        println!(
            "cargo::warning=Did NOT find libusb-1.0 search path. You may need to set DSP_LIBUSB1_LIB if linking fails."
        );
    }
}

fn try_env_var(var: &str) -> Option<String> {
    println!("cargo:rerun-if-env-changed={var}");
    std::env::var(var).ok()
}

fn try_find_link_paths(lib_envvar: &str) -> bool {
    use std::path::Path;

    if let Some(lib) = try_env_var(lib_envvar)
        && !lib.trim().is_empty()
    {
        let lib = Path::new(&lib);
        if std::fs::exists(lib).ok() != Some(true) {
            println!("cargo::warning=Unable to find lib from {lib_envvar}: {lib:?}");
        }

        // Break off the filename to get the lib name
        let mut lib_name = lib.file_stem().unwrap().to_string_lossy().to_string();
        // kill me
        let is_windows_target = std::env::var("CARGO_CFG_WINDOWS").is_ok();
        if !is_windows_target && lib_name.starts_with("lib") {
            lib_name = lib_name.strip_prefix("lib").unwrap().into();
        }

        let dirname = lib.parent().unwrap();
        println!("cargo::rustc-link-search=native={}", dirname.display());

        let kind = match lib
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
            .as_str()
        {
            "dll" | "so" | "dylib" => "dylib",
            _ => "static",
        };
        println!("cargo::rustc-link-lib={kind}={lib_name}");

        true
    } else {
        false
    }
}

/// Checks whether this build job needs build artifacts or not.
///
/// For example: clippy & doc builds do not, but anything that produces a binary does.
fn needs_binaries() -> bool {
    use std::sync::Once;

    let is_clippy = std::env::var("RUSTC_WORKSPACE_WRAPPER")
        .or_else(|_| std::env::var("RUSTC_WRAPPER"))
        .map(|v| v.contains("clippy"))
        .unwrap_or(false);

    // TODO: Detect doc builds

    let should_build = !is_clippy;

    static LOG_ONCE: Once = Once::new();
    LOG_ONCE.call_once(|| {
        println!("Building? {should_build}");
        println!("is_clippy={is_clippy}");
    });

    should_build
}
