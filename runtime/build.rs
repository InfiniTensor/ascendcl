use std::{env, path::PathBuf};

fn main() {
    use build_script_cfg::Cfg;
    use search_ascend_tools::{find_ascend_toolkit_home, watch_env_var};

    println!("cargo:rerun-if-changed=build.rs");
    watch_env_var();

    let ascend = Cfg::new("detected_ascend");
    let Some(ascend_home) = find_ascend_toolkit_home() else {
        return;
    };
    ascend.define();

    let rt_dir = ascend_home.join("runtime");
    println!(
        "cargo:rustc-link-search=native={}",
        rt_dir.join("lib64").display()
    );
    println!("cargo:rustc-link-lib=dylib=ascendcl");
    println!("cargo:rustc-link-lib=dylib=runtime");

    // Tell cargo to invalidate the built crate whenever the wrapper changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point to bindgen,
    // and lets you build up options for the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}", rt_dir.join("include").display()))
        // Only generate bindings for the functions in these namespaces.
        .allowlist_item("acl.*")
        // Annotate the given type with the #[must_use] attribute.
        .must_use_type("aclError")
        // Generate rust style enums.
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        // Use core instead of std in the generated bindings.
        .use_core()
        // Tell cargo to invalidate the built crate whenever any of the included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
