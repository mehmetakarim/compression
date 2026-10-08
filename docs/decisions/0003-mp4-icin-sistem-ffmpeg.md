# 0003 — MP4 için sistemdeki FFmpeg (paketlenmez)

- Tarih: 2026-10-08
- Durum: Kabul edildi
- Yerine geçtiği karar: —
- Yerine geçen karar: —

## Bağlam ve problem

MP4 sıkıştırma desteği istendi. Rust'ta video yeniden kodlayan olgun, saf bir
kütüphane yok; pratik çözüm FFmpeg.

## Değerlendirilen seçenekler

- **FFmpeg'i uygulamayla paketlemek (Tauri sidecar):** Kurulum gerektirmez;
  yükleyici ~80–100 MB büyür, her platform için ayrı ikili ve FFmpeg lisans
  (LGPL/GPL) yükümlülükleri gerekir.
- **Sistemdeki FFmpeg'i kullanmak:** Uygulama küçük kalır; FFmpeg yoksa
  kullanıcının kurması gerekir.
- **MP4'ü ertelemek.**

## Seçilen yaklaşım ve gerekçe

Kullanıcının kararı: sistemdeki FFmpeg kullanılır; zaten FFmpeg'i olan
kullanıcılar için gömülü kopya gereksiz yer kaplar. FFmpeg yoksa uygulama
kurulum yardımı sunar:

- Windows: `winget install --id Gyan.FFmpeg --exact --source winget`
  ayrı, görünür bir konsolda başlatılır.
- macOS: Homebrew varsa Terminal'de `brew install ffmpeg`; yoksa komut ve
  ffmpeg.org bağlantısı gösterilir.
- Linux: `sudo` gerektiği için komut gösterilir.

Arayüz kurulum sırasında `ffmpeg_status`'u 4 sn'de bir (en çok 15 dk)
yoklar; FFmpeg bulununca bekleyen videoları sıraya alır.

## Sonuçlar ve ödünleşimler

- FFmpeg PATH'e ek olarak bilinen klasörlerde aranır (winget `Links` ve
  `Packages\Gyan.FFmpeg_*`, Homebrew, `/usr/bin` …); uygulama açıkken yapılan
  kurulum PATH'e yansımaz, macOS GUI uygulamaları kabuk PATH'ini görmez.
- Kodlayıcı önceliği: `libx264` → `h264_videotoolbox` → `h264_mf` →
  `libopenh264`. Donanım kodlayıcılarında CRF yerine kaynak bit hızına oranlı
  hedef bit hızı kullanılır.
- FFmpeg sürümü/derlemesi kullanıcıya göre değişir; davranış farkları olabilir.
- Uygulama kapatılırsa çalışan FFmpeg süreci sonlandırılmaz (henüz iptal yok).
