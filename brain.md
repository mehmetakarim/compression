# brain.md — Güncel Çalışma Durumu

> Kısa tutulur (≈50–100 satır). Günlük değildir; eski durumlar ilgili kayda
> taşınır. Kurallar: [AGENTS.md](AGENTS.md)

## Son güncelleme

2026-10-08 13:55 (UTC+03:00)

## Git

- Aktif branch: `main` → `origin/main` (<https://github.com/mehmetakarim/compression>)
- Referans: uygulama kodu ve release iş akışı tek commit'te `main`'e push
  edildi; `v0.1.0` etiketi release iş akışını tetikler (commit kimliği için
  `git log -1`).

## Çalışma ağacı (son kontrol)

Release commit'i sonrası temiz. `.claude/` (yerel önizleme ayarı) Git dışı.

## Proje durumu

Tauri 2 + Rust masaüstü uygulaması: "Sıkıştırma" (PDF, JPG, PNG, WebP, MP4)
ve "Dönüştür" (PNG ↔ WebP) sekmeleri kodlandı.
Ayrıntı: [docs/architecture.md](docs/architecture.md).

## Aktif hedef

[TASK-004](docs/tasks.md) — GitHub Release v0.1.0 (Windows .exe + macOS .dmg,
GitHub Actions). TASK-001…003 kodu bu release ile commit edildi.

## Tamamlanan son anlamlı aşama

İki sekme eklendi: "Sıkıştırma" (mevcut ekran) ve "Dönüştür" (`convert.rs`;
PNG → WebP kayıpsız/kayıplı, WebP → PNG). Daha önce: görsel (`raster.rs`)
ve MP4 desteği (`video.rs`, sistem FFmpeg'i ve kurulum yardımı,
[ADR 0003](docs/decisions/0003-mp4-icin-sistem-ffmpeg.md)), ilerleme çubuğu,
dev modda hız düzeltmesi (BUG-003), footer imzası.

## Devam eden işler

- v0.1.0 release iş akışı (taslak release → iki yükleyici → yayınlama).

## Engeller ve açık sorunlar

- [ADR 0002](docs/decisions/0002-pdf-sikistirma-motoru.md) "Önerildi":
  saf Rust motoru için kullanıcı onayı bekleniyor.
- CMYK/Indexed görseller, JPX/JBIG2 ve şifreli PDF'ler desteklenmiyor.
- Flate ile saklanan keskin grafikler JPEG'e çevrilince bozulabilir.

## Son doğrulamalar (2026-10-08)

- `cargo test` 16/16 geçti; `cargo clippy --all-targets` uyarısız.
- PNG ↔ WebP dönüştürme uygulama içinde doğrulandı (sonuçlar TASK-003'te).
- JPG/PNG/WebP/MP4 uygulama içinde sıkıştırıldı (sonuçlar TASK-002'de).
- Makinede iki FFmpeg var: `C:\Users\Lenovo\.local\bin` (8.1.1, PATH'te
  önce; uygulama bunu kullanıyor) ve winget Gyan.FFmpeg 9.0.2 (test
  sırasında yanlışlıkla yükseltildi, bkz. BUG-004).
- 31 MB / 8 sayfalık sentetik tarama dev modda 4,4 sn (önce dakikalar).
- Fotoğraflı PDF 2,95 MB → 870 / 427 / 204 KB (Hafif/Dengeli/Güçlü);
  çıktılar PDFium ile render edildi, sayfa sayıları korundu.
- Arayüz tarayıcıda Tauri API taklidiyle açık/koyu temada kontrol edildi.
- İlerleme çubuğu gerçek pencerede CDP ekran görüntüsüyle doğrulandı.

## Henüz doğrulanmamış noktalar

- "Klasörde göster" düğmesi; sürükle-bırak ve dosya seçicinin ayrı ayrı.
- Düzeltilmiş `spawn_in_new_console` ile gerçek winget kurulum penceresi.
- macOS .dmg'nin CI'da derlenmesi ve gerçek bir Mac'te açılması.
- macOS/Linux derlemesi.
- Kullanıcıların gerçek (taranmış, CMYK, büyük) PDF'leriyle davranış.

## Sonraki somut adım

`gh run list --workflow release.yml` ile v0.1.0 derlemesini kontrol et;
iki yükleyici de taslak release'teyse `gh release edit v0.1.0 --draft=false`
ile yayınla ve TASK-004'ü güncelle. Hata varsa iş akışı günlüğünü incele.

## Bağlantılar

- Görevler: [docs/tasks.md](docs/tasks.md)
- Kararlar: [docs/decisions/README.md](docs/decisions/README.md)
- Çözümler: [docs/solutions.md](docs/solutions.md)
- Mimari: [docs/architecture.md](docs/architecture.md)
