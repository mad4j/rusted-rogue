/// Returns 32×32 RGBA pixel data for the roguelike `@` player icon.
/// Background: dark navy #1a1a2e · Foreground: gold #ffd700.
pub fn icon_rgba() -> Vec<u8> {
    make_icon_rgba(32)
}

/// Generates RGBA pixel data at `size`×`size` for the `@` icon.
/// The shape is computed geometrically so it scales cleanly.
pub fn make_icon_rgba(size: u32) -> Vec<u8> {
    let sz = size as usize;
    // Center slightly left so the horizontal tail fits inside the canvas.
    let cx = size as f32 * 0.44;
    let cy = size as f32 * 0.47;

    // Outer C-ring radii.
    let outer_min = size as f32 * 0.30;
    let outer_max = size as f32 * 0.39;
    // Inner ring radii.
    let inner_min = size as f32 * 0.11;
    let inner_max = size as f32 * 0.19;
    // Vertical bar that bridges the inner-ring gap (right side of inner @).
    let vbar_dx0 = size as f32 * 0.17;
    let vbar_dx1 = size as f32 * 0.23;
    let vbar_dy = size as f32 * 0.22;
    // Horizontal tail that closes the outer ring on the right.
    let tail_dx0 = size as f32 * 0.37;
    let tail_dx1 = size as f32 * 0.54;
    let tail_dy = size as f32 * 0.05;

    let bg = [26u8, 26, 46, 255]; // #1a1a2e
    let fg = [255u8, 215, 0, 255]; // #ffd700

    let mut buf = vec![0u8; sz * sz * 4];
    for i in 0..sz * sz {
        buf[i * 4..i * 4 + 4].copy_from_slice(&bg);
    }

    for row in 0..sz {
        for col in 0..sz {
            let dx = col as f32 + 0.5 - cx;
            let dy = row as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx).to_degrees(); // −180° … +180°

            // Outer C-ring: full ring minus the right-side opening.
            let outer = dist >= outer_min
                && dist <= outer_max
                && !(angle >= -20.0 && angle <= 45.0);

            // Inner ring: small ring, opening on the right side (the inner 'a').
            let inner = dist >= inner_min
                && dist <= inner_max
                && !(angle >= -5.0 && angle <= 40.0);

            // Vertical bar: fills the inner-ring right gap, staying within the outer ring.
            let vbar = dx >= vbar_dx0
                && dx <= vbar_dx1
                && dy.abs() <= vbar_dy
                && dist <= outer_max;

            // Horizontal tail: exits the outer-ring opening to the right.
            let tail = dy.abs() <= tail_dy && dx >= tail_dx0 && dx <= tail_dx1;

            if outer || inner || vbar || tail {
                let b = (row * sz + col) * 4;
                buf[b..b + 4].copy_from_slice(&fg);
            }
        }
    }

    buf
}
