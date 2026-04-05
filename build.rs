fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resources/favicon-32x32.png");

    #[cfg(windows)]
    embed_windows_icon();
}

#[cfg(windows)]
fn embed_windows_icon() {
    use std::io::BufWriter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let favicon_path = format!("{manifest_dir}/resources/favicon-32x32.png");
    let ico_path = format!("{manifest_dir}/resources/icon.ico");

    // Load the source favicon PNG.
    let src = image::open(&favicon_path)
        .expect("open favicon-32x32.png")
        .into_rgba8();

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for &size in &[16u32, 32, 48, 256] {
        let resized = image::imageops::resize(
            &src,
            size,
            size,
            image::imageops::FilterType::Lanczos3,
        );
        let rgba: Vec<u8> = resized.into_raw();
        let image = ico::IconImage::from_rgba_data(size, size, rgba);
        icon_dir
            .add_entry(ico::IconDirEntry::encode(&image).expect("encode ico entry"));
    }
    let file = std::fs::File::create(&ico_path).expect("create icon.ico");
    icon_dir.write(BufWriter::new(file)).expect("write icon.ico");

    winresource::WindowsResource::new()
        .set_icon(&ico_path)
        .compile()
        .expect("embed icon resource");
}
