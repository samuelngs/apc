fn main() {
    #[cfg(target_os = "macos")]
    {
        let deps_lib = concat!(env!("CARGO_MANIFEST_DIR"), "/../deps/out/lib");
        println!("cargo:rustc-link-lib=dylib=krun");
        println!("cargo:rustc-link-search={deps_lib}");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{deps_lib}");
        println!("cargo:rustc-link-lib=framework=IOSurface");
    }
}
