use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();

    let lib_dir = if target.contains("i686") {
        "C:/TwinCAT/AdsApi/TcAdsDll/Lib"
    } else {
        "C:/TwinCAT/AdsApi/TcAdsDll/x64/lib"
    };

    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=dylib=TcAdsDll");

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("--include-directory=C:/TwinCAT/AdsApi/TcAdsDll/Include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    bindings.write_to_file(&out_path).unwrap();
}
