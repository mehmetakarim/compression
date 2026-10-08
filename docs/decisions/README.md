# Teknik Karar Kayıtları (ADR)

Bu dizin, projede gerçekten alınmış önemli teknik kararları tutar. Her karar
ayrı bir dosyadır.

## Ne zaman ADR yazılır?

- Teknoloji, framework, veri modeli, mimari yapı veya dış servis seçimi gibi
  geri dönmesi maliyetli kararlar alındığında.
- Kodda gözlenen bir tercih, kullanıcı onayı veya açık bir karar olmadan ADR
  olarak yazılmaz; [../architecture.md](../architecture.md) içinde gözlem olarak belgelenir.

## Dosya adı

`NNNN-kisa-karar-basligi.md` — ör. `0001-dil-secimi.md`. Numaralar sıralıdır ve
yeniden kullanılmaz.

## Şablon

```markdown
# NNNN — Karar başlığı

- Tarih: YYYY-AA-GG
- Durum: Önerildi / Kabul edildi / Yerine yeni karar alındı
- Yerine geçtiği karar: (varsa bağlantı)
- Yerine geçen karar: (varsa bağlantı)

## Bağlam ve problem

## Değerlendirilen seçenekler

## Seçilen yaklaşım ve gerekçe

## Sonuçlar ve ödünleşimler
```

## Kurallar

- Değişen bir kararın dosyası silinmez; durumu "Yerine yeni karar alındı"
  yapılır ve yeni ADR'ye bağlantı eklenir.

## Karar listesi

| No | Başlık | Durum |
|---|---|---|
| [0001](0001-tauri-masaustu-cercevesi.md) | Masaüstü çerçevesi olarak Tauri 2 | Kabul edildi |
| [0002](0002-pdf-sikistirma-motoru.md) | PDF sıkıştırma motoru: lopdf + image | Önerildi |
| [0003](0003-mp4-icin-sistem-ffmpeg.md) | MP4 için sistemdeki FFmpeg (paketlenmez) | Kabul edildi |
