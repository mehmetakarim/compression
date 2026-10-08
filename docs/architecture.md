# Mimari

> Yalnızca dosyalardan doğrulanmış bilgi yazılır. Gözlenen ama karar olarak
> onaylanmamış tercihler "Gözlem" olarak işaretlenir; kararlar
> [decisions/](decisions/README.md) altındadır.

Son doğrulama: 2026-10-08 (kod ve `Cargo.toml`/`package.json` incelendi).

## Amaç

Dosya sıkıştıran ve dönüştüren cross-platform masaüstü mini uygulaması. İki
ekran (sekme) vardır:

- **Sıkıştırma:** PDF, JPG/JPEG, PNG, WebP, MP4.
- **Dönüştür:** PNG → WebP (kayıpsız varsayılan, isteğe bağlı kayıplı q90),
  WebP → PNG.

## Teknoloji yığını

| Katman | Teknoloji | Kaynak |
|---|---|---|
| Masaüstü çerçevesi | Tauri 2.12 (`tauri`, `tauri-plugin-dialog`, `tauri-plugin-opener`) | `src-tauri/Cargo.toml`, [ADR 0001](decisions/0001-tauri-masaustu-cercevesi.md) |
| Arka uç | Rust (edition 2021) | `src-tauri/` |
| PDF işleme | `lopdf` 0.45, `image` 0.25, `flate2` | [ADR 0002](decisions/0002-pdf-sikistirma-motoru.md) |
| Görsel işleme | `image` 0.25 (`jpeg`, `png`, `webp`), `oxipng` 10, `webp` 0.3 (libwebp), `color_quant` 1.1 | `src-tauri/src/raster.rs` |
| Video | Sistemdeki FFmpeg (alt süreç) | [ADR 0003](decisions/0003-mp4-icin-sistem-ffmpeg.md) |
| Arayüz | Paketleyicisiz HTML/CSS/JS, `window.__TAURI__` | `ui/` |
| CLI | `@tauri-apps/cli` 2.12.1 (npm devDependency) | `package.json` |

## Önemli dizinler

| Yol | Sorumluluk |
|---|---|
| `ui/` | Arayüz: `index.html` (iki sekme/panel), `styles.css`, `main.js` (Tauri `frontendDist`) |
| `src-tauri/src/lib.rs` | Tauri uygulaması; `compress_file`, `convert_file`, `ffmpeg_status`, `install_ffmpeg` komutları |
| `src-tauri/src/convert.rs` | PNG ↔ WebP dönüştürme ve testleri |
| `src-tauri/src/common.rs` | Ortak tipler: `Level`, `Progress`, `Stage`, `Outcome`, çıktı adı |
| `src-tauri/src/pdf.rs` | PDF sıkıştırma motoru ve testleri |
| `src-tauri/src/raster.rs` | JPG/PNG/WebP sıkıştırma ve testleri |
| `src-tauri/src/video.rs` | FFmpeg bulma/kurma, MP4 sıkıştırma |
| `src-tauri/tauri.conf.json` | Pencere, CSP, paketleme ayarları |
| `src-tauri/capabilities/default.json` | Arayüze verilen izinler |
| `src-tauri/icons/` | `app-icon.svg` kaynağından `tauri icon` ile üretilmiş ikonlar |
| `docs/` | Proje hafızası belgeleri |

## Veri akışı

1. Kullanıcı dosyayı bırakır (`onDragDropEvent`) veya seçer (`dialog.open`).
   Bırakılan dosyalar açık olan sekmeye gider; her sekmenin kendi listesi var.
2. `ui/main.js` dosyaları iki sekme için ortak tek bir sıraya alır; aynı
   anda tek dosya işlenir. Dönüştür sekmesi `convert_file` komutunu
   (`{ path, to: "webp" | "png", lossless }`) çağırır; çıktı aynı adla yeni
   uzantıyla yazılır (`logo.png` → `logo.webp`, varsa `logo (2).webp`) ve
   orijinalden büyük olsa da kaydedilir.
3. `invoke("compress_file", { path, level, onProgress })` → `lib.rs` işi
   `spawn_blocking` ile çalıştırır ve uzantıya göre `pdf`, `raster` veya
   `video` modülüne yönlendirir. `onProgress` bir Tauri `Channel`'ıdır;
   `Progress { percent, stage, done, total }` gönderir. Aşamalar: `reading`,
   `images` (PDF; görsel bayt boyutuyla ağırlıklı), `encoding`, `optimizing`,
   `video` (FFmpeg `-progress` çıktısındaki süreye göre), `saving`.
   MP4 eklenirken FFmpeg yoksa satır "FFmpeg bekleniyor" durumunda tutulur
   ve kurulum kutusu gösterilir (bkz. ADR 0003).
4. Örnek akış — `pdf.rs`: PDF'i yükler → şifreliyse reddeder → görselleri yeniden kodlar →
   akışları yeniden sıkıştırır → `save_modern` ile belleğe kaydeder →
   `load_mem` ile doğrular → küçükse `<ad>-compressed.pdf` olarak orijinalin
   yanına yazar (var olan dosyanın üzerine yazmaz, `(2)`, `(3)` ekler).
5. Sonuç (`outputPath`, `originalSize`, `compressedSize`,
   `imagesRecompressed`) arayüzde gösterilir; "Klasörde göster" `opener`
   eklentisini kullanır.

Footer'daki `target="_blank"` link, `tauri-plugin-opener`'ın tıklama
yakalayıcısıyla sistem tarayıcısında açılır. `capabilities/default.json`
içinde `opener:allow-open-url` yalnızca `https://www.mehmetakar.com.tr`
adresine izinlidir (başka URL'nin reddedildiği CDP ile doğrulandı).

Orijinal dosya hiçbir durumda değiştirilmez veya silinmez. Listedeki çöp kutusu
düğmesi yalnızca satırı listeden kaldırır.

## Sıkıştırma seviyeleri

| Tür | Hafif (`light`) | Dengeli (`balanced`, varsayılan) | Güçlü (`strong`) |
|---|---|---|---|
| PDF içi görsel | ≤2400 px, JPEG q85 | ≤1600 px, q70 | ≤1100 px, q50 |
| JPG/JPEG | boyut korunur, q85 | ≤3000 px, q75 | ≤2000 px, q60 |
| WebP (kayıplı) | boyut korunur, q85 | ≤3000 px, q75 | ≤2000 px, q60 |
| PNG | kayıpsız, oxipng preset 2 | kayıpsız, preset 4 | ≤2000 px, 256 renk + preset 3 |
| MP4 (libx264) | CRF 23, çözünürlük korunur, AAC 160k | CRF 26, kısa kenar ≤1080, AAC 128k | CRF 30, kısa kenar ≤720, AAC 96k |

Piksel sınırları uzun kenar içindir (MP4'te kısa kenar). PDF'te sınır piksel
bazlıdır, baskı boyutuna (DPI) göre değil.

## Bilinen sınırlamalar

- JPG/WebP yeniden kodlanınca EXIF meta verisi (kamera, konum, tarih) atılır;
  EXIF yönü piksellere uygulanır. JPEG'de ICC profili korunur, WebP'de korunmaz.
- Animasyonlu PNG/WebP reddedilir. Şifreli PDF reddedilir.
- MP4 çıktısı 8-bit `yuv420p`; HDR/10-bit kaynaklarda renkler değişebilir.
  Yalnızca ilk video akışı ve tüm ses akışları alınır; altyazılar atılır.
- Devam eden işlem iptal edilemez; uygulama kapanırsa FFmpeg süreci sürer.

## Dış entegrasyonlar

- GitHub uzak deposu: <https://github.com/mehmetakarim/compression> (`origin`).
- Ağ erişimi yok; tüm işlem yereldir.
