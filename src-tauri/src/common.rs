//! Dosya türlerinden bağımsız ortak tipler ve yardımcılar.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Light,
    Balanced,
    Strong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Reading,
    Images,
    Encoding,
    Optimizing,
    Video,
    Saving,
}

/// Arayüze gönderilen ilerleme bilgisi. `done`/`total` aşamaya göre anlam
/// taşır (PDF'te görsel sayısı); kullanılmıyorsa 0'dır.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub percent: u8,
    pub stage: Stage,
    pub done: usize,
    pub total: usize,
}

impl Progress {
    pub fn new(percent: u8, stage: Stage) -> Self {
        Self { percent, stage, done: 0, total: 0 }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    /// Yazılan dosyanın yolu; kazanç yoksa `None`.
    pub output_path: Option<String>,
    pub original_size: u64,
    pub compressed_size: u64,
}

impl Outcome {
    pub fn not_smaller(original_size: u64) -> Self {
        Self { output_path: None, original_size, compressed_size: original_size }
    }
}

pub fn file_size(path: &Path) -> Result<u64, String> {
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("Dosya okunamadı: {e}"))
}

/// Sonuç orijinalden küçükse `<ad>-compressed.<uzantı>` olarak yazar.
pub fn write_if_smaller(input: &Path, original_size: u64, bytes: &[u8]) -> Result<Outcome, String> {
    if bytes.len() as u64 >= original_size {
        return Ok(Outcome::not_smaller(original_size));
    }
    let output = unique_output_path(input);
    fs::write(&output, bytes).map_err(|e| format!("Çıktı yazılamadı: {e}"))?;
    Ok(Outcome {
        output_path: Some(output.to_string_lossy().into_owned()),
        original_size,
        compressed_size: bytes.len() as u64,
    })
}

/// `rapor.pdf` → `rapor-compressed.pdf`, varsa `rapor-compressed (2).pdf` …
/// Uzantı orijinal yazımıyla korunur (`foto.JPEG` → `foto-compressed.JPEG`).
pub fn unique_output_path(input: &Path) -> PathBuf {
    let stem = input.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
    let ext = input.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
    unique_sibling(input, &format!("{stem}-compressed"), &ext)
}

/// `input` ile aynı klasörde `<ad>.<uzantı>` yolunu döndürür; varsa
/// `<ad> (2).<uzantı>`, `<ad> (3).<uzantı>` … denenir. Var olan dosyanın
/// üzerine asla yazılmaz.
pub fn unique_sibling(input: &Path, name: &str, ext: &str) -> PathBuf {
    let dir = input.parent().unwrap_or_else(|| Path::new("."));
    let ext = if ext.is_empty() { String::new() } else { format!(".{ext}") };
    let mut candidate = dir.join(format!("{name}{ext}"));
    let mut n = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{name} ({n}){ext}"));
        n += 1;
    }
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_does_not_overwrite() {
        let dir = std::env::temp_dir().join(format!("compression-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("rapor.pdf");
        assert_eq!(unique_output_path(&input), dir.join("rapor-compressed.pdf"));
        fs::write(dir.join("rapor-compressed.pdf"), b"x").unwrap();
        assert_eq!(unique_output_path(&input), dir.join("rapor-compressed (2).pdf"));
        assert_eq!(unique_output_path(&dir.join("foto.JPEG")), dir.join("foto-compressed.JPEG"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
