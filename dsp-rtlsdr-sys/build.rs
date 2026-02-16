fn main() {
    for maybe_path in [
        "/usr/local/lib64", //
        "/usr/local/lib",   //
    ] {
        if std::fs::exists(maybe_path).unwrap_or(false) {
            println!("cargo::rustc-link-search={}", maybe_path);
        }
    }

    link_rtlsdr();
}

fn link_rtlsdr() {
    if try_find_link_paths("DSP_RTLSDR_LIB") {
        println!("Found librtsdr libs with DSP_RTLSDR_LIB");
        return;
    }

    if let Ok(_pkg) = pkg_config::Config::new()
        .atleast_version("2.0")
        .probe("librtlsdr")
    {
        println!("Found librtsdr libs with pkg-config");
        return;
    }

    println!("cargo::rustc-link-lib=rtlsdr");
    println!(
        "cargo::warning=Did NOT find librtsdr search path. You may need to set DSP_RTLSDR_LIB if linking fails."
    );
}

fn try_env_var(var: &str) -> Option<String> {
    println!("cargo::rerun-if-env-changed={var}");
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

        // Break off the dirs to add to search path
        let dirname = lib.parent().unwrap();
        println!("cargo::rustc-link-search={}", dirname.display());

        // Break off the filename to get the lib name
        let mut lib = lib.file_stem().unwrap().to_string_lossy().to_string();
        // kill me
        let is_windows_target = std::env::var("CARGO_CFG_WINDOWS").is_ok();
        if !is_windows_target && lib.starts_with("lib") {
            lib = lib.strip_prefix("lib").unwrap().into();
        }
        println!("cargo::rustc-link-lib={lib}");

        true
    } else {
        false
    }
}
