#[derive(Debug, Clone)]
pub struct ColorFormats {
    pub hex: String,
    pub rgb: String,
    pub hsl: String,
    pub cmyk: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn extract_hex(raw: &str) -> Option<String> {
    let s = raw.trim();
    // 1. Look for '#' followed by 6 hex characters
    if let Some(idx) = s.find('#') {
        let candidate = &s[idx + 1..];
        if candidate.len() >= 6 && candidate[..6].chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(format!("#{}", &candidate[..6].to_uppercase()));
        }
    }
    // 2. Scan for 6 consecutive ascii hex characters
    let chars: Vec<char> = s.chars().collect();
    for window in chars.windows(6) {
        if window.iter().all(|c| c.is_ascii_hexdigit()) {
            let hex_str: String = window.iter().collect();
            return Some(format!("#{}", hex_str.to_uppercase()));
        }
    }
    None
}

pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0, 0, (l * 100.0).round() as u8);
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let mut h = if max == rf {
        (gf - bf) / delta + (if gf < bf { 6.0 } else { 0.0 })
    } else if max == gf {
        (bf - rf) / delta + 2.0
    } else {
        (rf - gf) / delta + 4.0
    };
    h *= 60.0;

    (h.round() as u16, (s * 100.0).round() as u8, (l * 100.0).round() as u8)
}

pub fn rgb_to_cmyk(r: u8, g: u8, b: u8) -> (u8, u8, u8, u8) {
    if r == 0 && g == 0 && b == 0 {
        return (0, 0, 0, 100);
    }
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let k = 1.0 - rf.max(gf).max(bf);
    let c = (1.0 - rf - k) / (1.0 - k);
    let m = (1.0 - gf - k) / (1.0 - k);
    let y = (1.0 - bf - k) / (1.0 - k);

    (
        (c * 100.0).round() as u8,
        (m * 100.0).round() as u8,
        (y * 100.0).round() as u8,
        (k * 100.0).round() as u8,
    )
}

pub fn format_color(hex_str: &str) -> Option<ColorFormats> {
    let hex = extract_hex(hex_str)?;
    let (r, g, b) = hex_to_rgb(&hex)?;
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let (c, m, y, k) = rgb_to_cmyk(r, g, b);

    Some(ColorFormats {
        hex: hex.clone(),
        rgb: format!("rgb({}, {}, {})", r, g, b),
        hsl: format!("hsl({}, {}%, {}%)", h, s, l),
        cmyk: format!("cmyk({}%, {}%, {}%, {}%)", c, m, y, k),
        r,
        g,
        b,
    })
}
