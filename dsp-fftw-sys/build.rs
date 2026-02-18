fn main() {
    link_fftw3();
}

fn link_fftw3() {
    let mut found_all_libs = true;




    for lib in ["fftw3", "fftw3f", "fftw3l"] {
        if let Ok(pkg) = pkg_config::Config::new()
            .atleast_version("3.0")
            .statik(true)
            .probe(lib)
        {
            println!("Found {lib} lib with pkg-config: {pkg:#?}");
        } else {
            println!("cargo:warning=Couldn't find {lib} with pkg-config");
            found_all_libs = false;
        }

        println!("cargo:rustc-link-lib=static={lib}_threads");
    }

    assert!(found_all_libs);
}
