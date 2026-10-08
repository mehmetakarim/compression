# Sorunlar ve Çözümler

> Tekrarlama ihtimali olan sorunların, kök nedenlerin ve doğrulanmış
> çözümlerin ana kaynağı. Uzun log veya kod bloğu yerine özet ve dosya
> bağlantısı kullanılır.

## Kayıt şablonu

```markdown
### BUG-000 — Kısa başlık
- Belirti ve ortaya çıktığı koşul:
- Kök neden: (doğrulanmadıysa "Doğrulanmadı" yaz)
- İşe yaramayan önemli denemeler ve nedenleri:
- Uygulanan çözüm:
- Doğrulama: (nasıl doğrulandı)
- İlgili dosya / görev / commit:
```

Kimlikler sırayla verilir (`BUG-001`, `BUG-002`, …) ve yeniden kullanılmaz.

## Kayıtlar

### BUG-001 — `hidden` niteliğine rağmen düğme görünüyor
- Belirti: "Klasörde göster" düğmesi tüm satırlarda görünüyordu.
- Kök neden: `.icon-button { display: grid }` kuralı tarayıcının `[hidden]`
  varsayılan stilini eziyor (doğrulandı).
- Uygulanan çözüm: `ui/styles.css` içine `.icon-button[hidden] { display: none; }`.
- Doğrulama: Tarayıcıda `getComputedStyle` ile yalnızca başarılı satırda `grid`.
- İlgili: TASK-001. `display` atanan başka sınıflarda `hidden` kullanılırsa
  aynı kural eklenmeli.

### BUG-002 — Edge headless `--print-to-pdf` dosya üretmiyor
- Belirti: Test PDF'i üretmek için çalıştırılan `msedge --headless
  --print-to-pdf=...` hata vermeden çıktı üretmedi.
- Kök neden: Doğrulanmadı (açık Edge oturumunun profil kilidi olası).
- Uygulanan çözüm: `--headless=new --user-data-dir=<geçici klasör>
  --no-first-run` eklenince PDF üretildi.
- Doğrulama: 2,9 MB'lık 3 sayfalık PDF oluştu.
- İlgili: TASK-001 test verisi.

### BUG-003 — `npm run dev` ile büyük PDF'ler dakikalarca sürüyor
- Belirti: 16,7 MB'lık taranmış PDF `tauri dev` penceresinde ~5 dakika
  "Sıkıştırılıyor…" durumunda kaldı; kullanıcı uygulamanın takıldığını düşündü.
- Kök neden (doğrulandı): Debug profilinde `image`/`lopdf`/`flate2`
  optimizasyonsuz derleniyordu. Aynı 3 sayfalık PDF debug'da 25,6 sn,
  release'te ~1 sn sürdü. Sonuç arayüze ulaşıyordu; çıktı dosyası yazılmıştı.
- İşe yaramayan / dışlanan: Uzun IPC çağrısının düştüğü şüphesi — CDP ile
  ölçülünce `invoke` sonucunun döndüğü görüldü.
- Uygulanan çözüm: `src-tauri/Cargo.toml` içinde
  `[profile.dev.package."*"] opt-level = 3`; küçültme filtresi Lanczos3 →
  Triangle; satır altına aşama/yüzde gösteren ilerleme çubuğu (`Channel`).
- Doğrulama: 8 sayfalık 31 MB sentetik tarama dev modda 4,4 sn; ilerleme
  olayları %0→%100 monoton geldi. Kullanıcının taraması 16 MB → 3,6 MB.
- Not: Uygulama içi webview'i incelemek için
  `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9333 --remote-allow-origins=*"`
  ile başlatıp CDP (`http://127.0.0.1:9333/json`) kullanılabilir (yalnızca Windows/WebView2).
- İlgili: TASK-001.

### BUG-004 — winget kurulumu ayrı pencerede görünmüyor
- Belirti: `install_ffmpeg` çalışınca kurulum çıktısı yeni konsol penceresi
  yerine uygulamanın (dev'de `tauri dev`'in) standart çıktısına yazıldı;
  `pause` görünmeyen bir girişte bekledi.
- Kök neden (doğrulandı): `CREATE_NEW_CONSOLE` yeni konsol açsa da Rust
  `Command` alt sürece üst sürecin stdin/stdout/stderr'ini devrettiği için
  çıktı yeni pencereye gitmez.
- Uygulanan çözüm: `cmd /C start "<başlık>" cmd /C "<komut> & pause"`
  (`CREATE_NO_WINDOW` + `Stdio::null()`); `start` bağımsız, kendi konsolu
  olan bir süreç başlatır (`src-tauri/src/video.rs`, `spawn_in_new_console`).
- Doğrulama: Elle çalıştırılan `new_console_runs_command` testi zararsız bir
  komutla geçti. Gerçek winget ile yeni form henüz çalıştırılmadı.
- Ders: Test sırasında taklit devreye girmediyse yan etkili düğmelere
  (`ffmpeg-install`) tıklanmamalı; önce taklidin etkin olduğu doğrulanmalı.
  Tauri'de `__TAURI_INTERNALS__` salt okunurdur; IPC taklidi CDP `Fetch`
  ile `ipc.localhost` isteklerinde yapılabilir.
- İlgili: TASK-002.

### BUG-005 — İkinci örnek açılırken WebView2 0x8007139F
- Belirti: `tauri dev` penceresi açıkken release `compression.exe`
  başlatılınca "failed to create webview: WebView2 error ... 0x8007139F"
  ile panik.
- Kök neden: İki örnek aynı tanımlayıcıyı (`com.mehmetakarim.compression`),
  dolayısıyla aynı WebView2 kullanıcı veri klasörünü farklı tarayıcı
  argümanlarıyla (`--remote-debugging-port` 9333 / 9334) açmaya çalıştı;
  WebView2 aynı klasörde farklı ortam seçeneklerine izin vermez. Dev
  kapatılınca release örneği sorunsuz açıldı (doğrulandı).
- Uygulanan çözüm: Test sırasında tek örnek çalıştırmak. Son kullanıcıda
  yalnızca aynı anda farklı argümanlarla iki örnek açılırsa görülür.
- İlgili: TASK-004.
