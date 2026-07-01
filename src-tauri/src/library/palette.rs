use image::GenericImageView;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ThemeColors {
    pub vibrant: String,
    pub muted: String,
    pub dominant: String,
}

#[derive(Clone, Copy, Debug)]
struct RgbPixel {
    r: f32,
    g: f32,
    b: f32,
}

fn to_hex(r: f32, g: f32, b: f32) -> String {
    let r = (r.round() as u8).clamp(0, 255);
    let g = (g.round() as u8).clamp(0, 255);
    let b = (b.round() as u8).clamp(0, 255);
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn saturation(r: f32, g: f32, b: f32) -> f32 {
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let d = max - min;
    if d == 0.0 { 0.0 } else { d / (1.0 - (2.0 * luminance(r, g, b) - 1.0).abs()) }
}

fn value(r: f32, g: f32, b: f32) -> f32 {
    r.max(g.max(b))
}

fn distance_sq(a: &RgbPixel, b: &RgbPixel) -> f32 {
    let dr = a.r - b.r;
    let dg = a.g - b.g;
    let db = a.b - b.b;
    dr * dr + dg * dg + db * db
}

pub fn extract_palette(data: &[u8]) -> Option<ThemeColors> {
    let img = image::load_from_memory(data).ok()?;
    let thumb = img.thumbnail_exact(64, 64);
    let (w, h) = thumb.dimensions();

    let mut pixels: Vec<RgbPixel> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let p = thumb.get_pixel(x, y);
            if p[3] < 128 { continue; }
            pixels.push(RgbPixel { r: p[0] as f32, g: p[1] as f32, b: p[2] as f32 });
        }
    }

    if pixels.len() < 3 { return None; }

    const K: usize = 3;
    const ITERS: usize = 10;

    let mut centroids: [RgbPixel; K] = {
        let step = pixels.len() / K;
        [pixels[step * 0], pixels[step * 1], pixels[step * 2]]
    };

    let mut assignments = vec![0usize; pixels.len()];

    for _ in 0..ITERS {
        let mut changed = false;
        for (i, p) in pixels.iter().enumerate() {
            let mut best = 0;
            let mut best_d = distance_sq(p, &centroids[0]);
            for k in 1..K {
                let d = distance_sq(p, &centroids[k]);
                if d < best_d { best_d = d; best = k; }
            }
            if assignments[i] != best { assignments[i] = best; changed = true; }
        }
        if !changed { break; }
        for k in 0..K {
            let (mut sum_r, mut sum_g, mut sum_b, mut count) = (0.0f32, 0.0f32, 0.0f32, 0usize);
            for (i, p) in pixels.iter().enumerate() {
                if assignments[i] == k {
                    sum_r += p.r; sum_g += p.g; sum_b += p.b; count += 1;
                }
            }
            if count > 0 {
                centroids[k].r = sum_r / count as f32;
                centroids[k].g = sum_g / count as f32;
                centroids[k].b = sum_b / count as f32;
            }
        }
    }

    // Aggregate final clusters with metrics
    let mut clusters: Vec<(usize, f32, f32, f32, f32)> = Vec::new();
    for k in 0..K {
        let (mut sum_r, mut sum_g, mut sum_b, mut count) = (0.0f32, 0.0f32, 0.0f32, 0usize);
        for (i, p) in pixels.iter().enumerate() {
            if assignments[i] == k {
                sum_r += p.r; sum_g += p.g; sum_b += p.b; count += 1;
            }
        }
        if count > 0 {
            let ar = sum_r / count as f32;
            let ag = sum_g / count as f32;
            let ab = sum_b / count as f32;
            let s = saturation(ar / 255.0, ag / 255.0, ab / 255.0);
            let v = value(ar / 255.0, ag / 255.0, ab / 255.0);
            clusters.push((count, s * v, ar, ag, ab));
        }
    }

    if clusters.is_empty() { return None; }

    // Vibrant: highest saturation * value
    clusters.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let vibrant = to_hex(clusters[0].2, clusters[0].3, clusters[0].4);

    // Dominant: largest pixel count
    clusters.sort_by(|a, b| b.0.cmp(&a.0));
    let dominant = to_hex(clusters[0].2, clusters[0].3, clusters[0].4);

    // Muted: lowest saturation * value (or second largest if only 2 clusters)
    clusters.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let muted = if clusters.len() > 2 {
        to_hex(clusters[2].2, clusters[2].3, clusters[2].4)
    } else if clusters.len() > 1 {
        to_hex(clusters[1].2, clusters[1].3, clusters[1].4)
    } else {
        to_hex(clusters[0].2, clusters[0].3, clusters[0].4)
    };

    Some(ThemeColors { vibrant, muted, dominant })
}
