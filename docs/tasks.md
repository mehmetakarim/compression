# Görevler

> Görev durumu, kabul kriterleri ve doğrulama sonuçlarının ana kaynağı.
> Tamamlanan görevler çoğaldığında `docs/tasks-archive.md` dosyasına taşınır.

## Kayıt şablonu

```markdown
### TASK-000 — Başlık
- Öncelik: Yüksek / Orta / Düşük
- Durum: Bekliyor / Devam ediyor / Engellendi / Tamamlandı / İptal
- Amaç ve kapsam:
- Kabul kriterleri:
  - [ ] ...
- Bağımlılıklar / engeller:
- İlgili dosyalar:
- Doğrulama sonucu: (çalıştırılan kontrol, sonuç, doğrulanmayan noktalar)
- Yayın durumu: Bilinmiyor / Yayınlanmadı / Yayınlandı / Uygulanamaz
```

Kimlikler sırayla verilir (`TASK-001`, `TASK-002`, …) ve yeniden kullanılmaz.

## Aktif görevler

### TASK-001 — PDF sıkıştıran masaüstü mini uygulaması (MVP)
- Öncelik: Yüksek
- Durum: Devam ediyor (kodlandı; kullanıcı kabul testi bekleniyor)
- Amaç ve kapsam: Sürükle-bırak veya dosya seçme ile PDF sıkıştıran,
  cross-platform Tauri uygulaması. Arayüz kullanıcının paylaştığı "Upload
  files" kart tasarımını örnek alır. Diğer dosya türleri kapsam dışı.
- Kabul kriterleri:
  - [x] Dosya bırakma alanı ve "Dosya Seç" düğmesi (çoklu seçim)
  - [x] PDF'ler sıkıştırılıp orijinalin yanına `-compressed` ekiyle yazılır;
        orijinal değiştirilmez
  - [x] Hafif / Dengeli / Güçlü seviye seçimi
  - [x] Listede önceki/sonraki boyut, kazanç yüzdesi, hata ve "küçültülemedi" durumu
  - [x] "Klasörde göster" ve listeden kaldırma
  - [x] Sıkıştırma sırasında satır altında aşama ve yüzde gösteren ilerleme
        çubuğu (kullanıcı isteği, 2026-10-08)
  - [x] Footer imzası: "Bu uygulama Mehmet Akar tarafından geliştirilmiştir.";
        isim <https://www.mehmetakar.com.tr> adresine sistem tarayıcısında
        açılan link (kullanıcı isteği, 2026-10-08)
  - [x] Kullanıcı gerçek pencerede taranmış PDF denedi (16 MB → 3,6 MB);
        ilk denemedeki yavaşlık [BUG-003](solutions.md) ile giderildi
  - [ ] Windows dışı platformlarda derleme denendi
- Bağımlılıklar / engeller: macOS/Linux makinesi yok.
- İlgili dosyalar: `ui/`, `src-tauri/src/pdf.rs`, `src-tauri/src/lib.rs`,
  `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`
- Doğrulama sonucu (2026-10-08, Windows 11):
  - `cargo test`: 4 test geçti (sentetik PDF küçültme/ölçekleme, ilerleme
    sırası, seviye sıralaması, çıktı adının üzerine yazmaması).
  - `cargo clippy --all-targets`: uyarı yok.
  - `real_samples` (ignored test): Edge ile üretilmiş 3 sayfalık fotoğraflı
    PDF 2.950.978 B → 869.534 (Hafif) / 427.259 (Dengeli) / 204.114 (Güçlü);
    6 sayfalık metin ağırlıklı PDF 268.160 B → ~247 KB. Sayfa sayıları korundu;
    tüm çıktılar PDFium ile hatasız render edildi, Güçlü seviyede görsel olarak
    kayda değer bozulma gözlenmedi.
  - Arayüz, Tauri API'leri taklit edilerek tarayıcıda açık/koyu temada
    kontrol edildi.
  - `npm run dev` penceresinde CDP ile: 31 MB sentetik tarama 4,4 sn'de
    2,5 MB; ilerleme çubuğu ekran görüntüsüyle doğrulandı.
  - Kullanıcı yerel pencerede kendi taramasını sıkıştırdı (sonuç geldi).
    Sürükle-bırak mı dosya seçici mi kullanıldığı ve "Klasörde göster"
    düğmesi ayrıca doğrulanmadı.
  - `npm run build` (yükleyici üretimi) çalıştırılmadı.
- Yayın durumu: Yayınlanmadı

### TASK-002 — PNG, JPG/JPEG, WebP ve MP4 desteği
- Öncelik: Yüksek
- Durum: Devam ediyor (kodlandı; kullanıcı kabul testi bekleniyor)
- Amaç ve kapsam: Mevcut uygulamaya görsel ve MP4 sıkıştırma eklemek. MP4
  için sistemdeki FFmpeg kullanılır; yoksa kurulum yardımı sunulur
  ([ADR 0003](decisions/0003-mp4-icin-sistem-ffmpeg.md)).
- Kabul kriterleri:
  - [x] JPG/JPEG, PNG, WebP, MP4 seçilebilir/bırakılabilir; diğerleri
        "Desteklenmeyen dosya türü"
  - [x] Üç seviye her türde anlamlı farklılık gösterir
  - [x] FFmpeg yoksa uyarı + "FFmpeg'i yükle" / "Tekrar kontrol et";
        kurulunca bekleyen videolar otomatik işlenir
  - [x] Video için ilerleme çubuğu
  - [ ] Düzeltilmiş kurulum penceresiyle gerçek winget kurulumu kullanıcı
        tarafından görüldü
  - [ ] macOS/Linux kurulum yardımı denendi
- İlgili dosyalar: `src-tauri/src/{common,raster,video,lib}.rs`, `ui/`,
  `src-tauri/capabilities/default.json`
- Doğrulama sonucu (2026-10-08, Windows 11):
  - `cargo test`: 12 test geçti (raster 6, video ayrıştırma 2, pdf 3,
    common 1); `cargo clippy --all-targets` uyarısız. Elle çalıştırılan
    `new_console_runs_command` (ignored) geçti.
  - Uygulamada (CDP ile, Dengeli): JPG 529 KB → 192 KB, yüksek kaliteli JPG
    701 → 214 KB, WebP 250 → 51 KB, PNG 189 → 167 KB ve 113 → 76 KB
    (kayıpsız), `.txt` reddedildi. Çıktılar görsel olarak kontrol edildi.
  - MP4 (sistemdeki FFmpeg 8.1.1, libx264): 1080p 12 sn 18,7 MB → 5,9 MB
    (Dengeli, 1080p korunur) ve 0,9 MB (Güçlü, 1280×720); dikey 1080×1920 →
    720×1280; sessiz video işlendi; süre ve ses akışı ffprobe ile doğrulandı.
  - FFmpeg yok → kuruluyor → hazır akışı, IPC yanıtları CDP `Fetch` ile
    taklit edilerek ekran görüntüleriyle doğrulandı.
  - Test sırasında yanlışlıkla gerçek `install_ffmpeg` çalıştı ve kullanıcının
    winget'le kurulu Gyan.FFmpeg paketini 9.0.2'ye yükseltti; aynı çalıştırma
    pencere hatasını ortaya çıkardı ([BUG-004](solutions.md)).
- Yayın durumu: Yayınlanmadı

### TASK-003 — Sekmeler: Sıkıştırma ve Dönüştür (PNG ↔ WebP)
- Öncelik: Yüksek
- Durum: Devam ediyor (kodlandı; kullanıcı kabul testi bekleniyor)
- Amaç ve kapsam: Mevcut ekran "Sıkıştırma" sekmesi olur; yeni "Dönüştür"
  sekmesinde şimdilik yalnızca PNG → WebP ve WebP → PNG.
- Kabul kriterleri:
  - [x] Üstte "Sıkıştırma" ve "Dönüştür" sekmeleri; mevcut işlevler korunur
  - [x] Dönüştür: PNG → WebP / WebP → PNG seçimi, bırakma alanı, dosya seçici
        (seçili türe filtreli), ilerleme ve sonuç satırları
  - [x] PNG → WebP için "Kayıpsız" seçeneği (varsayılan açık; kapalıyken q90)
  - [x] Yanlış türde dosya açıklayıcı hatayla reddedilir; var olan dosyanın
        üzerine yazılmaz
- İlgili dosyalar: `src-tauri/src/convert.rs`, `src-tauri/src/lib.rs`,
  `src-tauri/src/common.rs` (`unique_sibling`), `ui/`
- Doğrulama sonucu (2026-10-08, Windows 11):
  - `cargo test`: 16 test geçti (convert 4 yeni: kayıpsız piksel eşliği +
    alfa, kayıplı geçerlilik, WebP → PNG piksel eşliği, yanlış tür ve üzerine
    yazmama); clippy uyarısız.
  - Uygulamada (CDP): PNG → WebP kayıpsız 189 → 129 KB ve 113 → 36 KB,
    kayıplı 189 → 28 KB (`page (2).webp`, mevcut dosya korundu); WebP → PNG
    250 KB → 1,7 MB (fotoğraf içerik, beklenen); JPG reddedildi. Sekmeler ve
    seçenekler ekran görüntüsüyle kontrol edildi; sıkıştırma sekmesi
    regresyonu (JPG, PDF) geçti.
  - Klavye ile sekme gezinme (ok tuşları) ve gerçek sürükle-bırak elle
    denenmedi.
- Yayın durumu: Yayınlanmadı

### TASK-004 — GitHub Release: Windows .exe ve macOS .dmg
- Öncelik: Yüksek
- Durum: Devam ediyor
- Amaç ve kapsam: `v*` etiketiyle GitHub Actions üzerinde Windows (NSIS
  `.exe`) ve macOS (universal `.dmg`) yükleyicilerini derleyip GitHub
  Release'e yüklemek. Kod imzalama kapsam dışı.
- Kabul kriterleri:
  - [x] `.github/workflows/release.yml` (tauri-action v1, taslak release)
  - [x] Yerel Windows release derlemesi ve duman testi
  - [ ] CI'da iki platform da başarılı, iki yükleyici release'e yüklendi
  - [ ] Release yayınlandı (taslaktan çıkarıldı)
- İlgili dosyalar: `.github/workflows/release.yml`, `src-tauri/tauri.conf.json`
- Doğrulama sonucu (2026-10-08):
  - `npx tauri build --bundles nsis` yerelde başarılı:
    `Compression_0.1.0_x64-setup.exe` (2,73 MiB), release derlemesi 3 dk 49 sn.
  - Release `compression.exe` tek başına çalıştırıldı (CDP): arayüz
    `http://tauri.localhost/`'tan yüklendi, `ffmpeg_status` ve JPG
    sıkıştırma çalıştı. Geliştirme penceresi açıkken ikinci örnek
    WebView2 `0x8007139F` ile açılmadı ([BUG-005](solutions.md)).
  - macOS derlemesi yerelde yapılamaz; yalnızca CI'da doğrulanacak.
- Yayın durumu: Yayınlanmadı

## Tamamlanan görevler

Yok.
