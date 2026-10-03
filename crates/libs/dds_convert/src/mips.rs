//! Mipmap generation: a 2x2 box average of straight-alpha RGBA8 pixels.

use std::borrow::Cow;

/// One RGBA8 mip level, tightly packed row-major. A level copied from a
/// decoded source borrows; a generated level owns.
#[derive(Clone)]
pub(crate) struct Mip<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: Cow<'a, [u8]>,
}

/// The size of the level below a `width`x`height` one: each side halved,
/// rounded down, and at least 1.
fn halved(width: u32, height: u32) -> (u32, u32) {
    ((width / 2).max(1), (height / 2).max(1))
}

/// How many levels the full chain of a `width`x`height` texture holds, its
/// top level included: halving until both sides are 1.
pub(crate) fn chain_len(width: u32, height: u32) -> u32 {
    let mut levels = 1;
    let (mut width, mut height) = (width, height);
    while width > 1 || height > 1 {
        (width, height) = halved(width, height);
        levels += 1;
    }
    levels
}

/// Builds the full mip chain below `top`, down to 1x1. Each output pixel is
/// the 2x2 box average of the level above, rounded to nearest; on an odd
/// edge the row or column that exists is averaged alone.
pub(crate) fn generate(width: u32, height: u32, top: &[u8]) -> Vec<Mip<'static>> {
    let below = chain_len(width, height) - 1;
    let mut chain: Vec<Mip<'static>> = Vec::with_capacity(below as usize);
    for _ in 0..below {
        let level = match chain.last() {
            Some(above) => downsample(above.width, above.height, &above.pixels),
            None => downsample(width, height, top),
        };
        chain.push(level);
    }
    chain
}

fn downsample(source_width: u32, source_height: u32, source: &[u8]) -> Mip<'static> {
    let (width, height) = halved(source_width, source_height);
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
        pixels: Cow::Owned(pixels),
    }
}
