//! Biçim dönüştürme: PNG → WebP ve WebP → PNG.
//!
//! Çıktı orijinalin yanına aynı adla, yeni uzantıyla yazılır (`logo.png` →
//! `logo.webp`); dosya varsa `logo (2).webp` denenir. Dönüştürmede çıktı
//! orijinalden büyük olsa da yazılır.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::common::{unique_sibling, Outcome, Progress, Stage};
use crate::raster;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Webp,
    Png,
}

impl Target {
    fn source_extension(self) -> &'static str {
        match self {
            Target::Webp => "png",
            Target::Png => "webp",
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Target::Webp => "webp",
            Target::Png => "png",
        }
    }
}

/// Kayıplı PNG → WebP için kalite; görsel farkın fark edilmeyeceği düzey.
const LOSSY_WEBP_QUALITY: f32 = 90.0;

pub fn convert_file(
    input: &Path,
    target: Target,
    lossless: bool,
    report: &mut dyn FnMut(Progress),
) -> Result<Outcome, String> {
    let source_ext = input
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if source_ext != target.source_extension() {
        return Err(format!(
            "Bu seçenek yalnızca {} dosyalarını dönüştürür.",
            target.source_extension().to_uppercase()
        ));
    }

    report(Progress::new(0, Stage::Reading));
    let original = fs::read(input).map_err(|e| format!("Dosya okunamadı: {e}"))?;
    let bytes = convert_bytes(&original, target, lossless, report)?;

    let stem = input.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
    let output = unique_sibling(input, &stem, target.extension());
    fs::write(&output, &bytes).map_err(|e| format!("Çıktı yazılamadı: {e}"))?;
    report(Progress::new(100, Stage::Saving));
    Ok(Outcome {
        output_path: Some(output.to_string_lossy().into_owned()),
        original_size: original.len() as u64,
        compressed_size: bytes.len() as u64,
    })
}

pub fn convert_bytes(
    original: &[u8],
    target: Target,
    lossless: bool,
    report: &mut dyn FnMut(Progress),
) -> Result<Vec<u8>, String> {
    if target == Target::Png && raster::is_animated_webp(original) {
        return Err("Animasyonlu WebP dosyaları henüz desteklenmiyor.".into());
    }
    let (image, icc) = raster::decode(original)?;
    report(Progress::new(40, Stage::Encoding));
    match target {
        Target::Webp => {
            let (w, h) = (image.width(), image.height());
            let encoder_input;
            let encoder = if image.color().has_alpha() {
                encoder_input = image.to_rgba8().into_raw();
                webp::Encoder::from_rgba(&encoder_input, w, h)
            } else {
                encoder_input = image.to_rgb8().into_raw();
                webp::Encoder::from_rgb(&encoder_input, w, h)
            };
            let memory = if lossless {
                encoder.encode_lossless()
            } else {
                encoder.encode(LOSSY_WEBP_QUALITY)
            };
            Ok(memory.to_vec())
        }
        Target::Png => {
            let png = raster::encode_png(&image.to_rgba8(), icc)?;
            report(Progress::new(70, Stage::Optimizing));
            // Kayıpsız optimizasyon: gereksiz alfa kanalını ve büyük
            // sıkıştırmayı temizler, pikselleri değiştirmez.
            raster::optimize_png(&png, 2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    fn sample() -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_fn(200, 120, |x, y| {
            Rgba([(x % 256) as u8, (y * 2 % 256) as u8, 90, if x < 100 { 255 } else { 128 }])
        }))
    }

    fn encode(image: &DynamicImage, format: ImageFormat) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, format).unwrap();
        out.into_inner()
    }

    #[test]
    fn png_to_webp_lossless_keeps_pixels_and_alpha() {
        let image = sample();
        let webp = convert_bytes(&encode(&image, ImageFormat::Png), Target::Webp, true, &mut |_| {}).unwrap();
        let decoded = image::load_from_memory_with_format(&webp, ImageFormat::WebP).unwrap();
        assert_eq!(decoded.to_rgba8(), image.to_rgba8());
    }

    #[test]
    fn png_to_webp_lossy_is_valid_webp() {
        let webp = convert_bytes(&encode(&sample(), ImageFormat::Png), Target::Webp, false, &mut |_| {}).unwrap();
        let decoded = image::load_from_memory_with_format(&webp, ImageFormat::WebP).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (200, 120));
        assert!(decoded.color().has_alpha());
    }

    #[test]
    fn webp_to_png_keeps_pixels() {
        let image = sample();
        let png = convert_bytes(&encode(&image, ImageFormat::WebP), Target::Png, false, &mut |_| {}).unwrap();
        let decoded = image::load_from_memory_with_format(&png, ImageFormat::Png).unwrap();
        assert_eq!(decoded.to_rgba8(), image.to_rgba8());
    }

    #[test]
    fn rejects_wrong_source_and_never_overwrites() {
        let dir = std::env::temp_dir().join(format!("compression-convert-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let png = dir.join("logo.png");
        fs::write(&png, encode(&sample(), ImageFormat::Png)).unwrap();

        let err = convert_file(&png, Target::Png, false, &mut |_| {}).unwrap_err();
        assert!(err.contains("WEBP"), "{err}");

        fs::write(dir.join("logo.webp"), b"mevcut").unwrap();
        let out = convert_file(&png, Target::Webp, true, &mut |_| {}).unwrap();
        assert_eq!(out.output_path.unwrap(), dir.join("logo (2).webp").to_string_lossy());
        assert_eq!(fs::read(dir.join("logo.webp")).unwrap(), b"mevcut");
        fs::remove_dir_all(&dir).unwrap();
    }
}
