#[derive(Debug, Clone)]
pub struct ColorFormats {
    pub hex: String,
    pub hex_clean: String,
    pub color_name: String,
    pub rgb: String,
    pub hsl: String,
    pub hsv: String,
    pub cmyk: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub h_deg: u16,
    pub s_pct: u8,
    pub v_pct: u8,
    pub shades: Vec<(u8, u8, u8)>,
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

pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };

    let mut h = if delta == 0.0 {
        0.0
    } else if max == rf {
        (gf - bf) / delta + (if gf < bf { 6.0 } else { 0.0 })
    } else if max == gf {
        (bf - rf) / delta + 2.0
    } else {
        (rf - gf) / delta + 4.0
    };
    h *= 60.0;

    (h.round() as u16, (s * 100.0).round() as u8, (v * 100.0).round() as u8)
}

pub fn hsv_to_rgb(h: u16, s: u8, v: u8) -> (u8, u8, u8) {
    let hf = (h % 360) as f32;
    let sf = (s as f32) / 100.0;
    let vf = (v as f32) / 100.0;

    let c = vf * sf;
    let x = c * (1.0 - ((hf / 60.0) % 2.0 - 1.0).abs());
    let m = vf - c;

    let (rf, gf, bf) = match (hf / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((rf + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((gf + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((bf + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

pub fn hsl_to_rgb(h: u16, s: u8, l: u8) -> (u8, u8, u8) {
    let hf = (h % 360) as f32;
    let sf = (s as f32) / 100.0;
    let lf = (l as f32) / 100.0;

    let c = (1.0 - (2.0 * lf - 1.0).abs()) * sf;
    let x = c * (1.0 - ((hf / 60.0) % 2.0 - 1.0).abs());
    let m = lf - c / 2.0;

    let (rf, gf, bf) = match (hf / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((rf + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((gf + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((bf + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

pub fn generate_shades(r: u8, g: u8, b: u8) -> Vec<(u8, u8, u8)> {
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let l_int = l as i16;
    let steps = [
        (l_int + 24).clamp(10, 95) as u8, // Very light tint
        (l_int + 12).clamp(8, 92) as u8,  // Lighter tint
        l,                                // Exact Base
        (l_int - 12).clamp(5, 90) as u8,  // Darker shade
        (l_int - 24).clamp(5, 85) as u8,  // Very dark shade
    ];

    steps.iter().map(|&step_l| hsl_to_rgb(h, s, step_l)).collect()
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

pub fn get_color_name(r: u8, g: u8, b: u8) -> String {
    let max_c = r.max(g).max(b);
    let min_c = r.min(g).min(b);
    let delta = max_c - min_c;
    let l = (max_c as f32 + min_c as f32) / 510.0;

    // Handle near-neutral tones
    if delta < 15 {
        if l < 0.08 { return "Black".to_string(); }
        if l < 0.22 { return "Charcoal".to_string(); }
        if l < 0.45 { return "Dim grey".to_string(); }
        if l < 0.65 { return "Grey".to_string(); }
        if l < 0.85 { return "Silver".to_string(); }
        if l < 0.96 { return "Light grey".to_string(); }
        return "White".to_string();
    }

    // Curated color palette matching Windows PowerToys naming
    let palette: [(&str, u8, u8, u8); 53] = [
        ("Sky blue", 168, 199, 210),
        ("Sky blue", 135, 206, 235),
        ("Powder blue", 176, 224, 230),
        ("Light blue", 173, 216, 230),
        ("Steel blue", 70, 130, 180),
        ("Deep sky blue", 0, 191, 255),
        ("Dodger blue", 30, 144, 255),
        ("Royal blue", 65, 105, 225),
        ("Midnight blue", 25, 25, 112),
        ("Navy", 0, 0, 128),
        ("Blue", 0, 0, 255),
        ("Cyan", 0, 255, 255),
        ("Aqua", 0, 255, 255),
        ("Teal", 0, 128, 128),
        ("Turquoise", 64, 224, 208),
        ("Aquamarine", 127, 255, 212),
        ("Sage green", 156, 182, 135),
        ("Mint green", 152, 251, 152),
        ("Sea green", 46, 139, 87),
        ("Forest green", 34, 139, 34),
        ("Green", 0, 128, 0),
        ("Lime", 0, 255, 0),
        ("Lime green", 50, 205, 50),
        ("Olive", 128, 128, 0),
        ("Yellow", 255, 255, 0),
        ("Gold", 255, 215, 0),
        ("Khaki", 240, 230, 140),
        ("Orange", 255, 165, 0),
        ("Dark orange", 255, 140, 0),
        ("Coral", 255, 127, 80),
        ("Salmon", 250, 128, 114),
        ("Peach", 240, 174, 139),
        ("Peach", 255, 218, 185),
        ("Red", 255, 0, 0),
        ("Crimson", 220, 20, 60),
        ("Dark red", 139, 0, 0),
        ("Maroon", 128, 0, 0),
        ("Brown", 165, 42, 42),
        ("Chocolate", 210, 105, 30),
        ("Hot pink", 255, 105, 180),
        ("Deep pink", 255, 20, 147),
        ("Pink", 255, 192, 203),
        ("Light pink", 255, 182, 193),
        ("Magenta", 255, 0, 255),
        ("Violet", 238, 130, 238),
        ("Plum", 221, 160, 221),
        ("Purple", 128, 0, 128),
        ("Lavender", 181, 158, 230),
        ("Lavender", 230, 230, 250),
        ("Indigo", 75, 0, 130),
        ("White", 255, 255, 255),
        ("Black", 0, 0, 0),
        ("Charcoal", 54, 69, 79),
    ];

    let mut best_name = "Custom".to_string();
    let mut min_dist = u32::MAX;

    for (name, cr, cg, cb) in palette {
        let dr = (r as i32) - (cr as i32);
        let dg = (g as i32) - (cg as i32);
        let db = (b as i32) - (cb as i32);
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < min_dist {
            min_dist = dist;
            best_name = name.to_string();
        }
    }

    best_name
}

pub fn format_color(hex_str: &str) -> Option<ColorFormats> {
    let hex = extract_hex(hex_str)?;
    let (r, g, b) = hex_to_rgb(&hex)?;
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let (h_v, s_v, v) = rgb_to_hsv(r, g, b);
    let (c, m, y, k) = rgb_to_cmyk(r, g, b);
    let color_name = get_color_name(r, g, b);
    let hex_clean = hex.trim_start_matches('#').to_lowercase();
    let shades = generate_shades(r, g, b);

    Some(ColorFormats {
        hex: hex.clone(),
        hex_clean,
        color_name,
        rgb: format!("rgb({}, {}, {})", r, g, b),
        hsl: format!("hsl({}, {}%, {}%)", h, s, l),
        hsv: format!("hsv({}, {}%, {}%)", h_v, s_v, v),
        cmyk: format!("cmyk({}%, {}%, {}%, {}%)", c, m, y, k),
        r,
        g,
        b,
        h_deg: h_v,
        s_pct: s_v,
        v_pct: v,
        shades,
    })
}
