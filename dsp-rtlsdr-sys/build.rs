fn main() {
    link_rtlsdr();
}

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
        // librtlsdr is a thin wrapper around libusb and life is easier if we don't dynamically link it.
        .statik(true)
        .probe("librtlsdr")
    {
        println!("Found librltsdr lib with pkg-config: {pkg:#?}");
        return;
    }

    println!("cargo::rustc-link-lib=rtlsdr");
    println!(
        "cargo::warning=Did NOT find librltsdr search path. You may need to set DSP_RTLSDR_LIB if linking fails."
    );
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
