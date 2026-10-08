//! MP4 sıkıştırma: sistemde kurulu FFmpeg kullanılır, uygulamayla paketlenmez
//! (bkz. docs/decisions/0003-mp4-icin-sistem-ffmpeg.md).
//!
//! FFmpeg yoksa arayüz `ffmpeg_status` ile bunu görür ve `install_ffmpeg` ile
//! platforma uygun kurulumu başlatır (Windows: winget, macOS: Homebrew).

use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::common::{file_size, unique_output_path, Level, Outcome, Progress, Stage};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const EXE: &str = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallMethod {
    Winget,
    Homebrew,
    Manual,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegStatus {
    pub available: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub install_method: InstallMethod,
    /// Elle kurulum için gösterilecek komut (Linux / Homebrew'suz macOS).
    pub manual_command: Option<String>,
}

pub fn status() -> FfmpegStatus {
    let found = find_ffmpeg();
    FfmpegStatus {
        available: found.is_some(),
        path: found.as_ref().map(|(p, _)| p.to_string_lossy().into_owned()),
        version: found.map(|(_, v)| v),
        install_method: install_method(),
        manual_command: manual_command(),
    }
}

/// Kurulumu ayrı, görünür bir pencerede başlatır ve hemen döner. Arayüz
/// kurulumun bittiğini `status` ile yoklayarak anlar.
pub fn start_install() -> Result<InstallMethod, String> {
    let method = install_method();
    match method {
        InstallMethod::Winget => {
            #[cfg(windows)]
            spawn_in_new_console("FFmpeg kurulumu", WINGET_INSTALL)?;
        }
        InstallMethod::Homebrew => {
            let brew = homebrew().ok_or("Homebrew bulunamadı.")?;
            let script = format!(
                "tell application \"Terminal\"\nactivate\ndo script \"{} install ffmpeg\"\nend tell",
                brew.display()
            );
            Command::new("osascript")
                .args(["-e", &script])
                .spawn()
                .map_err(|e| format!("Terminal açılamadı: {e}"))?;
        }
        InstallMethod::Manual => {}
    }
    Ok(method)
}

#[cfg(windows)]
const WINGET_INSTALL: &str = "winget install --id Gyan.FFmpeg --exact --source winget";

/// Komutu kendi konsol penceresinde çalıştırır; `pause` kullanıcı sonucu
/// okuyabilsin diye pencereyi açık tutar. `CREATE_NEW_CONSOLE` tek başına
/// yetmez: Rust alt sürece üst sürecin standart akışlarını verdiği için çıktı
/// yeni pencereye değil uygulamaya gider. `start` ise yeni konsolun kendi
/// akışlarıyla bağımsız bir süreç başlatır.
#[cfg(windows)]
fn spawn_in_new_console(title: &str, command: &str) -> Result<(), String> {
    Command::new("cmd")
        .raw_arg(format!("/C start \"{title}\" cmd /C \"{command} & pause\""))
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Kurulum penceresi açılamadı: {e}"))?;
    Ok(())
}

fn install_method() -> InstallMethod {
    if cfg!(windows) && which_in_path(if cfg!(windows) { "winget.exe" } else { "winget" }).is_some() {
        InstallMethod::Winget
    } else if cfg!(target_os = "macos") && homebrew().is_some() {
        InstallMethod::Homebrew
    } else {
        InstallMethod::Manual
    }
}

fn manual_command() -> Option<String> {
    if cfg!(target_os = "linux") {
        Some("sudo apt install ffmpeg   (Fedora: sudo dnf install ffmpeg)".into())
    } else if cfg!(target_os = "macos") && homebrew().is_none() {
        Some("brew install ffmpeg".into())
    } else {
        None
    }
}

fn homebrew() -> Option<PathBuf> {
    ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.is_file())
}

fn which_in_path(exe: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|dir| dir.join(exe))
        .find(|p| p.is_file())
}

/// FFmpeg'i önce PATH'te, sonra yaygın kurulum yerlerinde arar. Uygulama
/// açıkken yapılan kurulumlar PATH'e yansımadığı ve macOS'ta GUI uygulamaları
/// kabuk PATH'ini görmediği için bilinen klasörlere de bakılır.
fn find_ffmpeg() -> Option<(PathBuf, String)> {
    let mut candidates: Vec<PathBuf> = which_in_path(EXE).into_iter().collect();
    if cfg!(windows) {
        if let Some(local) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
            candidates.push(local.join(r"Microsoft\WinGet\Links").join(EXE));
            // winget'in kullanıcı kapsamındaki paket klasörü:
            // Packages\Gyan.FFmpeg_*\ffmpeg-*\bin\ffmpeg.exe
            candidates.extend(glob_ffmpeg_in(&local.join(r"Microsoft\WinGet\Packages"), "Gyan.FFmpeg"));
        }
        for base in ["ProgramFiles", "ProgramData"] {
            if let Some(dir) = env::var_os(base).map(PathBuf::from) {
                candidates.push(dir.join(r"ffmpeg\bin").join(EXE));
            }
        }
        candidates.push(PathBuf::from(r"C:\ffmpeg\bin").join(EXE));
    } else {
        for dir in ["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin", "/usr/bin", "/snap/bin"] {
            candidates.push(Path::new(dir).join(EXE));
        }
    }
    candidates
        .into_iter()
        .filter(|p| p.is_file())
        .find_map(|p| version_of(&p).map(|v| (p, v)))
}

fn glob_ffmpeg_in(packages: &Path, prefix: &str) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(packages) else { return Vec::new() };
    entries
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(prefix))
        .flat_map(|pkg| fs::read_dir(pkg.path()).into_iter().flatten().flatten())
        .map(|build| build.path().join("bin").join(EXE))
        .collect()
}

fn quiet(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// `ffmpeg -version` çıktısının ilk satırından sürümü okur; çalışmıyorsa `None`.
fn version_of(ffmpeg: &Path) -> Option<String> {
    let output = quiet(Command::new(ffmpeg).arg("-version")).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let first = text.lines().next()?;
    Some(first.split_whitespace().nth(2).unwrap_or(first).to_string())
}

struct Settings {
    crf: u8,
    /// Kısa kenar üst sınırı (ör. 1080p); `None` ise çözünürlük korunur.
    max_short_side: Option<u32>,
    audio_kbps: u32,
    /// Donanım kodlayıcılarında CRF olmadığından kaynak bit hızına çarpan.
    bitrate_factor: f64,
}

fn settings(level: Level) -> Settings {
    match level {
        Level::Light => Settings { crf: 23, max_short_side: None, audio_kbps: 160, bitrate_factor: 0.7 },
        Level::Balanced => Settings { crf: 26, max_short_side: Some(1080), audio_kbps: 128, bitrate_factor: 0.5 },
        Level::Strong => Settings { crf: 30, max_short_side: Some(720), audio_kbps: 96, bitrate_factor: 0.3 },
    }
}

/// libx264 yoksa platformun donanım/sistem H.264 kodlayıcısına düşülür.
fn pick_encoder(ffmpeg: &Path) -> Result<&'static str, String> {
    let output = quiet(Command::new(ffmpeg).args(["-hide_banner", "-encoders"]))
        .output()
        .map_err(|e| format!("FFmpeg çalıştırılamadı: {e}"))?;
    let list = String::from_utf8_lossy(&output.stdout);
    let has = |name: &str| list.lines().any(|l| l.split_whitespace().nth(1) == Some(name));
    ["libx264", "h264_videotoolbox", "h264_mf", "libopenh264"]
        .into_iter()
        .find(|e| has(e))
        .ok_or_else(|| "FFmpeg'te H.264 kodlayıcısı bulunamadı.".to_string())
}

pub fn compress_file(input: &Path, level: Level, report: &mut dyn FnMut(Progress)) -> Result<Outcome, String> {
    let (ffmpeg, _) = find_ffmpeg().ok_or("FFmpeg bulunamadı. MP4 için FFmpeg kurulmalı.")?;
    let original_size = file_size(input)?;
    report(Progress::new(0, Stage::Reading));

    let encoder = pick_encoder(&ffmpeg)?;
    let probe = probe(&ffmpeg, input)?;
    let s = settings(level);

    let output = unique_output_path(input);
    let partial = output.with_extension("mp4.part");

    let mut args: Vec<String> = ["-hide_banner", "-nostdin", "-y", "-i"].map(String::from).to_vec();
    args.push(input.to_string_lossy().into_owned());
    args.extend(["-map", "0:v:0", "-map", "0:a?", "-c:v", encoder].map(String::from));
    if encoder == "libx264" {
        args.extend(["-preset", "medium", "-crf", &s.crf.to_string()].map(String::from));
    } else {
        let source_kbps = probe.video_kbps.unwrap_or(4000);
        let target = ((source_kbps as f64 * s.bitrate_factor) as u32).max(300);
        args.extend(["-b:v".into(), format!("{target}k")]);
    }
    if let Some(max) = s.max_short_side {
        // Yalnızca küçültür; yatay ve dikey videoda kısa kenarı sınırlar.
        // Yükseklik/genişlik çift sayı olmalı (yuv420p).
        args.extend([
            "-vf".into(),
            format!(
                "scale='if(gte(iw,ih),-2,trunc(min({max},iw)/2)*2)':'if(gte(iw,ih),trunc(min({max},ih)/2)*2,-2)'"
            ),
        ]);
    }
    args.extend(
        [
            "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", &format!("{}k", s.audio_kbps),
            "-movflags", "+faststart", "-map_metadata", "0", "-progress", "pipe:1", "-nostats", "-f", "mp4",
        ]
        .map(String::from),
    );
    args.push(partial.to_string_lossy().into_owned());

    let mut child = quiet(Command::new(&ffmpeg).args(&args))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("FFmpeg başlatılamadı: {e}"))?;

    // stderr ayrı iş parçacığında okunur; aksi hâlde tampon dolup FFmpeg bekler.
    let stderr_text = Arc::new(Mutex::new(String::new()));
    let stderr_thread = {
        let mut stderr = child.stderr.take().unwrap();
        let sink = Arc::clone(&stderr_text);
        std::thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            *sink.lock().unwrap() = text;
        })
    };

    report(Progress::new(2, Stage::Video));
    let mut last = 2u8;
    for line in BufReader::new(child.stdout.take().unwrap()).lines().map_while(Result::ok) {
        // `out_time_us` mikro saniyedir (eski sürümlerde `out_time_ms` adıyla da aynı birim).
        let Some(value) = line.strip_prefix("out_time_us=").or_else(|| line.strip_prefix("out_time_ms=")) else {
            continue;
        };
        if let (Ok(us), Some(duration)) = (value.trim().parse::<f64>(), probe.duration_secs) {
            let percent = (2.0 + 96.0 * (us / 1e6 / duration).clamp(0.0, 1.0)) as u8;
            if percent > last {
                last = percent;
                report(Progress::new(percent, Stage::Video));
            }
        }
    }

    let status = child.wait().map_err(|e| format!("FFmpeg beklenemedi: {e}"))?;
    let _ = stderr_thread.join();
    if !status.success() {
        let _ = fs::remove_file(&partial);
        let text = stderr_text.lock().unwrap();
        let tail: Vec<&str> = text.lines().rev().filter(|l| !l.trim().is_empty()).take(2).collect();
        return Err(format!(
            "FFmpeg hata verdi: {}",
            tail.into_iter().rev().collect::<Vec<_>>().join(" ")
        ));
    }

    report(Progress::new(99, Stage::Saving));
    let compressed_size = file_size(&partial)?;
    if compressed_size >= original_size {
        let _ = fs::remove_file(&partial);
        return Ok(Outcome::not_smaller(original_size));
    }
    fs::rename(&partial, &output).map_err(|e| format!("Çıktı yazılamadı: {e}"))?;
    report(Progress::new(100, Stage::Saving));
    Ok(Outcome {
        output_path: Some(output.to_string_lossy().into_owned()),
        original_size,
        compressed_size,
    })
}

struct Probe {
    duration_secs: Option<f64>,
    video_kbps: Option<u32>,
}

/// `ffmpeg -i` çıktısından süreyi ve video bit hızını okur (ffprobe her
/// kurulumda bulunmayabilir).
fn probe(ffmpeg: &Path, input: &Path) -> Result<Probe, String> {
    let output = quiet(Command::new(ffmpeg).args(["-hide_banner", "-i"]).arg(input))
        .output()
        .map_err(|e| format!("FFmpeg çalıştırılamadı: {e}"))?;
    let text = String::from_utf8_lossy(&output.stderr);
    if !text.contains("Video:") {
        return Err("Dosyada video akışı bulunamadı.".into());
    }
    Ok(parse_probe(&text))
}

fn parse_probe(text: &str) -> Probe {
    let duration_secs = text
        .split("Duration: ")
        .nth(1)
        .and_then(|rest| rest.split(',').next())
        .and_then(|hms| {
            let parts: Vec<f64> = hms.trim().split(':').filter_map(|p| p.parse().ok()).collect();
            (parts.len() == 3).then(|| parts[0] * 3600.0 + parts[1] * 60.0 + parts[2])
        })
        .filter(|d| *d > 0.0);
    let video_kbps = text
        .lines()
        .find(|l| l.contains("Video:"))
        .and_then(|l| l.split(" kb/s").next())
        .and_then(|l| l.rsplit([' ', ',']).next())
        .and_then(|n| n.parse().ok());
    Probe { duration_secs, video_kbps }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'a.mp4':
  Duration: 00:01:02.50, start: 0.000000, bitrate: 5210 kb/s
  Stream #0:0[0x1](und): Video: h264 (High) (avc1 / 0x31637661), yuv420p(progressive), 1920x1080, 5000 kb/s, 30 fps, 30 tbr
  Stream #0:1[0x2](und): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 192 kb/s";

    #[test]
    fn parses_duration_and_bitrate() {
        let probe = parse_probe(SAMPLE);
        assert_eq!(probe.duration_secs, Some(62.5));
        assert_eq!(probe.video_kbps, Some(5000));
    }

    /// Elle: `cargo test new_console -- --ignored`. Kısa süre bir konsol
    /// penceresi açılır; komutun ayrı konsolda çalıştığını doğrular.
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn new_console_runs_command() {
        let marker = env::temp_dir().join(format!("compression-console-{}.txt", std::process::id()));
        let _ = fs::remove_file(&marker);
        spawn_in_new_console("test", &format!("echo ok> \"{}\" & exit", marker.display())).unwrap();
        for _ in 0..50 {
            if marker.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert_eq!(fs::read_to_string(&marker).unwrap().trim(), "ok");
        fs::remove_file(&marker).unwrap();
    }

    #[test]
    fn missing_values_are_none() {
        let probe = parse_probe("Duration: N/A, bitrate: N/A\n Stream #0:0: Video: h264, 640x480");
        assert_eq!(probe.duration_secs, None);
        assert_eq!(probe.video_kbps, None);
    }
}
