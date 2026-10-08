# brain.md — Güncel Çalışma Durumu

> Kısa tutulur (≈50–100 satır). Günlük değildir; eski durumlar ilgili kayda
> taşınır. Kurallar: [AGENTS.md](AGENTS.md)

## Son güncelleme

2026-10-08 11:56 (UTC+03:00)

## Git

- Aktif branch: `main` (yerelde `git init -b main` ile oluşturuldu)
- Referans commit: Yok — henüz hiç commit yapılmadı
- Uzak repo: `origin` → <https://github.com/mehmetakarim/compression>
  (2026-10-08 itibarıyla `git ls-remote` boş döndü; uzak repoda commit yok)

## Çalışma ağacı (son kontrol)

Hafıza dosyaları dışında dosya yok. Tüm dosyalar commit edilmemiş (untracked).

## Proje durumu

Proje boş: kaynak kod, README, bağımlılık veya yapılandırma dosyası yok.
Amaç, teknoloji yığını ve mimari henüz belirlenmedi.

## Aktif hedef

Henüz aktif görev seçilmedi.

## Tamamlanan son anlamlı aşama

Git ile sürümlenebilir proje hafızası yapısı kuruldu (`AGENTS.md`, `CLAUDE.md`,
`brain.md`, `docs/`). Commit/push yapılmadı.

## Devam eden işler

Yok.

## Engeller ve açık sorunlar

- Projenin amacı ve kapsamı kullanıcı tarafından henüz tanımlanmadı.

## Son doğrulamalar

- Hafıza dosyaları arası göreli bağlantılar elle kontrol edildi; hedef dosyalar mevcut.
- Uygulama testi yok (çalıştırılacak kod yok).

## Henüz doğrulanmamış noktalar

- Uzak repoya push erişimi denenmedi.
- `CLAUDE.md` içindeki `@AGENTS.md` içe aktarımının Claude Code tarafından
  yüklendiği yeni bir session'da gözlenmedi.

## Sonraki somut adım

Kullanıcıdan projenin amacını ve ilk görevi al; bunu `docs/tasks.md` içine
`TASK-001` olarak kabul kriterleriyle kaydet. Kullanıcı isterse önce hafıza
dosyalarını ilk commit olarak `main` dalına commit edip `origin`'e push et.

## Bağlantılar

- Görevler: [docs/tasks.md](docs/tasks.md)
- Kararlar: [docs/decisions/README.md](docs/decisions/README.md)
- Çözümler: [docs/solutions.md](docs/solutions.md)
- Mimari: [docs/architecture.md](docs/architecture.md)
