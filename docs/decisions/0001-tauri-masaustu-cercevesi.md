# 0001 — Masaüstü çerçevesi olarak Tauri 2

- Tarih: 2026-10-08
- Durum: Kabul edildi
- Yerine geçtiği karar: —
- Yerine geçen karar: —

## Bağlam ve problem

Kullanıcı, dosya (öncelikle PDF) sıkıştıran, sürükle-bırak ve dosya seçme
destekli bir masaüstü mini uygulaması istedi ve uygulamanın cross-platform
olmasını belirtti.

## Değerlendirilen seçenekler

- **Python + pywebview + pikepdf/Pillow:** PDF tarafı olgun, ancak dağıtım için
  Python çalışma zamanının paketlenmesi gerekir. İlk denemede seçildi, kurulum
  başlamadan kullanıcının tercihiyle bırakıldı.
- **Electron:** Cross-platform, fakat Node tarafında PDF görsellerini yeniden
  kodlayan olgun bir kütüphane yok ve dağıtım boyutu büyük.
- **Tauri 2 (Rust + sistem WebView):** Küçük dağıtım, Windows/macOS/Linux
  desteği, sıkıştırma mantığı doğrudan Rust ile yazılabilir.

## Seçilen yaklaşım ve gerekçe

Tauri 2 (kararlı 2.x serisi; 3.0 alfa olduğu için kullanılmadı). Kullanıcı
Tauri'yi açıkça tercih etti. Arayüz, paketleyici (bundler) olmadan düz
HTML/CSS/JS olarak `ui/` altında tutulur ve `withGlobalTauri` ile
`window.__TAURI__` üzerinden API'lere erişir.

## Sonuçlar ve ödünleşimler

- Sıkıştırma motoru Rust ile yazılmalıdır (bkz. [0002](0002-pdf-sikistirma-motoru.md)).
- Windows'ta WebView2, Linux'ta WebKitGTK gerekir.
- Dosya yolları yalnızca Tauri'nin `onDragDropEvent` olayıyla alınır;
  tarayıcı `drop` olayı gerçek yolu vermez.
