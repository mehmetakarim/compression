//! PDF sıkıştırma motoru.
//!
//! Kazancın büyük kısmı gömülü görsellerden gelir: görseller seviyeye göre
//! küçültülür ve JPEG olarak yeniden kodlanır. Ardından görsel olmayan akışlar
//! en yüksek Flate seviyesiyle yeniden sıkıştırılır ve dosya nesne akışları
//! (object streams) ile kaydedilir. Sonuç orijinalden küçük değilse dosya
//! yazılmaz.

use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

use flate2::write::ZlibEncoder;
use flate2::Compression;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GrayImage, ImageFormat, RgbImage};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

use crate::common::{file_size, write_if_smaller, Level, Outcome, Progress, Stage};

/// Tek bir akışın açılmış hâli için üst sınır (sıkıştırma bombalarına karşı).
const MAX_DECODED_BYTES: usize = 512 * 1024 * 1024;
/// Bundan küçük görseller yeniden kodlamaya değmez.
const MIN_IMAGE_PIXELS: u64 = 64 * 64;

struct Settings {
    /// Görselin uzun kenarı için piksel üst sınırı.
    max_side: u32,
    jpeg_quality: u8,
}

fn settings(level: Level) -> Settings {
    match level {
        Level::Light => Settings { max_side: 2400, jpeg_quality: 85 },
        Level::Balanced => Settings { max_side: 1600, jpeg_quality: 70 },
        Level::Strong => Settings { max_side: 1100, jpeg_quality: 50 },
    }
}

// İlerleme çubuğundaki aşama aralıkları (yüzde).
const READ_END: f64 = 10.0;
const IMAGES_END: f64 = 90.0;

/// `input` dosyasını sıkıştırır ve aynı klasöre `<ad>-compressed.pdf` yazar.
pub fn compress_file(
    input: &Path,
    level: Level,
    report: &mut dyn FnMut(Progress),
) -> Result<Outcome, String> {
    let original_size = file_size(input)?;
    report(Progress::new(0, Stage::Reading));
    let mut doc = Document::load(input).map_err(|e| format!("PDF açılamadı: {e}"))?;
    if doc.was_encrypted() || doc.is_encrypted() {
        return Err("Şifreli PDF dosyaları henüz desteklenmiyor.".into());
    }
    let (bytes, _) = compress_document(&mut doc, level, report)?;
    write_if_smaller(input, original_size, &bytes)
}

/// Belgeyi bellekte sıkıştırır; kaydedilmiş baytları ve yeniden kodlanan
/// görsel sayısını döndürür. Çıktı yeniden ayrıştırılarak doğrulanır.
pub fn compress_document(
    doc: &mut Document,
    level: Level,
    report: &mut dyn FnMut(Progress),
) -> Result<(Vec<u8>, usize), String> {
    let settings = settings(level);

    // Yumuşak maskeler (alfa kanalı) JPEG'e çevrilmez; kenarlarda bozulma yapar.
    let smask_ids: HashSet<ObjectId> = doc
        .objects
        .values()
        .filter_map(|o| match o {
            Object::Stream(s) => s.dict.get(b"SMask").and_then(Object::as_reference).ok(),
            _ => None,
        })
        .collect();

    let image_ids: Vec<ObjectId> = doc
        .objects
        .iter()
        .filter(|(id, o)| !smask_ids.contains(id) && matches!(o, Object::Stream(s) if is_image(s)))
        .map(|(id, _)| *id)
        .collect();

    // İlerleme, görsellerin bayt büyüklüğüyle ağırlıklandırılır; büyük bir
    // tarama sayfası küçük bir logodan daha uzun sürer.
    let weights: Vec<u64> = image_ids
        .iter()
        .map(|id| match doc.objects.get(id) {
            Some(Object::Stream(s)) => s.content.len() as u64 + 1,
            _ => 1,
        })
        .collect();
    let total_weight: u64 = weights.iter().sum::<u64>().max(1);
    let images_total = image_ids.len();
    let mut done_weight = 0u64;
    report(Progress {
        percent: READ_END as u8,
        stage: Stage::Images,
        done: 0,
        total: images_total,
    });

    let mut images_recompressed = 0;
    for (index, id) in image_ids.into_iter().enumerate() {
        done_weight += weights[index];
        let replacement = {
            let Ok(Object::Stream(stream)) = doc.get_object(id) else { continue };
            let Some(components) = color_components(doc, &stream.dict) else { continue };
            recompress_image(stream, components, &settings)
        };
        if let Some((data, width, height)) = replacement {
            if let Ok(Object::Stream(stream)) = doc.get_object_mut(id) {
                stream.dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
                stream.dict.remove(b"DecodeParms");
                stream.dict.set("Width", width as i64);
                stream.dict.set("Height", height as i64);
                stream.dict.set("BitsPerComponent", 8i64);
                stream.set_content(data);
                stream.allows_compression = false;
                images_recompressed += 1;
            }
        }
        let fraction = done_weight as f64 / total_weight as f64;
        report(Progress {
            percent: (READ_END + (IMAGES_END - READ_END) * fraction) as u8,
            stage: Stage::Images,
            done: index + 1,
            total: images_total,
        });
    }

    report(Progress {
        percent: IMAGES_END as u8,
        stage: Stage::Saving,
        done: images_total,
        total: images_total,
    });
    for object in doc.objects.values_mut() {
        if let Object::Stream(stream) = object {
            if !is_image(stream) {
                redeflate(stream);
            }
        }
    }
    doc.compress();

    let mut bytes = Vec::new();
    doc.save_modern(&mut bytes)
        .map_err(|e| format!("PDF kaydedilemedi: {e}"))?;
    Document::load_mem(&bytes).map_err(|e| format!("Sıkıştırılmış çıktı doğrulanamadı: {e}"))?;
    report(Progress {
        percent: 100,
        stage: Stage::Saving,
        done: images_total,
        total: images_total,
    });
    Ok((bytes, images_recompressed))
}

fn is_image(stream: &Stream) -> bool {
    matches!(stream.dict.get(b"Subtype").and_then(Object::as_name), Ok(b"Image"))
}

/// Görselin renk bileşeni sayısı; yalnızca gri (1) ve RGB (3) desteklenir.
fn color_components(doc: &Document, dict: &Dictionary) -> Option<u8> {
    let cs = doc.dereference(dict.get(b"ColorSpace").ok()?).ok()?.1;
    let n = match cs {
        Object::Name(name) => match name.as_slice() {
            b"DeviceGray" | b"G" => 1,
            b"DeviceRGB" | b"RGB" => 3,
            _ => return None,
        },
        Object::Array(arr) => match arr.first()?.as_name().ok()? {
            b"CalGray" => 1,
            b"CalRGB" => 3,
            b"ICCBased" => {
                let profile = doc.dereference(arr.get(1)?).ok()?.1.as_stream().ok()?;
                profile.dict.get(b"N").and_then(Object::as_i64).ok()? as u8
            }
            _ => return None,
        },
        _ => return None,
    };
    matches!(n, 1 | 3).then_some(n)
}

/// Görseli gerekirse küçültüp JPEG olarak kodlar. Anlamlı kazanç yoksa `None`.
fn recompress_image(stream: &Stream, components: u8, settings: &Settings) -> Option<(Vec<u8>, u32, u32)> {
    let dict = &stream.dict;
    if dict.get(b"ImageMask").and_then(Object::as_bool).unwrap_or(false) || dict.has(b"Decode") {
        return None;
    }
    if dict.get(b"BitsPerComponent").and_then(Object::as_i64).ok()? != 8 {
        return None;
    }
    let width = u32::try_from(dict.get(b"Width").and_then(Object::as_i64).ok()?).ok()?;
    let height = u32::try_from(dict.get(b"Height").and_then(Object::as_i64).ok()?).ok()?;
    if (width as u64) * (height as u64) < MIN_IMAGE_PIXELS {
        return None;
    }

    let filters = stream.filters().unwrap_or_default();
    let image = match filters.as_slice() {
        [b"DCTDecode"] => {
            let img = image::load_from_memory_with_format(&stream.content, ImageFormat::Jpeg).ok()?;
            if img.width() != width || img.height() != height || img.color().channel_count() != components {
                return None;
            }
            img
        }
        [] | [b"FlateDecode"] => {
            let raw = stream.get_plain_content_with_limit(MAX_DECODED_BYTES).ok()?;
            if raw.len() != width as usize * height as usize * components as usize {
                return None;
            }
            if components == 1 {
                DynamicImage::ImageLuma8(GrayImage::from_raw(width, height, raw)?)
            } else {
                DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, raw)?)
            }
        }
        _ => return None,
    };

    let image = downscale(image, settings.max_side);
    let mut encoded = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut encoded, settings.jpeg_quality);
    let result = if components == 1 {
        encoder.encode_image(&image.to_luma8())
    } else {
        encoder.encode_image(&image.to_rgb8())
    };
    result.ok()?;

    // En az ~%10 kazanç yoksa orijinali koru; gereksiz kalite kaybını önler.
    if encoded.len() + encoded.len() / 10 >= stream.content.len() {
        return None;
    }
    Some((encoded, image.width(), image.height()))
}

fn downscale(image: DynamicImage, max_side: u32) -> DynamicImage {
    let (w, h) = (image.width(), image.height());
    let long = w.max(h);
    if long <= max_side {
        return image;
    }
    let scale = max_side as f64 / long as f64;
    let nw = ((w as f64 * scale).round() as u32).max(1);
    let nh = ((h as f64 * scale).round() as u32).max(1);
    // Triangle, küçültmede Lanczos3'e yakın sonuç verip belirgin şekilde hızlıdır.
    image.resize_exact(nw, nh, FilterType::Triangle)
}

/// Yalnızca tek `FlateDecode` filtreli ve parametresiz akışları en yüksek
/// seviyede yeniden sıkıştırır; daha küçük değilse dokunmaz.
fn redeflate(stream: &mut Stream) {
    let only_flate = matches!(stream.filters().as_deref(), Ok([b"FlateDecode"]));
    if !only_flate || stream.dict.has(b"DecodeParms") {
        return;
    }
    let Ok(plain) = stream.get_plain_content_with_limit(MAX_DECODED_BYTES) else { return };
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    if encoder.write_all(&plain).is_err() {
        return;
    }
    if let Ok(packed) = encoder.finish() {
        if packed.len() < stream.content.len() {
            stream.set_content(packed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Object};
    use std::fs;
    use std::path::PathBuf;

    /// Tek sayfalı, büyük ve sıkıştırılmamış RGB görsel içeren bir PDF üretir.
    fn sample_pdf(width: u32, height: u32) -> Document {
        let mut doc = Document::with_version("1.4");
        let pixels: Vec<u8> = (0..width * height)
            .flat_map(|i| {
                let (x, y) = (i % width, i / width);
                [(x * 255 / width) as u8, (y * 255 / height) as u8, ((x ^ y) & 0xff) as u8]
            })
            .collect();
        let image = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => width as i64,
                "Height" => height as i64,
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => 8,
            },
            pixels,
        );
        let image_id = doc.add_object(image);
        let content = doc.add_object(Stream::new(dictionary! {}, b"q 500 0 0 500 50 50 cm /Im0 Do Q".to_vec()));
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 600.into(), 600.into()],
            "Contents" => content,
            "Resources" => dictionary! { "XObject" => dictionary! { "Im0" => image_id } },
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page_id.into()],
                "Count" => 1,
            }),
        );
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        doc
    }

    fn saved_len(doc: &mut Document) -> usize {
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out.len()
    }

    #[test]
    fn shrinks_and_downscales_large_image() {
        let mut doc = sample_pdf(3000, 2000);
        let before = saved_len(&mut doc);
        let (bytes, images) = compress_document(&mut doc, Level::Balanced, &mut |_| {}).unwrap();
        assert_eq!(images, 1);
        assert!(bytes.len() * 10 < before, "{} -> {}", before, bytes.len());

        let out = Document::load_mem(&bytes).unwrap();
        let image = out
            .objects
            .values()
            .find_map(|o| o.as_stream().ok().filter(|s| is_image(s)))
            .unwrap();
        assert_eq!(image.dict.get(b"Width").unwrap().as_i64().unwrap(), 1600);
        assert_eq!(image.dict.get(b"Height").unwrap().as_i64().unwrap(), 1067);
        assert_eq!(image.filters().unwrap(), vec![b"DCTDecode".as_slice()]);
        assert_eq!(out.get_pages().len(), 1);
    }

    #[test]
    fn progress_is_monotonic_and_completes() {
        let mut seen = Vec::new();
        compress_document(&mut sample_pdf(1800, 1800), Level::Balanced, &mut |p| seen.push(p)).unwrap();
        let percents: Vec<u8> = seen.iter().map(|p| p.percent).collect();
        assert!(percents.windows(2).all(|w| w[0] <= w[1]), "{percents:?}");
        assert_eq!(*percents.last().unwrap(), 100);
        assert!(seen.iter().any(|p| matches!(p.stage, Stage::Images) && p.done == 1 && p.total == 1));
    }

    #[test]
    fn stronger_level_gives_smaller_output() {
        let (light, _) = compress_document(&mut sample_pdf(2600, 2600), Level::Light, &mut |_| {}).unwrap();
        let (strong, _) = compress_document(&mut sample_pdf(2600, 2600), Level::Strong, &mut |_| {}).unwrap();
        assert!(strong.len() < light.len());
    }

    /// Gerçek PDF'lerle elle doğrulama:
    /// `COMPRESSION_SAMPLES=<klasör> cargo test real_samples -- --ignored --nocapture`
    /// Çıktılar örneklerin yanına `-compressed` ekiyle yazılır.
    #[test]
    #[ignore]
    fn real_samples() {
        let dir = std::env::var("COMPRESSION_SAMPLES").expect("COMPRESSION_SAMPLES tanımlı değil");
        let mut inputs: Vec<PathBuf> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
                    && !p.to_string_lossy().contains("-compressed")
            })
            .collect();
        inputs.sort();
        for input in inputs {
            for level in [Level::Light, Level::Balanced, Level::Strong] {
                match compress_file(&input, level, &mut |_| {}) {
                    Ok(o) => {
                        if let Some(out) = &o.output_path {
                            let pages = Document::load(&input).unwrap().get_pages().len();
                            assert_eq!(Document::load(out).unwrap().get_pages().len(), pages);
                        }
                        println!(
                            "{:?} {:?}: {} -> {} {:?}",
                            input.file_name().unwrap(),
                            level,
                            o.original_size,
                            o.compressed_size,
                            o.output_path.as_deref().map(|p| Path::new(p).file_name().unwrap())
                        );
                    }
                    Err(e) => println!("{:?} {:?}: HATA {e}", input.file_name().unwrap(), level),
                }
            }
        }
    }
}
