# 0002 — PDF sıkıştırma motoru: lopdf + image (saf Rust)

- Tarih: 2026-10-08
- Durum: Önerildi (kullanıcı onayı bekleniyor)
- Yerine geçtiği karar: —
- Yerine geçen karar: —

## Bağlam ve problem

PDF boyutunun büyük kısmı gömülü görsellerden gelir. Anlamlı sıkıştırma için
görsellerin küçültülüp yeniden kodlanması gerekir. Uygulama Tauri ile
([0001](0001-tauri-masaustu-cercevesi.md)) cross-platform dağıtılacak.

## Değerlendirilen seçenekler

- **Ghostscript (`gs -sDEVICE=pdfwrite`):** En güçlü sonuçlar; ancak AGPL
  lisanslı, ayrı ikili dosya olarak her platform için paketlenmesi gerekir.
  Makinede kurulu değil.
- **qpdf/pikepdf:** Yalnızca kayıpsız yapısal iyileştirme; görsel yeniden
  kodlama için ek katman gerekir, Rust'tan doğrudan kullanılamaz.
- **lopdf + image (saf Rust):** Ek ikili dosya yok, MIT lisanslı; görsel
  işleme mantığı uygulama içinde yazılır.

## Seçilen yaklaşım ve gerekçe

`lopdf` 0.45 ile PDF ayrıştırılır; `image` 0.25 ile gri/RGB, 8-bit,
`DCTDecode`/`FlateDecode`/filtresiz görseller seviyeye göre küçültülüp JPEG
olarak yeniden kodlanır. Görsel olmayan Flate akışları en yüksek seviyede
yeniden sıkıştırılır, dosya nesne akışları ve xref akışıyla kaydedilir.
Çıktı yeniden ayrıştırılarak doğrulanır; orijinalden küçük değilse yazılmaz.

## Sonuçlar ve ödünleşimler

- Desteklenmeyenler (dokunulmadan bırakılır): CMYK/Indexed görseller,
  8-bit dışı görseller, `/Decode` dizisi olan görseller, yumuşak maskeler
  (`SMask`), JPX/JBIG2/CCITT filtreleri. Şifreli PDF'ler reddedilir.
- Metin ağırlıklı PDF'lerde kazanç düşüktür (gözlem: ~%7–8).
- Flate ile saklanan keskin grafikler (ekran görüntüsü, çizim) JPEG'e
  çevrilince kenarlarda bozulma görülebilir; henüz ayrı bir kural yok.
- Ghostscript seviyesinde sonuç gerekirse bu karar yeniden değerlendirilebilir.
