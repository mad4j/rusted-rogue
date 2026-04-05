fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    embed_windows_icon();
}

#[cfg(windows)]
fn embed_windows_icon() {
    use std::io::BufWriter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ico_path = format!("{manifest_dir}/resources/icon.ico");

    // Regenerate the ICO from programmatic pixel data every build.
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for &size in &[16u32, 32, 48, 256] {
        let rgba = make_icon_rgba(size);
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

/// Generates RGBA pixel data for the roguelike `@` icon at `size`×`size`.
/// Kept in sync with `src/ui_terminal/icon.rs`.
fn make_icon_rgba(size: u32) -> Vec<u8> {
    let sz = size as usize;
    let cx = size as f32 * 0.44;
    let cy = size as f32 * 0.47;

    let outer_min = size as f32 * 0.30;
    let outer_max = size as f32 * 0.39;
    let inner_min = size as f32 * 0.11;
    let inner_max = size as f32 * 0.19;
    let vbar_dx0 = size as f32 * 0.17;
    let vbar_dx1 = size as f32 * 0.23;
    let vbar_dy = size as f32 * 0.22;
    let tail_dx0 = size as f32 * 0.37;
    let tail_dx1 = size as f32 * 0.54;
    let tail_dy = size as f32 * 0.05;

    let bg = [26u8, 26, 46, 255];
    let fg = [255u8, 215, 0, 255];

    let mut buf = vec![0u8; sz * sz * 4];
    for i in 0..sz * sz {
        buf[i * 4..i * 4 + 4].copy_from_slice(&bg);
    }

    for row in 0..sz {
        for col in 0..sz {
            let dx = col as f32 + 0.5 - cx;
            let dy = row as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx).to_degrees();

            let outer = dist >= outer_min
                && dist <= outer_max
                && !(angle >= -20.0 && angle <= 45.0);
            let inner = dist >= inner_min
                && dist <= inner_max
                && !(angle >= -5.0 && angle <= 40.0);
            let vbar = dx >= vbar_dx0
                && dx <= vbar_dx1
                && dy.abs() <= vbar_dy
                && dist <= outer_max;
            let tail = dy.abs() <= tail_dy && dx >= tail_dx0 && dx <= tail_dx1;

            if outer || inner || vbar || tail {
                let b = (row * sz + col) * 4;
                buf[b..b + 4].copy_from_slice(&fg);
            }
        }
    }

    buf
}
