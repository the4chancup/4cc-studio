//! Kit-color extraction calibration: runs the plan's algorithm with parameters taken from the
//! command line over every (config, texture) pair in .tmp/kit_pairs.tsv and reports how often
//! the extracted colors agree with the colors the manager declared.
use std::fs;

use kit_config::{KitConfig, Rgb};
use pes_version::PesVersion;

#[derive(Clone, Copy)]
struct Rect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

struct Params {
    shirt: Rect,
    shorts: [Rect; 2],
    /// Quantization bits per channel.
    bits: u32,
    /// Bins closer than this (RGB euclid) merge into one cluster.
    merge: f32,
    /// Minimum share of the shirt region for a second shirt color.
    second_share: f32,
    /// Minimum distance (RGB euclid) between color 1 and 2.
    distinct: f32,
    /// Sampling stride in pixels.
    stride: usize,
}

struct Cluster {
    color: [f32; 3],
    share: f32,
}

fn sample(rgba: &[u8], width: usize, height: usize, rect: Rect, stride: usize, out: &mut Vec<[u8; 3]>) {
    let x0 = (rect.x0 * width as f32) as usize;
    let x1 = (rect.x1 * width as f32) as usize;
    let y0 = (rect.y0 * height as f32) as usize;
    let y1 = (rect.y1 * height as f32) as usize;
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            let at = 4 * (y * width + x);
            if rgba[at + 3] >= 128 {
                out.push([rgba[at], rgba[at + 1], rgba[at + 2]]);
            }
            x += stride;
        }
        y += stride;
    }
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn cluster(pixels: &[[u8; 3]], bits: u32, merge: f32) -> Vec<Cluster> {
    let shift = 8 - bits;
    let levels = 1usize << bits;
    // Histogram with per-bin color sums for the bin's mean.
    let mut count = vec![0u32; levels * levels * levels];
    let mut sum = vec![[0u64; 3]; levels * levels * levels];
    for p in pixels {
        let bin = ((p[0] >> shift) as usize * levels + (p[1] >> shift) as usize) * levels + (p[2] >> shift) as usize;
        count[bin] += 1;
        for c in 0..3 {
            sum[bin][c] += p[c] as u64;
        }
    }
    let mut bins: Vec<(u32, [f32; 3])> = count
        .iter()
        .zip(&sum)
        .filter(|(n, _)| **n > 0)
        .map(|(n, s)| (*n, [s[0] as f32 / *n as f32, s[1] as f32 / *n as f32, s[2] as f32 / *n as f32]))
        .collect();
    bins.sort_by(|a, b| b.0.cmp(&a.0));
    // Greedy merge: each bin joins the first (largest) cluster within `merge`.
    let mut clusters: Vec<(f64, [f64; 3])> = Vec::new();
    for (n, color) in bins {
        let n = n as f64;
        let found = clusters.iter_mut().find(|(w, mean)| {
            dist([mean[0] as f32, mean[1] as f32, mean[2] as f32], color) < merge && *w > 0.0
        });
        match found {
            Some((w, mean)) => {
                for c in 0..3 {
                    mean[c] = (mean[c] * *w + color[c] as f64 * n) / (*w + n);
                }
                *w += n;
            }
            None => clusters.push((n, [color[0] as f64, color[1] as f64, color[2] as f64])),
        }
    }
    clusters.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let total: f64 = clusters.iter().map(|c| c.0).sum();
    clusters
        .into_iter()
        .map(|(w, m)| Cluster { color: [m[0] as f32, m[1] as f32, m[2] as f32], share: (w / total) as f32 })
        .collect()
}

fn rgb(c: Rgb) -> [f32; 3] {
    [c.0 as f32, c.1 as f32, c.2 as f32]
}

fn hex(c: [f32; 3]) -> String {
    format!("{:02X}{:02X}{:02X}", c[0].round() as u8, c[1].round() as u8, c[2].round() as u8)
}

fn arg(name: &str, default: f32) -> f32 {
    std::env::args()
        .find_map(|a| a.strip_prefix(&format!("--{name}=")).map(|v| v.parse().unwrap()))
        .unwrap_or(default)
}

fn main() {
    let params = Params {
        shirt: Rect { x0: arg("sx0", 0.34), y0: arg("sy0", 0.02), x1: arg("sx1", 0.66), y1: arg("sy1", 0.90) },
        shorts: [
            Rect { x0: arg("hx0", 0.02), y0: arg("hy0", 0.59), x1: arg("hx1", 0.31), y1: arg("hy1", 0.90) },
            Rect { x0: 1.0 - arg("hx1", 0.31), y0: arg("hy0", 0.59), x1: 1.0 - arg("hx0", 0.02), y1: arg("hy1", 0.90) },
        ],
        bits: arg("bits", 5.0) as u32,
        merge: arg("merge", 24.0),
        second_share: arg("second", 0.25),
        distinct: arg("distinct", 60.0),
        stride: arg("stride", 4.0) as usize,
    };
    let verbose = std::env::args().any(|a| a == "-v");
    let dump = std::env::args().any(|a| a == "--dump");
    let mut dumped = String::new();
    let close = arg("close", 48.0);
    let pairs = fs::read_to_string(r"C:\Data\4cc\Tools_Mine\4cc-studio\.tmp\kit_pairs.tsv").unwrap();
    let mut n = 0;
    let (mut hit1, mut hit2, mut hit_shorts) = (0, 0, 0);
    let (mut sum1, mut sum2) = (0.0f32, 0.0f32);
    let mut second_from_shirt = 0;
    let (mut present1, mut first1, mut second_present, mut second_is_shorts) = (0, 0, 0, 0);
    let mut second_shares: Vec<f32> = Vec::new();
    let mut misranked: Vec<String> = Vec::new();
    for line in pairs.lines() {
        let (bin, dds) = line.split_once('\t').unwrap();
        let config = KitConfig::decode(&fs::read(bin).unwrap(), PesVersion::Pes17).unwrap();
        let decoded = dds_convert::decode(&fs::read(dds).unwrap(), dds_convert::SourceFormat::Dds).unwrap();
        let (w, h) = (decoded.width as usize, decoded.height as usize);
        let rgba = &decoded.mips[0];
        let mut shirt_px = Vec::new();
        sample(rgba, w, h, params.shirt, params.stride, &mut shirt_px);
        let mut shorts_px = Vec::new();
        for r in params.shorts {
            sample(rgba, w, h, r, params.stride, &mut shorts_px);
        }
        let shirt = cluster(&shirt_px, params.bits, params.merge);
        let shorts = cluster(&shorts_px, params.bits, params.merge);
        if shirt.is_empty() {
            continue;
        }
        let color1 = shirt[0].color;
        let (color2, from_shirt) = match shirt.get(1) {
            Some(c) if c.share >= params.second_share && dist(c.color, color1) >= params.distinct => (c.color, true),
            _ => (shorts.first().map(|c| c.color).unwrap_or(color1), false),
        };
        second_from_shirt += from_shirt as u32;
        let d1 = dist(color1, rgb(config.colors.shirt1));
        let d2 = dist(color2, rgb(config.colors.shirt2));
        let dsh = shorts.first().map(|c| dist(c.color, rgb(config.colors.shorts))).unwrap_or(999.0);
        n += 1;
        // Is the declared shirt color present at all among the shirt clusters (top 4)? If so,
        // did we rank it first? Same for the declared second color: present, and its share.
        let decl1 = rgb(config.colors.shirt1);
        let decl2 = rgb(config.colors.shirt2);
        if let Some(pos) = shirt.iter().take(4).position(|c| dist(c.color, decl1) < close) {
            present1 += 1;
            if pos == 0 { first1 += 1; } else { misranked.push(format!("{} rank {pos} share {:.0}% vs top {:.0}%", dds.rsplit('\\').nth(2).unwrap_or(""), shirt[pos].share * 100.0, shirt[0].share * 100.0)); }
        }
        if dist(decl1, decl2) >= params.distinct {
            if let Some(pos) = shirt.iter().take(4).position(|c| dist(c.color, decl2) < close) {
                if pos > 0 { second_present += 1; second_shares.push(shirt[pos].share); }
            }
            if shorts.first().map_or(false, |c| dist(c.color, decl2) < close) { second_is_shorts += 1; }
        }
        hit1 += (d1 < close) as u32;
        hit2 += (d2 < close) as u32;
        hit_shorts += (dsh < close) as u32;
        sum1 += d1;
        sum2 += d2;
        if dump {
            let top: Vec<String> = shirt.iter().take(3).map(|c| format!("{}:{:.0}", hex(c.color), c.share * 100.0)).collect();
            let sh: Vec<String> = shorts.iter().take(2).map(|c| format!("{}:{:.0}", hex(c.color), c.share * 100.0)).collect();
            dumped.push_str(&format!("{dds}\t{}\t{}\t{}\t{}\t{}\n", hex(color1), hex(color2), from_shirt, top.join(" "), sh.join(" ")));
        }
        if verbose {
            let name = dds.rsplit('\\').nth(2).unwrap_or("");
            let file = dds.rsplit('\\').next().unwrap_or("");
            let top: Vec<String> = shirt.iter().take(3).map(|c| format!("{}:{:.0}%", hex(c.color), c.share * 100.0)).collect();
            let sh: Vec<String> = shorts.iter().take(2).map(|c| format!("{}:{:.0}%", hex(c.color), c.share * 100.0)).collect();
            println!(
                "{d1:5.0} {d2:5.0} {dsh:5.0} | decl {} {} {} | ours {} {}{} | shirt {} | shorts {} | {name} {file}",
                hex(rgb(config.colors.shirt1)), hex(rgb(config.colors.shirt2)), hex(rgb(config.colors.shorts)),
                hex(color1), hex(color2), if from_shirt { "*" } else { "" }, top.join(" "), sh.join(" ")
            );
        }
    }
    if dump { fs::write(r"C:\Data\4cc\Tools_Mine\4cc-studio\.tmp\kit_extracted.tsv", dumped).unwrap(); }
    second_shares.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("declared shirt1 present among top-4 shirt clusters: {present1}, ranked first: {first1}; misranked: {misranked:?}");
    println!("declared distinct shirt2 present as a lower shirt cluster: {second_present}, shares {:?}; equals shorts top: {second_is_shorts}",
        second_shares.iter().map(|s| format!("{:.0}", s * 100.0)).collect::<Vec<_>>());
    println!(
        "kits {n}: color1 within {close}: {hit1} ({:.0}%), color2: {hit2} ({:.0}%), shorts vs declared shorts: {hit_shorts} ({:.0}%); mean d1 {:.1} d2 {:.1}; color2 from shirt {second_from_shirt}",
        100.0 * hit1 as f32 / n as f32, 100.0 * hit2 as f32 / n as f32, 100.0 * hit_shorts as f32 / n as f32,
        sum1 / n as f32, sum2 / n as f32
    );
}
