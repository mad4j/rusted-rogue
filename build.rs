fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resources/icon.ico");

    #[cfg(windows)]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let ico_path = format!("{manifest_dir}/resources/icon.ico");
        winresource::WindowsResource::new()
            .set_icon(&ico_path)
            .compile()
            .expect("embed icon resource");
    }
}
