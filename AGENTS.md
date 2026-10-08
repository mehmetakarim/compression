# AGENTS.md — Ortak Çalışma Kuralları ve Proje Hafızası Protokolü

Bu dosya, bu projede çalışan tüm AI geliştirme araçları (Claude Code, Codex,
Cursor vb.) ve insanlar için ortak kuralları tanımlar. Araca özgü ek
talimatlar ilgili giriş dosyasında bulunur (ör. `CLAUDE.md`).

Bu hafıza yapısı bağlamın dosyalardan yeniden kurulmasını sağlar; dosyaların
otomatik okunacağını veya her şeyin eksiksiz hatırlanacağını garanti etmez.
Aşağıdaki okuma ve güncelleme adımları bu yüzden açıkça uygulanmalıdır.

## Proje özeti

- Proje adı: `compression`
- Uzak repo: <https://github.com/mehmetakarim/compression> (`origin`)
- Amaç, kapsam, teknoloji yığını ve mimari: **Henüz belirlenmedi.**
- Güncel durum: [brain.md](brain.md) · Mimari: [docs/architecture.md](docs/architecture.md)

## Hafıza dosyaları

| Dosya | Ana kaynak olduğu bilgi |
|---|---|
| [brain.md](brain.md) | Güncel çalışma durumu, session devralma notu, sonraki adım |
| [docs/architecture.md](docs/architecture.md) | Doğrulanmış teknoloji yığını, yapı, modüller, veri akışları, entegrasyonlar |
| [docs/tasks.md](docs/tasks.md) | Görevler, öncelikler, kabul kriterleri, doğrulama ve yayın durumu |
| [docs/decisions/](docs/decisions/README.md) | Teknik karar kayıtları (ADR) |
| [docs/solutions.md](docs/solutions.md) | Tekrarlayabilecek sorunlar, kök nedenler, doğrulanmış çözümler |
| `AGENTS.md` (bu dosya) | Ortak kurallar, protokol, projeye özgü komutlar |

## Session başlangıcı

1. [brain.md](brain.md) dosyasını oku.
2. Git varsa branch'i ve çalışma ağacını kontrol et (`git status`,
   `git log --oneline -10`). Kullanıcının commit edilmemiş değişikliklerini not et.
3. Aktif görevle ilgili kodu ve yalnızca gerekli hafıza belgelerini incele
   (ilgili `TASK-xxx`, ADR, `BUG-xxx`). Tüm tarihsel kayıtları her session'da okuma.
4. Hafızadaki durum iddialarını (ör. "tamamlandı", "testler geçiyor") kod ve
   Git ile doğrula.
5. Çelişki varsa kullanıcıya açıkça belirt. Amaçlanan davranışı yalnızca mevcut
   uygulamadan çıkarma; kullanıcı talebini ve kabul kriterlerini esas al.

## Çalışma sırasında

- Görevin kapsamını ve kabul kriterlerini netleştir; `docs/tasks.md` içinde kaydet.
- Varsayım, gözlem ve doğrulanmış sonucu birbirinden ayırarak yaz.
- Kullanıcının mevcut değişikliklerini koru; ilgisiz kod değişikliği yapma.
- Anlamlı bir aşama tamamlandığında `brain.md` ve ilgili görev kaydını güncelle
  (yalnızca session sonunda değil).
- Önemli teknik kararları gerekçeleriyle ADR olarak kaydet
  ([docs/decisions/README.md](docs/decisions/README.md)).
- Tekrarlanabilecek sorunlarda doğrulanmış çözümü ve işe yaramayan önemli
  denemeleri [docs/solutions.md](docs/solutions.md) içine kaydet.

## Görev tamamlandığında

- Değişiklikle ilgili mevcut doğrulama yöntemlerini (test, lint, build, manuel
  kontrol) uygula.
- Çalıştırılan kontrolü, sonucunu ve doğrulanmayan noktaları kaydet.
- **Kodlandı**, **doğrulandı** ve **yayına alındı** durumlarını birbirine karıştırma.
- Görev durumunu (`docs/tasks.md`) ve `brain.md` dosyasını güncelle.
- Sonraki session için somut bir ilk adım bırak (dosya + işlem/kontrol + amaç).
- Hafıza kurulumu veya güncellemesi kendiliğinden commit, push ya da deployment
  yetkisi vermez; bunlar için kullanıcının açık talebi gerekir.

## Projeye özgü komutlar

Proje henüz boş olduğu için doğrulanmış komut yoktur. Bir komut eklenirken
hangi dizinde çalıştığı, ön koşulları ve **dosyada tanımlı** mı yoksa
**başarıyla çalıştırıldı** mı olduğu ayrı belirtilmelidir.

| Amaç | Komut | Dizin | Ön koşul | Durum |
|---|---|---|---|---|
| Kurulum | Henüz belirlenmedi | — | — | Tanımlı değil |
| Geliştirme | Henüz belirlenmedi | — | — | Tanımlı değil |
| Test | Henüz belirlenmedi | — | — | Tanımlı değil |
| Lint | Henüz belirlenmedi | — | — | Tanımlı değil |
| Build | Henüz belirlenmedi | — | — | Tanımlı değil |

## Kayıt formatları

- Görev kaydı formatı: [docs/tasks.md](docs/tasks.md) başındaki şablon.
- Karar kaydı (ADR) formatı: [docs/decisions/README.md](docs/decisions/README.md).
- Çözüm kaydı formatı: [docs/solutions.md](docs/solutions.md) başındaki şablon.
- Kodda gözlenen bir tercihi geçmişte alınmış ve onaylanmış karar gibi sunma;
  gözlenen mimariyi `docs/architecture.md` içinde "gözlem" olarak belgele.

## Hafıza bakım kuralları

- Her bilginin tek bir ana kaynağı olsun (yukarıdaki tablo); diğer dosyalardan
  bağlantı ver, tekrar etme.
- `brain.md` içindeki eski durumları kaldır veya ilgili görev/çözüm/karar
  kaydına taşı. `brain.md` yaklaşık 50–100 satır kalmalı; günlük değildir.
- Tamamlanan görevler büyüdüğünde `docs/tasks-archive.md` dosyasına taşı ve
  `docs/tasks.md` içinde bağlantı bırak.
- Değişen kararların geçmişini silme; eski ADR'nin durumunu
  "Yerine yeni karar alındı" yap ve yeni ADR'ye bağla.
- Gereksiz log, uzun kod bloğu ve konuşma dökümü saklama.
- Parola, token, API anahtarı veya kişisel veri yazma.
- Proje köküne göre göreli yollar kullan.
- Belgeleri Türkçe yaz; kod, komut ve teknik tanımlayıcıları olduğu gibi koru.
- Bu dosyalar Git ile sürümlenir; `.gitignore` eklenirse bu dosyaları
  dışlamadığından emin ol.
- Sahte görev, hata, karar veya tamamlanmış iş üretme; bilinmeyeni
  "Henüz belirlenmedi" / "Bilinmiyor" olarak işaretle.
