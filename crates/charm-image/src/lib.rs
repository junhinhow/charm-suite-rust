//! Render imagens em terminais usando Kitty, iTerm2, Sixel, truecolor ou ASCII.
//!
//! `charm-image` é um porte Rust de `reference/go/charm-image`. Ele trabalha
//! diretamente com [`image::DynamicImage`] e retorna sequências prontas para
//! impressão. Veja [`render`] e [`Options`] para a API principal.
//!
//! ```
//! use image::{DynamicImage, Rgba, RgbaImage};
//! use charm_image::{render, Options, Protocol};
//!
//! let mut pixels = RgbaImage::new(2, 2);
//! pixels.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
//! pixels.put_pixel(1, 0, Rgba([255, 0, 0, 255]));
//! pixels.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
//! pixels.put_pixel(1, 1, Rgba([0, 0, 255, 255]));
//! let image = DynamicImage::ImageRgba8(pixels);
//! let output = render(&image, Options::default().protocol(Protocol::HalfBlock))?;
//! assert!(output.contains('▀'));
//! # Ok::<(), charm_image::Error>(())
//! ```
//!
//! # Erros
//!
//! As funções retornam [`Error`] para imagens sem pixels, dimensões que não
//! podem ser alocadas, protocolos desconhecidos e falhas de codificação PNG.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod sixel;

use std::{fmt::Write as _, io::Cursor};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{DynamicImage, GenericImageView, Rgba};
use thiserror::Error as ThisError;

const DEFAULT_RAMP: &str = " .:-=+*#%@";
const DEFAULT_MAX_WIDTH: i32 = 80;
const CELL_ASPECT: i64 = 2;
const SIXEL_CELL_WIDTH: usize = 8;
const SIXEL_CELL_HEIGHT: usize = 16;
const KITTY_CHUNK_SIZE: usize = 4096;

/// Protocolo de imagem utilizado para renderizar.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Protocol {
    /// Seleciona um protocolo pelo ambiente do processo.
    #[default]
    Auto,
    /// Saída sem escape de controle, usando a rampa ASCII.
    None,
    /// Blocos superiores com cores truecolor ANSI.
    HalfBlock,
    /// Protocolo Sixel encapsulado em DCS.
    Sixel,
    /// Protocolo de imagem Kitty (APC).
    Kitty,
    /// Protocolo inline do iTerm2 (OSC 1337).
    ITerm2,
    /// Valor não suportado, usado para reportar enumerações desconhecidas.
    Unknown(i32),
}

impl Protocol {
    /// Nome canônico do protocolo.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::HalfBlock => "halfblock",
            Self::Sixel => "sixel",
            Self::Kitty => "kitty",
            Self::ITerm2 => "iterm2",
            Self::Unknown(_) => "unknown",
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Erros produzidos durante a renderização.
#[derive(Debug, ThisError)]
pub enum Error {
    /// A imagem não possui pixels.
    #[error("charm-image: imagem sem pixels")]
    EmptyImage,
    /// Protocolo não reconhecido.
    #[error("charm-image: protocolo desconhecido: {0}")]
    UnknownProtocol(i32),
    /// Dimensões grandes demais para processamento/alocação.
    #[error("charm-image: dimensões de saída excedem o limite suportado")]
    DimensionsTooLarge,
    /// Erro ao codificar os pixels como PNG para protocolos de imagem.
    #[error("charm-image: falha ao codificar imagem em PNG: {0}")]
    Png(#[from] image::ImageError),
}

/// Configuração de uma chamada a [`render`].
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Largura em células. Zero ou negativo usa a largura natural, limitada a 80.
    pub width: i32,
    /// Altura em células. Zero ou negativo deriva a altura da proporção.
    pub height: i32,
    /// Protocolo ou detecção automática.
    pub protocol: Protocol,
    /// Aplica dithering Bayer 4×4 ao fallback ASCII.
    pub dither: bool,
    /// Fundo para compor pixels transparentes; `None` significa preto.
    pub background: Option<Rgba<u8>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            protocol: Protocol::Auto,
            dither: false,
            background: None,
        }
    }
}

impl Options {
    /// Define a largura em células.
    #[must_use]
    pub const fn width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }

    /// Define a altura em células.
    #[must_use]
    pub const fn height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }

    /// Define o protocolo.
    #[must_use]
    pub const fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Ativa ou desativa o dithering ordenado no fallback ASCII.
    #[must_use]
    pub const fn dither(mut self, dither: bool) -> Self {
        self.dither = dither;
        self
    }

    /// Define a cor opaca sobre a qual pixels transparentes serão compostos.
    #[must_use]
    pub const fn background(mut self, background: Rgba<u8>) -> Self {
        self.background = Some(background);
        self
    }
}

/// Detecta o protocolo adequado a partir do ambiente do processo.
#[must_use]
pub fn detect() -> Protocol {
    detect_from(|key| std::env::var(key).unwrap_or_default())
}

/// Detecta o protocolo usando uma função de consulta de ambiente fornecida.
///
/// `TERM=dumb` ou `NO_COLOR` têm precedência, seguido por Kitty, iTerm2 e Sixel.
/// Sem indicação explícita, usa half-block truecolor.
#[must_use]
pub fn detect_from<F>(mut getenv: F) -> Protocol
where
    F: FnMut(&str) -> String,
{
    let term = getenv("TERM");
    let no_color = getenv("NO_COLOR");
    if term.trim().eq_ignore_ascii_case("dumb") || !no_color.trim().is_empty() {
        return Protocol::None;
    }
    if !getenv("KITTY_WINDOW_ID").trim().is_empty() {
        return Protocol::Kitty;
    }
    match getenv("TERM_PROGRAM").trim().to_ascii_lowercase().as_str() {
        "iterm.app" | "wezterm" | "mintty" => return Protocol::ITerm2,
        _ => {}
    }
    if term.to_ascii_lowercase().contains("sixel")
        || getenv("COLORTERM").to_ascii_lowercase().contains("sixel")
    {
        return Protocol::Sixel;
    }
    Protocol::HalfBlock
}

/// Renderiza uma imagem com o protocolo configurado ou detectado.
pub fn render(image: &DynamicImage, options: Options) -> Result<String, Error> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::EmptyImage);
    }
    let protocol = match options.protocol {
        Protocol::Auto => detect(),
        protocol => protocol,
    };
    let background = opaque_background(options.background.unwrap_or(Rgba([0, 0, 0, 255])));
    match protocol {
        Protocol::None => Ok(ascii_art_with_background(
            image,
            options.width,
            options.height,
            DEFAULT_RAMP,
            options.dither,
            background,
        )?),
        Protocol::HalfBlock => {
            render_half_block_with_background(image, options.width, options.height, background)
        }
        Protocol::Sixel => {
            render_sixel_with_background(image, options.width, options.height, background)
                .map(|output| format!("\x1b[1;1H{output}"))
        }
        Protocol::Kitty => render_kitty(image, options.width, options.height),
        Protocol::ITerm2 => render_iterm2(image, options.width, options.height),
        Protocol::Auto => unreachable!("Auto é resolvido antes do despacho"),
        Protocol::Unknown(value) => Err(Error::UnknownProtocol(value)),
    }
}

/// Desenha a imagem usando uma rampa monocromática configurável.
///
/// Rampa vazia seleciona a rampa padrão; a saída é vazia para imagens sem pixels.
#[must_use]
pub fn ascii_art(image: &DynamicImage, width: i32, height: i32, ramp: &str) -> String {
    if image.width() == 0 || image.height() == 0 {
        return String::new();
    }
    ascii_art_with_background(image, width, height, ramp, false, Rgba([0, 0, 0, 255]))
        .unwrap_or_default()
}

fn ascii_art_with_background(
    image: &DynamicImage,
    width: i32,
    height: i32,
    ramp: &str,
    dither: bool,
    background: Rgba<u8>,
) -> Result<String, Error> {
    if image.width() == 0 || image.height() == 0 {
        return Ok(String::new());
    }
    let (cols, rows) = resolve_size(image.width(), image.height(), width, height)?;
    let ramp: Vec<char> = if ramp.is_empty() {
        DEFAULT_RAMP.chars().collect()
    } else {
        ramp.chars().collect()
    };
    let last = ramp.len().saturating_sub(1);
    let mut output = String::new();
    for row in 0..rows {
        if row > 0 {
            output.push('\n');
        }
        for col in 0..cols {
            let color = average_rect(image, col, row, cols, rows, background);
            let mut luminance = luminance(color);
            if dither && last > 0 {
                const BAYER: [i32; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
                let step = 255 / i32::try_from(last).unwrap_or(i32::MAX);
                let offset = (BAYER[(row % 4) * 4 + col % 4] - 8) * step / 8;
                luminance = (luminance + offset).clamp(0, 255);
            }
            let index = if last == 0 {
                0
            } else {
                (luminance as usize * last / 255).min(last)
            };
            output.push(ramp[index]);
        }
    }
    Ok(output)
}

/// Renderiza usando `▀` e sequências ANSI truecolor.
pub fn render_half_block(image: &DynamicImage, width: i32, height: i32) -> Result<String, Error> {
    render_half_block_with_background(image, width, height, Rgba([0, 0, 0, 255]))
}

fn render_half_block_with_background(
    image: &DynamicImage,
    width: i32,
    height: i32,
    background: Rgba<u8>,
) -> Result<String, Error> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::EmptyImage);
    }
    let (cols, cell_rows) = resolve_size(image.width(), image.height(), width, height)?;
    let pixel_rows = cell_rows.checked_mul(2).ok_or(Error::DimensionsTooLarge)?;
    let mut output = String::new();
    for row in 0..cell_rows {
        if row > 0 {
            output.push('\n');
        }
        for col in 0..cols {
            let top = average_rect(image, col, row * 2, cols, pixel_rows, background);
            let bottom = average_rect(image, col, row * 2 + 1, cols, pixel_rows, background);
            write!(
                output,
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                top[0], top[1], top[2], bottom[0], bottom[1], bottom[2]
            )
            .expect("escrita em String não falha");
        }
        output.push_str("\x1b[0m");
    }
    Ok(output)
}

/// Codifica e renderiza uma imagem no protocolo iTerm2 inline.
pub fn render_iterm2(image: &DynamicImage, width: i32, height: i32) -> Result<String, Error> {
    validate_image(image)?;
    let (cols, rows) = resolve_size(image.width(), image.height(), width, height)?;
    let png = encode_png(image)?;
    Ok(format!(
        "\x1b]1337;File=inline=1;width={cols};height={rows}:{}\x07",
        STANDARD.encode(png)
    ))
}

/// Codifica e renderiza uma imagem no protocolo Kitty, usando chunks de até 4096 bytes.
pub fn render_kitty(image: &DynamicImage, width: i32, height: i32) -> Result<String, Error> {
    validate_image(image)?;
    let (cols, rows) = resolve_size(image.width(), image.height(), width, height)?;
    let payload = STANDARD.encode(encode_png(image)?);
    let mut output = String::new();
    if payload.len() <= KITTY_CHUNK_SIZE {
        write!(
            output,
            "\x1b_Gf=100,a=T,c={cols},r={rows},q=2;{payload}\x1b\\"
        )
        .expect("escrita em String não falha");
        return Ok(output);
    }
    for (index, chunk) in payload.as_bytes().chunks(KITTY_CHUNK_SIZE).enumerate() {
        let more = usize::from((index + 1) * KITTY_CHUNK_SIZE < payload.len());
        let chunk = std::str::from_utf8(chunk).expect("base64 é UTF-8 ASCII");
        if index == 0 {
            write!(
                output,
                "\x1b_Gf=100,a=T,c={cols},r={rows},q=2,m={more};{chunk}\x1b\\"
            )
            .expect("escrita em String não falha");
        } else {
            write!(output, "\x1b_Gm={more};{chunk}\x1b\\").expect("escrita em String não falha");
        }
    }
    Ok(output)
}

fn render_sixel_with_background(
    image: &DynamicImage,
    width: i32,
    height: i32,
    background: Rgba<u8>,
) -> Result<String, Error> {
    sixel::render_sixel(image, width, height, background)
}

/// Renderiza Sixel na forma DCS bruta contratada pelo porte (payload em base64).
pub fn render_sixel(image: &DynamicImage, width: i32, height: i32) -> Result<String, Error> {
    render_sixel_with_background(image, width, height, Rgba([0, 0, 0, 255]))
}

fn validate_image(image: &DynamicImage) -> Result<(), Error> {
    if image.width() == 0 || image.height() == 0 {
        Err(Error::EmptyImage)
    } else {
        Ok(())
    }
}

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, Error> {
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageFormat::Png)?;
    Ok(buffer.into_inner())
}

fn resolve_size(
    src_width: u32,
    src_height: u32,
    width: i32,
    height: i32,
) -> Result<(usize, usize), Error> {
    let cols = if width <= 0 {
        src_width.min(DEFAULT_MAX_WIDTH as u32)
    } else {
        width as u32
    }
    .max(1) as usize;
    let rows = if height <= 0 {
        let numerator = (cols as u128) * u128::from(src_height);
        let denominator = u128::from(src_width) * CELL_ASPECT as u128;
        ((numerator + denominator / 2) / denominator).max(1)
    } else {
        height as u128
    };
    let rows = usize::try_from(rows).map_err(|_| Error::DimensionsTooLarge)?;
    if cols.checked_mul(rows).is_none()
        || cols.saturating_mul(rows) > 16_777_216
        || cols > 16_384
        || rows > 16_384
    {
        return Err(Error::DimensionsTooLarge);
    }
    Ok((cols, rows))
}

fn average_rect(
    image: &DynamicImage,
    col: usize,
    row: usize,
    cols: usize,
    rows: usize,
    background: Rgba<u8>,
) -> Rgba<u8> {
    let src_width = image.width() as usize;
    let src_height = image.height() as usize;
    let x0 = col * src_width / cols;
    let x1 = ((col + 1) * src_width / cols).max(x0 + 1).min(src_width);
    let y0 = row * src_height / rows;
    let y1 = ((row + 1) * src_height / rows).max(y0 + 1).min(src_height);
    let mut sums = [0_u64; 3];
    let mut count = 0_u64;
    for y in y0..y1 {
        for x in x0..x1 {
            let pixel = image.get_pixel(x as u32, y as u32).0;
            for channel in 0..3 {
                sums[channel] +=
                    u64::from(composite(pixel[channel], background[channel], pixel[3]));
            }
            count += 1;
        }
    }
    Rgba([
        (sums[0] / count) as u8,
        (sums[1] / count) as u8,
        (sums[2] / count) as u8,
        255,
    ])
}

fn composite(foreground: u8, background: u8, alpha: u8) -> u8 {
    ((u32::from(foreground) * u32::from(alpha)
        + u32::from(background) * u32::from(255 - alpha)
        + 127)
        / 255) as u8
}

fn opaque_background(color: Rgba<u8>) -> Rgba<u8> {
    Rgba([
        composite(color[0], 0, color[3]),
        composite(color[1], 0, color[3]),
        composite(color[2], 0, color[3]),
        255,
    ])
}

fn luminance(color: Rgba<u8>) -> i32 {
    (299 * i32::from(color[0]) + 587 * i32::from(color[1]) + 114 * i32::from(color[2]) + 500) / 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    fn solid(width: u32, height: u32, color: [u8; 4]) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(width, height, Rgba(color)))
    }

    #[test]
    fn ascii_art_maps_black_and_white() {
        let image = DynamicImage::ImageRgba8(
            RgbaImage::from_raw(2, 1, vec![0, 0, 0, 255, 255, 255, 255, 255]).unwrap(),
        );
        assert_eq!(ascii_art(&image, 2, 1, "#."), "#.");
        assert_eq!(ascii_art(&image, 2, 1, ""), " @");
    }

    #[test]
    fn half_block_has_exact_truecolor_sequence() {
        let mut pixels = RgbaImage::new(2, 2);
        pixels.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        pixels.put_pixel(1, 0, Rgba([255, 0, 0, 255]));
        pixels.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        pixels.put_pixel(1, 1, Rgba([0, 0, 255, 255]));
        assert_eq!(
            render_half_block(&DynamicImage::ImageRgba8(pixels), 1, 1).unwrap(),
            "\x1b[38;2;255;0;0m\x1b[48;2;0;0;255m▀\x1b[0m"
        );
    }

    #[test]
    fn detection_priorities_follow_reference() {
        let env = [
            ("NO_COLOR", ""),
            ("KITTY_WINDOW_ID", "1"),
            ("TERM_PROGRAM", "iTerm.app"),
        ];
        assert_eq!(
            detect_from(|key| env
                .iter()
                .find(|(name, _)| *name == key)
                .map_or_else(String::new, |(_, value)| (*value).to_owned())),
            Protocol::Kitty
        );
        assert_eq!(
            detect_from(|key| if key == "NO_COLOR" {
                "1".to_owned()
            } else {
                String::new()
            }),
            Protocol::None
        );
    }

    #[test]
    fn unknown_protocol_and_empty_image_are_errors() {
        assert!(matches!(
            render(
                &solid(1, 1, [0, 0, 0, 255]),
                Options::default().protocol(Protocol::Unknown(42))
            ),
            Err(Error::UnknownProtocol(42))
        ));
        assert!(matches!(
            render_half_block(&solid(0, 0, [0, 0, 0, 0]), 1, 1),
            Err(Error::EmptyImage)
        ));
    }

    #[test]
    fn kitty_and_iterm2_emit_protocol_envelopes() {
        let image = solid(1, 1, [10, 20, 30, 255]);
        let kitty = render_kitty(&image, 1, 1).unwrap();
        let iterm = render_iterm2(&image, 1, 1).unwrap();
        assert!(kitty.starts_with("\x1b_Gf=100,a=T,c=1,r=1,q=2;"));
        assert!(iterm.starts_with("\x1b]1337;File=inline=1;width=1;height=1:"));
        let kitty_png = kitty
            .trim_start_matches("\x1b_Gf=100,a=T,c=1,r=1,q=2;")
            .trim_end_matches("\x1b\\");
        let iterm_png = iterm
            .trim_start_matches("\x1b]1337;File=inline=1;width=1;height=1:")
            .trim_end_matches('\x07');
        for encoded in [kitty_png, iterm_png] {
            let decoded = image::load_from_memory(&STANDARD.decode(encoded).unwrap()).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (1, 1));
        }
    }

    #[test]
    fn kitty_chunks_large_png_as_reference() {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_fn(96, 96, |x, y| {
            Rgba([
                ((x * 73 + y * 41) % 256) as u8,
                ((x * 29 + y * 151) % 256) as u8,
                ((x * 197 + y * 17) % 256) as u8,
                255,
            ])
        }));
        let output = render_kitty(&image, 48, 48).unwrap();
        assert!(output.matches("\x1b_G").count() > 1);
        assert!(output.contains("m=1;"));
        let payload = output
            .split("\x1b_G")
            .filter(|part| !part.is_empty())
            .map(|part| part.trim_end_matches("\x1b\\").split_once(';').unwrap().1)
            .collect::<String>();
        let decoded = image::load_from_memory(&STANDARD.decode(payload).unwrap()).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (96, 96));
    }

    #[test]
    fn transparent_pixels_compose_over_configured_background() {
        let image = solid(1, 1, [255, 0, 0, 0]);
        let output = render(
            &image,
            Options::default()
                .protocol(Protocol::HalfBlock)
                .width(1)
                .height(1)
                .background(Rgba([0, 0, 255, 255])),
        )
        .unwrap();
        assert_eq!(output, "\x1b[38;2;0;0;255m\x1b[48;2;0;0;255m▀\x1b[0m");
    }

    #[test]
    fn render_positions_cursor_before_sixel_but_raw_api_does_not() {
        let image = solid(1, 1, [255, 0, 0, 255]);
        let raw = render_sixel(&image, 1, 1).unwrap();
        let rendered = render(
            &image,
            Options::default()
                .protocol(Protocol::Sixel)
                .width(1)
                .height(1),
        )
        .unwrap();
        assert!(raw.starts_with("\x1bPq"));
        assert!(rendered.starts_with("\x1b[1;1H\x1bPq"));
    }

    #[test]
    fn ordered_dither_changes_ascii_render() {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_fn(16, 8, |x, _| {
            let v = ((x * 16) as u8).saturating_add(96);
            Rgba([v, v, v, 255])
        }));
        let plain = render(
            &image,
            Options::default()
                .protocol(Protocol::None)
                .width(16)
                .height(8),
        )
        .unwrap();
        let dithered = render(
            &image,
            Options::default()
                .protocol(Protocol::None)
                .width(16)
                .height(8)
                .dither(true),
        )
        .unwrap();
        assert_ne!(plain, dithered);
    }

    #[test]
    fn ascii_golden_output_matches_vector() {
        let image = DynamicImage::ImageRgba8(
            RgbaImage::from_raw(
                4,
                2,
                vec![
                    0, 0, 0, 255, 64, 64, 64, 255, 128, 128, 128, 255, 255, 255, 255, 255, 255, 0,
                    0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
                ],
            )
            .unwrap(),
        );
        let output = ascii_art(&image, 4, 2, " .:-=+*#%@");
        charm_parity::assert_golden("image_ascii", &format!("{output}\n"));
    }
}
