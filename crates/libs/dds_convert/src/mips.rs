//! Mipmap generation: a 2x2 box average of straight-alpha RGBA8 pixels.

/// One RGBA8 mip level, tightly packed row-major.
#[derive(Clone)]
pub(crate) struct Mip {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Builds the full mip chain below `top`, down to 1x1. Each output pixel is
/// the 2x2 box average of the level above, rounded to nearest; on an odd
/// edge the row or column that exists is averaged alone.
pub(crate) fn generate(width: u32, height: u32, top: &[u8]) -> Vec<Mip> {
    let mut chain = Vec::new();
    let (mut width, mut height) = (width, height);
    while width > 1 || height > 1 {
        let pixels = chain
            .last()
            .map_or(top, |level: &Mip| level.pixels.as_slice());
        let level = downsample(width, height, pixels);
        width = level.width;
        height = level.height;
        chain.push(level);
    }
    chain
}

fn downsample(source_width: u32, source_height: u32, source: &[u8]) -> Mip {
    let width = (source_width / 2).max(1);
    let height = (source_height / 2).max(1);
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            // The 2x2 source block, clamped to the pixels that exist.
            let x0 = 2 * x;
            let y0 = 2 * y;
            let x1 = (x0 + 1).min(source_width - 1);
            let y1 = (y0 + 1).min(source_height - 1);
            let mut sums = [0u32; 4];
            for sy in y0..=y1 {
                for sx in x0..=x1 {
                    let at = ((sy * source_width + sx) * 4) as usize;
                    for (channel, sum) in sums.iter_mut().enumerate() {
                        *sum += u32::from(source[at + channel]);
                    }
                }
            }
            let count = (x1 - x0 + 1) * (y1 - y0 + 1);
            let out = ((y * width + x) * 4) as usize;
            for channel in 0..4 {
                pixels[out + channel] = ((sums[channel] + count / 2) / count) as u8;
            }
        }
    }
    Mip {
        width,
        height,
        pixels,
    }
}
