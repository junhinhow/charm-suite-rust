//! Codificador Sixel da compatibilidade Go.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{DynamicImage, Rgba};
use std::fmt::Write as _;

use crate::{average_rect, luminance, resolve_size, Error, SIXEL_CELL_HEIGHT, SIXEL_CELL_WIDTH};

pub(super) fn render_sixel(
    image: &DynamicImage,
    width: i32,
    height: i32,
    background: Rgba<u8>,
) -> Result<String, Error> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::EmptyImage);
    }
    let (cols, rows) = resolve_size(image.width(), image.height(), width, height)?;
    let pixel_width = cols
        .checked_mul(SIXEL_CELL_WIDTH)
        .ok_or(Error::DimensionsTooLarge)?;
    let pixel_height = rows
        .checked_mul(SIXEL_CELL_HEIGHT)
        .ok_or(Error::DimensionsTooLarge)?;
    let pixel_count = pixel_width
        .checked_mul(pixel_height)
        .ok_or(Error::DimensionsTooLarge)?;
    if pixel_count > 16_777_216 {
        return Err(Error::DimensionsTooLarge);
    }
    let mut pixels = vec![0_u8; pixel_count];
    let mut used = [false; 256];
    for y in 0..pixel_height {
        for x in 0..pixel_width {
            let index = quantize_sixel(average_rect(
                image,
                x,
                y,
                pixel_width,
                pixel_height,
                background,
            ));
            pixels[y * pixel_width + x] = index;
            used[index as usize] = true;
        }
    }
    let payload = build_sixel_payload(pixel_width, pixel_height, &pixels, &used);
    Ok(format!(
        "\x1bPq{}\x1b\\",
        STANDARD.encode(payload.as_bytes())
    ))
}

fn quantize_sixel(color: Rgba<u8>) -> u8 {
    let [r, g, b, _] = color.0;
    let r = i32::from(r);
    let g = i32::from(g);
    let b = i32::from(b);
    let ri = (r * 5 + 127) / 255;
    let gi = (g * 5 + 127) / 255;
    let bi = (b * 5 + 127) / 255;
    let cube_index = ri * 36 + gi * 6 + bi;
    let gray_level = (luminance(color) * 39 + 127) / 255;
    let gray = gray_level * 255 / 39;
    let cube_distance = square(r - ri * 51) + square(g - gi * 51) + square(b - bi * 51);
    let gray_distance = square(r - gray) + square(g - gray) + square(b - gray);
    if gray_distance < cube_distance {
        (216 + gray_level) as u8
    } else {
        cube_index as u8
    }
}

fn square(value: i32) -> i32 {
    value * value
}

fn sixel_color(index: u8) -> [u8; 3] {
    if index < 216 {
        [index / 36 * 51, (index % 36) / 6 * 51, index % 6 * 51]
    } else {
        let gray = (u16::from(index) - 216) * 255 / 39;
        [gray as u8; 3]
    }
}

fn build_sixel_payload(width: usize, height: usize, pixels: &[u8], used: &[bool; 256]) -> String {
    let mut output = String::new();
    write!(output, "\"1;1;{width};{height}").expect("escrita em String não falha");
    for (index, is_used) in used.iter().copied().enumerate() {
        if is_used {
            let color = sixel_color(index as u8);
            write!(
                output,
                "#{index};2;{};{};{}",
                u32::from(color[0]) * 100 / 255,
                u32::from(color[1]) * 100 / 255,
                u32::from(color[2]) * 100 / 255
            )
            .expect("escrita em String não falha");
        }
    }

    let bands = height.div_ceil(6);
    for band in 0..bands {
        if band > 0 {
            output.push('-');
        }
        let mut present = [false; 256];
        let start = band * 6;
        let end = (start + 6).min(height);
        for y in start..end {
            for index in &pixels[y * width..(y + 1) * width] {
                present[*index as usize] = true;
            }
        }
        let mut first = true;
        for (index, is_present) in present.iter().copied().enumerate() {
            if !is_present {
                continue;
            }
            if !first {
                output.push('$');
            }
            first = false;
            write!(output, "#{index}").expect("escrita em String não falha");
            write_band(&mut output, width, height, band, pixels, index as u8);
        }
    }
    output.push('-');
    output
}

fn write_band(
    output: &mut String,
    width: usize,
    height: usize,
    band: usize,
    pixels: &[u8],
    index: u8,
) {
    let mut run_char = 0_u8;
    let mut run_length = 0_usize;
    let flush = |output: &mut String, run_char: u8, run_length: &mut usize| {
        if *run_length > 3 {
            write!(output, "!{}{}", *run_length, char::from(run_char))
                .expect("escrita em String não falha");
        } else {
            output.extend(std::iter::repeat_n(char::from(run_char), *run_length));
        }
        *run_length = 0;
    };
    for x in 0..width {
        let mut bits = 0_u8;
        for bit in 0..6 {
            let y = band * 6 + bit;
            if y < height && pixels[y * width + x] == index {
                bits |= 1 << bit;
            }
        }
        let current = b'?' + bits;
        if current == run_char {
            run_length += 1;
        } else {
            flush(output, run_char, &mut run_length);
            run_char = current;
            run_length = 1;
        }
    }
    flush(output, run_char, &mut run_length);
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn red_image_has_expected_encoded_dcs() {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, Rgba([255, 0, 0, 255])));
        let output = render_sixel(&image, 2, 2, Rgba([0, 0, 0, 255])).unwrap();
        assert!(output.starts_with("\x1bPq"));
        assert!(output.ends_with("\x1b\\"));
        let encoded = output
            .trim_start_matches("\x1bPq")
            .trim_end_matches("\x1b\\");
        let data = STANDARD.decode(encoded).unwrap();
        let text = String::from_utf8(data).unwrap();
        assert!(text.starts_with("\"1;1;16;32"));
        assert!(text.contains("#180;2;100;0;0"));
        assert!(text.contains("!16"));
        assert!(text.ends_with('-'));
    }
}
