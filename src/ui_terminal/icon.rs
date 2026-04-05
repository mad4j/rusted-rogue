/// Returns 32×32 RGBA pixel data for the roguelike `@` player icon.
/// Background: transparent · Foreground: black #000000.
pub fn icon_rgba() -> Vec<u8> {
    make_icon_rgba(32)
}

/// Generates RGBA pixel data at `size`×`size` for the `@` icon.
/// Uses a classic 8×8 bitmap glyph scaled to `size` with nearest-neighbor.
pub fn make_icon_rgba(size: u32) -> Vec<u8> {
    // Classic 8×8 bitmap of '@' (MSB = leftmost column).
    #[rustfmt::skip]
    const GLYPH: [u8; 8] = [
        0b00111100, // ..####..   outer arc top
        0b01000010, // .#....#.   outer arc sides
        0b10011010, // #..##.#.   inner ring top
        0b10100110, // #.#..##.   inner sides + vertical bar
        0b10100111, // #.#..###   inner sides + tail exit (right)
        0b10011100, // #..###..   inner ring bottom
        0b01000000, // .#......   outer arc bottom-left
        0b00111100, // ..####..   outer arc bottom
    ];

    let sz = size as usize;
    // Initialised all-zero: R=0 G=0 B=0 A=0 → transparent black.
    let mut buf = vec![0u8; sz * sz * 4];

    for row in 0..sz {
        for col in 0..sz {
            let glyph_row = (row * 8) / sz;
            let glyph_col = (col * 8) / sz;
            let bit = 7 - glyph_col;
            if (GLYPH[glyph_row] >> bit) & 1 == 1 {
                // Black pixel: R/G/B stay 0, set alpha = 255.
                buf[(row * sz + col) * 4 + 3] = 255;
            }
        }
    }

    buf
}
