use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=Leap");
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rerun-if-changed=wrapper.cpp");
    println!("cargo:rerun-if-env-changed=LEAPCPP_REGEN");

    // Compile and link against our wrapper
    cc::Build::new()
        .cpp(true)
        .file("wrapper.cpp")
        .compile("LeapRust");

    if env::var_os("LEAPCPP_REGEN").is_some() {
        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let bindings = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.hpp")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .allowlist_function("Leap.*")
            .allowlist_type("Leap.*")
            .allowlist_var("Leap.*")
            .opaque_type("std::.*")
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        /*let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");*/

        let out_path = PathBuf::from("./src");
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}