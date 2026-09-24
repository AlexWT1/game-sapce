//! Local placeholder artwork generation (no copyrighted assets).

fn hash_hue(input: &str) -> u32 {
    let mut h: u32 = 5381;
    for b in input.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    h % 360
}

fn hsl_to_hex(h: u32, s: u32, l: u32) -> String {
    let s = s as f64 / 100.0;
    let l = l as f64 / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h as f64) / 60.0;
    let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let to_hex = |v: f64| -> String {
        let n = ((v + m).clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("{n:02x}")
    };
    format!("#{}{}{}", to_hex(r1), to_hex(g1), to_hex(b1))
}

/// Generates an SVG data-URI cover for a game title.
pub fn placeholder_cover(title: &str) -> String {
    let hue = hash_hue(title);
    let c1 = hsl_to_hex(hue, 62, 22);
    let c2 = hsl_to_hex((hue + 40) % 360, 70, 38);
    let accent = hsl_to_hex((hue + 200) % 360, 75, 62);
    let initials: String = title
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(3)
        .collect::<String>()
        .to_uppercase();
    let short: String = title.chars().take(28).collect();
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="840" viewBox="0 0 600 840">
<defs>
  <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="{c1}"/>
    <stop offset="100%" stop-color="{c2}"/>
  </linearGradient>
  <radialGradient id="glow" cx="0.7" cy="0.25" r="0.8">
    <stop offset="0%" stop-color="{accent}" stop-opacity="0.55"/>
    <stop offset="100%" stop-color="{accent}" stop-opacity="0"/>
  </radialGradient>
</defs>
<rect width="600" height="840" fill="{c1}"/>
<rect width="600" height="840" fill="url(#g)" opacity="0.85"/>
<rect width="600" height="840" fill="url(#glow)"/>
<g opacity="0.18" fill="none" stroke="#ffffff" stroke-width="2">
  <path d="M0 640 L600 480"/>
  <path d="M0 700 L600 540"/>
  <path d="M0 760 L600 600"/>
  <circle cx="470" cy="180" r="120"/>
  <circle cx="470" cy="180" r="170"/>
</g>
<text x="50%" y="46%" text-anchor="middle" font-family="Inter, Arial, sans-serif" font-size="150" font-weight="700" fill="#ffffff" opacity="0.92">{initials}</text>
<text x="50%" y="56%" text-anchor="middle" font-family="Inter, Arial, sans-serif" font-size="30" font-weight="500" fill="#ffffff" opacity="0.75">{short}</text>
<rect x="0" y="780" width="600" height="60" fill="#05060f" opacity="0.55"/>
<text x="50%" y="818" text-anchor="middle" font-family="Inter, Arial, sans-serif" font-size="22" fill="#ffffff" opacity="0.55">GAME SPACE</text>
</svg>"##
    );
    // Escape for data URI
    let encoded: String = svg
        .chars()
        .map(|c| match c {
            '#' => "%23".to_string(),
            '"' => "%22".to_string(),
            '<' => "%3C".to_string(),
            '>' => "%3E".to_string(),
            ' ' => "%20".to_string(),
            '\n' => String::new(),
            c if c.is_ascii() => c.to_string(),
            c => format!("%{:04X}", c as u32),
        })
        .collect();
    format!("data:image/svg+xml,{encoded}")
}
