fn main() {
    println!("cargo:rerun-if-changed=native/helper.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("native/helper.m")
            .flag("-fobjc-arc")
            .flag("-fblocks")
            .flag("-mmacosx-version-min=14.0")
            .compile("coqui_helper_native");
        for f in ["Foundation", "Security", "AppKit"] {
            println!("cargo:rustc-link-lib=framework={f}");
        }
    }
}
