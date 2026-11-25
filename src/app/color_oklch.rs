//! OKLCH → sRGB for proposal-aligned clip paints (see revisão `ms.js`).

use eframe::egui::Color32 as C;

/// Camelot wheel hue in degrees (A/B share the number). Index 0 unused; 1..=12.
pub const KEY_HUE: [f32; 13] =
    [0.0, 190.0, 165.0, 142.0, 118.0, 85.0, 58.0, 32.0, 4.0, 340.0, 310.0, 278.0, 235.0];

pub fn camelot_index(key: &str) -> usize {
    let k = key.trim();
    let digits: String = k.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse::<usize>().unwrap_or(8).clamp(1, 12)
}

pub fn camelot_hue(key: &str) -> f32 {
    KEY_HUE[camelot_index(key)]
}

fn linear_to_srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let v = if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round() as u8
}

fn oklab_to_linear_rgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.291485548 * b;
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;
    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let bl = -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s;
    (r, g, bl)
}

/// Convert OKLCH (L 0..1, C chroma, H degrees) to opaque `Color32`.
pub fn oklch(l: f32, c: f32, h_deg: f32) -> C {
    oklch_alpha(l, c, h_deg, 1.0)
}

pub fn oklch_alpha(l: f32, c: f32, h_deg: f32, alpha: f32) -> C {
    let h = h_deg.to_radians();
    let a = c * h.cos();
    let b = c * h.sin();
    let (lr, lg, lb) = oklab_to_linear_rgb(l, a, b);
    C::from_rgba_unmultiplied(
        linear_to_srgb(lr),
        linear_to_srgb(lg),
        linear_to_srgb(lb),
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

pub fn clip_body_fill(key: &str) -> C {
    oklch(0.20, 0.018, camelot_hue(key))
}

pub fn clip_header_fill(key: &str) -> C {
    oklch(0.26, 0.03, camelot_hue(key))
}

pub fn key_badge_fill(key: &str) -> C {
    oklch_alpha(0.80, 0.12, camelot_hue(key), 0.2)
}

pub fn key_badge_text(key: &str) -> C {
    oklch(0.85, 0.12, camelot_hue(key))
}

pub fn wave_low() -> C {
    oklch(0.62, 0.16, 255.0)
}
pub fn wave_mid() -> C {
    oklch(0.80, 0.12, 175.0)
}
pub fn wave_hi() -> C {
    C::from_rgba_unmultiplied(236, 240, 246, 217)
}