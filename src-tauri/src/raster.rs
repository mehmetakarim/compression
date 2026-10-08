//! JPG/JPEG, PNG ve WebP sıkıştırma.
//!
//! - JPEG: EXIF yönü uygulanır, gerekirse küçültülür, renk profili (ICC)
//!   korunarak yeniden kodlanır. EXIF meta verisi (kamera, konum) atılır.
//! - PNG: Hafif/Dengeli kayıpsızdır (oxipng). Güçlü seviyede renkler 256'ya
//!   indirilir (NeuQuant) ve ardından kayıpsız optimize edilir.
//! - WebP: Kayıplı WebP olarak (libwebp) yeniden kodlanır.
//!
//! Animasyonlu PNG/WebP dosyaları kare kaybı olmaması için reddedilir.

use std::fs;
use std::io::Cursor;
use std::path::Path;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageEncoder, ImageReader, RgbaImage};

use crate::common::{write_if_smaller, Level, Outcome, Progress, Stage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Jpeg,
    Png,
    Webp,
}

impl Kind {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Kind::Jpeg),
            "png" => Some(Kind::Png),
            "webp" => Some(Kind::Webp),
            _ => None,
        }
    }
}

struct Settings {
    /// Uzun kenar üst sınırı; `None` ise boyut korunur.
    max_side: Option<u32>,
    /// JPEG/WebP kalitesi (0–100).
    quality: u8,
}

fn settings(level: Level) -> Settings {
    match level {
        Level::Light => Settings { max_side: None, quality: 85 },
        Level::Balanced => Settings { max_side: Some(3000), quality: 75 },
        Level::Strong => Settings { max_side: Some(2000), quality: 60 },
    }
}

pub fn compress_file(
    input: &Path,
    kind: Kind,
    level: Level,
    report: &mut dyn FnMut(Progress),
) -> Result<Outcome, String> {
    report(Progress::new(0, Stage::Reading));
    let original = fs::read(input).map_err(|e| format!("Dosya okunamadı: {e}"))?;
    let bytes = compress_bytes(&original, kind, level, report)?;
    let outcome = write_if_smaller(input, original.len() as u64, &bytes)?;
    report(Progress::new(100, Stage::Saving));
    Ok(outcome)
}

pub fn compress_bytes(
    original: &[u8],
    kind: Kind,
    level: Level,
    report: &mut dyn FnMut(Progress),
) -> Result<Vec<u8>, String> {
    let settings = settings(level);
    match kind {
        Kind::Png if is_animated_png(original) => {
            Err("Animasyonlu PNG dosyaları henüz desteklenmiyor.".into())
        }
        Kind::Webp if is_animated_webp(original) => {
            Err("Animasyonlu WebP dosyaları henüz desteklenmiyor.".into())
        }
        Kind::Png if level != Level::Strong => {
            report(Progress::new(10, Stage::Optimizing));
            optimize_png(original, if level == Level::Light { 2 } else { 4 })
        }
        _ => {
            let (image, icc) = decode(original)?;
            report(Progress::new(40, Stage::Encoding));
            let image = downscale(image, settings.max_side);
            match kind {
                Kind::Jpeg => encode_jpeg(&image, icc, settings.quality),
                Kind::Webp => Ok(encode_webp(&image, settings.quality)),
                Kind::Png => {
                    let quantized = encode_png(&quantize(&image), icc)?;
                    report(Progress::new(70, Stage::Optimizing));
                    optimize_png(&quantized, 3)
                }
            }
        }
    }
}

/// Görseli çözer, EXIF yönünü uygular ve varsa ICC profilini döndürür.
pub(crate) fn decode(bytes: &[u8]) -> Result<(DynamicImage, Option<Vec<u8>>), String> {
    let mut decoder = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| format!("Görsel okunamadı: {e}"))?
        .into_decoder()
        .map_err(|e| format!("Görsel çözülemedi: {e}"))?;
    let icc = decoder.icc_profile().ok().flatten();
    let orientation = decoder.orientation().ok();
    let mut image = DynamicImage::from_decoder(decoder).map_err(|e| format!("Görsel çözülemedi: {e}"))?;
    if let Some(orientation) = orientation {
        image.apply_orientation(orientation);
    }
    Ok((image, icc))
}

fn downscale(image: DynamicImage, max_side: Option<u32>) -> DynamicImage {
    match max_side {
        Some(max) if image.width().max(image.height()) > max => {
            // `resize` en-boy oranını koruyarak sınır kutusuna sığdırır.
            image.resize(max, max, FilterType::Triangle)
        }
        _ => image,
    }
}

fn encode_jpeg(image: &DynamicImage, icc: Option<Vec<u8>>, quality: u8) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut out, quality);
    if let Some(icc) = icc {
        let _ = encoder.set_icc_profile(icc);
    }
    let result = if image.color().has_color() {
        encoder.encode_image(&image.to_rgb8())
    } else {
        encoder.encode_image(&image.to_luma8())
    };
    result.map_err(|e| format!("JPEG kodlanamadı: {e}"))?;
    Ok(out)
}

fn encode_webp(image: &DynamicImage, quality: u8) -> Vec<u8> {
    let (w, h) = (image.width(), image.height());
    let memory = if image.color().has_alpha() {
        webp::Encoder::from_rgba(image.to_rgba8().as_raw(), w, h).encode(quality as f32)
    } else {
        webp::Encoder::from_rgb(image.to_rgb8().as_raw(), w, h).encode(quality as f32)
    };
    memory.to_vec()
}

/// Renkleri (alfa dahil) 256'lık bir paletle eşler. Sonuç RGBA kalır;
/// oxipng 256 veya daha az renkli görseli kendiliğinden paletli PNG'ye çevirir.
fn quantize(image: &DynamicImage) -> RgbaImage {
    let mut rgba = image.to_rgba8();
    let quantizer = color_quant::NeuQuant::new(10, 256, rgba.as_raw());
    let palette = quantizer.color_map_rgba();
    for pixel in rgba.pixels_mut() {
        let i = quantizer.index_of(&pixel.0) * 4;
        pixel.0.copy_from_slice(&palette[i..i + 4]);
    }
    rgba
}

pub(crate) fn encode_png(image: &RgbaImage, icc: Option<Vec<u8>>) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut encoder = PngEncoder::new_with_quality(&mut out, CompressionType::Fast, PngFilter::Adaptive);
    if let Some(icc) = icc {
        let _ = encoder.set_icc_profile(icc);
    }
    encoder
        .write_image(image.as_raw(), image.width(), image.height(), image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("PNG kodlanamadı: {e}"))?;
    Ok(out)
}

pub(crate) fn optimize_png(bytes: &[u8], preset: u8) -> Result<Vec<u8>, String> {
    let mut options = oxipng::Options::from_preset(preset);
    options.strip = oxipng::StripChunks::Safe;
    oxipng::optimize_from_memory(bytes, &options).map_err(|e| format!("PNG optimize edilemedi: {e}"))
}

/// APNG, ilk `IDAT`'tan önce bir `acTL` parçası içerir.
fn is_animated_png(bytes: &[u8]) -> bool {
    let mut pos = 8;
    while pos + 8 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        match &bytes[pos + 4..pos + 8] {
            b"acTL" => return true,
            b"IDAT" => return false,
            _ => pos += 12 + len,
        }
    }
    false
}

/// Genişletilmiş (VP8X) WebP başlığındaki animasyon bayrağı.
pub(crate) fn is_animated_webp(bytes: &[u8]) -> bool {
    bytes.len() > 20 && &bytes[12..16] == b"VP8X" && bytes[20] & 0x02 != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgb, RgbImage};

    /// Gürültülü degrade: gerçek fotoğraflar gibi sıkıştırması zor bir görsel.
    fn photo(width: u32, height: u32) -> DynamicImage {
        let mut seed = 12345u32;
        DynamicImage::ImageRgb8(RgbImage::from_fn(width, height, |x, y| {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let n = (seed >> 24) as u8 / 4;
            Rgb([((x * 255 / width) as u8).saturating_add(n), (y * 255 / height) as u8, 128 + n])
        }))
    }

    fn encode(image: &DynamicImage, format: ImageFormat) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, format).unwrap();
        out.into_inner()
    }

    fn high_quality_jpeg(image: &DynamicImage) -> Vec<u8> {
        let mut out = Vec::new();
        JpegEncoder::new_with_quality(&mut out, 98).encode_image(image).unwrap();
        out
    }

    #[test]
    fn jpeg_shrinks_and_respects_max_side() {
        let original = high_quality_jpeg(&photo(4000, 3000));
        let out = compress_bytes(&original, Kind::Jpeg, Level::Strong, &mut |_| {}).unwrap();
        assert!(out.len() * 3 < original.len(), "{} -> {}", original.len(), out.len());
        let decoded = image::load_from_memory_with_format(&out, ImageFormat::Jpeg).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (2000, 1500));
    }

    #[test]
    fn jpeg_light_keeps_dimensions() {
        let original = high_quality_jpeg(&photo(1200, 800));
        let out = compress_bytes(&original, Kind::Jpeg, Level::Light, &mut |_| {}).unwrap();
        let decoded = image::load_from_memory(&out).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (1200, 800));
    }

    #[test]
    fn png_lossless_levels_keep_pixels() {
        let image = photo(300, 200);
        let original = encode(&image, ImageFormat::Png);
        let out = compress_bytes(&original, Kind::Png, Level::Balanced, &mut |_| {}).unwrap();
        let decoded = image::load_from_memory(&out).unwrap();
        assert_eq!(decoded.to_rgb8(), image.to_rgb8());
    }

    #[test]
    fn png_strong_quantizes_and_shrinks() {
        let original = encode(&photo(800, 600), ImageFormat::Png);
        let out = compress_bytes(&original, Kind::Png, Level::Strong, &mut |_| {}).unwrap();
        assert!(out.len() * 2 < original.len(), "{} -> {}", original.len(), out.len());
        let decoded = image::load_from_memory(&out).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (800, 600));
    }

    #[test]
    fn webp_shrinks_lossless_input() {
        let original = encode(&photo(800, 600), ImageFormat::WebP);
        let out = compress_bytes(&original, Kind::Webp, Level::Balanced, &mut |_| {}).unwrap();
        assert!(out.len() < original.len(), "{} -> {}", original.len(), out.len());
        let decoded = image::load_from_memory_with_format(&out, ImageFormat::WebP).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (800, 600));
    }

    #[test]
    fn detects_animation_markers() {
        let mut apng = b"\x89PNG\r\n\x1a\n".to_vec();
        apng.extend_from_slice(&[0, 0, 0, 13]);
        apng.extend_from_slice(b"IHDR");
        apng.extend_from_slice(&[0; 17]);
        apng.extend_from_slice(&[0, 0, 0, 8]);
        apng.extend_from_slice(b"acTL");
        assert!(is_animated_png(&apng));
        assert!(!is_animated_png(&encode(&photo(10, 10), ImageFormat::Png)));

        let mut webp = b"RIFF\0\0\0\0WEBPVP8X".to_vec();
        webp.extend_from_slice(&[10, 0, 0, 0, 0x02]);
        assert!(is_animated_webp(&webp));
    }
}
