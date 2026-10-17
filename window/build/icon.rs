//! The app icon, drawn in code: the Lucide "download" sign in white on a
//! dark rounded square, as in the window's title bar. The build script
//! turns it into the `.ico` embedded in keepclip.exe, which Windows shows in
//! File Explorer, on the taskbar and in the notifications.

/// Sizes Windows picks from, from small icons (16 px at 100% scaling) up to
/// File Explorer's extra large icons (256 px).
const SIZES: [u32; 10] = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256];

/// The square's color, the near-black of the window's primary buttons.
const BACKGROUND: [f64; 3] = [26.0, 27.0, 32.0];

/// The "download" icon of Lucide (ISC, see ui/icons/LICENSE-LUCIDE) on its
/// 24 × 24 grid: the tray, the arrow head and the shaft. Each arc of the
/// tray is a quarter circle of radius 2.
fn strokes() -> Vec<Vec<(f64, f64)>> {
    let arc = |cx: f64, cy: f64, from: f64, to: f64| -> Vec<(f64, f64)> {
        (0..=8)
            .map(|i| {
                let angle = (from + (to - from) * f64::from(i) / 8.0).to_radians();
                (cx + 2.0 * angle.cos(), cy + 2.0 * angle.sin())
            })
            .collect()
    };
    // M21 15 v4 a2 2 0 0 1 -2 2 H5 a2 2 0 0 1 -2 -2 v-4
    let mut tray = vec![(21.0, 15.0)];
    tray.extend(arc(19.0, 19.0, 0.0, 90.0));
    tray.extend(arc(5.0, 19.0, 90.0, 180.0));
    tray.push((3.0, 15.0));
    vec![
        tray,
        vec![(7.0, 10.0), (12.0, 15.0), (17.0, 10.0)],
        vec![(12.0, 15.0), (12.0, 3.0)],
    ]
}

/// RGBA pixels, top to bottom, with straight (not premultiplied) alpha.
pub fn app(size: u32) -> Vec<u8> {
    let s = f64::from(size);
    // A small margin, as Windows' own icons have; none at the smallest
    // sizes, where every pixel counts.
    let margin = if size <= 24 { 0.0 } else { s / 16.0 };
    let half = s / 2.0 - margin;
    let radius = half * 0.44;
    // The 24-unit grid of the sign over most of the square, a little more
    // at the smallest sizes so its lines stay visible.
    let grid = if size <= 24 { 0.72 } else { 0.62 } * 2.0 * half;
    let unit = grid / 24.0;
    let half_stroke = (unit * 2.0).max(1.3) / 2.0;
    let origin = s / 2.0 - grid / 2.0;
    let lines: Vec<Vec<(f64, f64)>> = strokes()
        .into_iter()
        .map(|line| {
            line.into_iter()
                .map(|(x, y)| (origin + x * unit, origin + y * unit))
                .collect()
        })
        .collect();

    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let square = coverage(rounded_square(px - s / 2.0, py - s / 2.0, half, radius));
            let distance = lines
                .iter()
                .flat_map(|line| line.windows(2))
                .map(|pair| to_segment((px, py), pair[0], pair[1]))
                .fold(f64::INFINITY, f64::min);
            let sign = coverage(distance - half_stroke).min(square);
            let channel = |c: f64| (c + (255.0 - c) * sign).round() as u8;
            out.extend_from_slice(&[
                channel(BACKGROUND[0]),
                channel(BACKGROUND[1]),
                channel(BACKGROUND[2]),
                (square * 255.0).round() as u8,
            ]);
        }
    }
    out
}

/// How much of a pixel a shape covers, from its signed distance to the
/// pixel's center: about one pixel of soft edge.
fn coverage(distance: f64) -> f64 {
    (0.5 - distance).clamp(0.0, 1.0)
}

/// Signed distance from (x, y), relative to the center, to a square of
/// half side `half` with corners of `radius`.
fn rounded_square(x: f64, y: f64, half: f64, radius: f64) -> f64 {
    let qx = x.abs() - (half - radius);
    let qy = y.abs() - (half - radius);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    outside + qx.max(qy).min(0.0) - radius
}

fn to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length).clamp(0.0, 1.0)
    };
    (p.0 - (a.0 + t * dx)).hypot(p.1 - (a.1 + t * dy))
}

/// The `.ico` file, one PNG image per size. `zlib` compresses the rows of
/// each image.
pub fn ico(zlib: impl Fn(&[u8]) -> Vec<u8>) -> Vec<u8> {
    let images: Vec<(u32, Vec<u8>)> = SIZES
        .iter()
        .map(|&size| (size, png(size, &app(size), &zlib)))
        .collect();

    // ICONDIR: reserved, type (1 = icon), number of images.
    let mut ico = Vec::new();
    for field in [0u16, 1, images.len() as u16] {
        ico.extend_from_slice(&field.to_le_bytes());
    }
    // One ICONDIRENTRY per image, then the images themselves.
    let mut offset = 6 + 16 * images.len();
    for (size, png) in &images {
        // A width and height of 0 mean 256.
        let side = if *size >= 256 { 0 } else { *size as u8 };
        // Width, height, palette size, reserved, color planes, bits per pixel.
        ico.extend_from_slice(&[side, side, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
        ico.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += png.len();
    }
    for (_, png) in images {
        ico.extend_from_slice(&png);
    }
    ico
}

/// A square RGBA image as a PNG file.
fn png(size: u32, rgba: &[u8], zlib: impl Fn(&[u8]) -> Vec<u8>) -> Vec<u8> {
    let row = size as usize * 4;
    let mut raw = Vec::with_capacity((row + 1) * size as usize);
    for line in rgba.chunks(row) {
        // Filter type 0: the row as it is.
        raw.push(0);
        raw.extend_from_slice(line);
    }
    let mut header = Vec::new();
    header.extend_from_slice(&size.to_be_bytes());
    header.extend_from_slice(&size.to_be_bytes());
    // 8 bits per channel, RGBA, deflate, standard filters, no interlace.
    header.extend_from_slice(&[8, 6, 0, 0, 0]);

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &header);
    chunk(&mut png, b"IDAT", &zlib(&raw));
    chunk(&mut png, b"IEND", &[]);
    png
}

fn chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = png.len();
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let crc = crc32(&png[start..]);
    png.extend_from_slice(&crc.to_be_bytes());
}

/// The CRC-32 of PNG chunks (the one of zip and Ethernet).
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
