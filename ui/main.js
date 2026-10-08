const { invoke, Channel } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const { revealItemInDir } = window.__TAURI__.opener;
const { getCurrentWebview } = window.__TAURI__.webview;

const template = document.getElementById("file-row");

// Aynı anda tek dosya işlenir (iki ekran ortak sırayı kullanır); büyük
// dosyalarda bellek ve CPU kullanımını sınırlar.
let queue = Promise.resolve();

function formatSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value.toLocaleString("tr-TR", { maximumFractionDigits: value < 10 ? 1 : 0 })} ${units[i]}`;
}

function fileName(path) {
  return path.split(/[\\/]/).pop();
}

function extensionOf(path) {
  const name = fileName(path);
  return name.includes(".") ? name.split(".").pop().toLowerCase() : "";
}

function checkedValue(name) {
  return document.querySelector(`input[name="${name}"]:checked`).value;
}

// --- Dosya satırları --------------------------------------------------------

function createRow(list, path) {
  const row = template.content.firstElementChild.cloneNode(true);
  row.querySelector(".badge").textContent = (extensionOf(path) || "?").slice(0, 4).toUpperCase();
  row.querySelector(".file-name").textContent = fileName(path);
  row.querySelector(".file-name").title = path;
  row.querySelector(".remove").addEventListener("click", () => row.remove());
  list.prepend(row);
  return row;
}

function setRow(row, state, size, status) {
  row.dataset.state = state;
  row.querySelector(".size").textContent = size;
  row.querySelector(".dot").hidden = !size;
  row.querySelector(".status").textContent = status;
  row.querySelector(".remove").disabled = state === "working";
  row.querySelector(".progress").hidden = state !== "working";
}

const STAGE_LABELS = {
  reading: "Dosya okunuyor",
  images: "Görseller sıkıştırılıyor",
  encoding: "Kodlanıyor",
  optimizing: "Optimize ediliyor",
  video: "Video sıkıştırılıyor",
  saving: "Kaydediliyor",
};

function showProgress(row, { percent, stage, done, total }) {
  const bar = row.querySelector(".progress");
  bar.setAttribute("aria-valuenow", percent);
  row.querySelector(".progress-fill").style.width = `${percent}%`;
  const detail = stage === "images" && total ? ` (${done}/${total})` : "";
  row.querySelector(".status").textContent = `${STAGE_LABELS[stage] ?? "İşleniyor"}${detail} • %${percent}`;
}

function sizeChange(result) {
  const sizes = `${formatSize(result.originalSize)} → ${formatSize(result.compressedSize)}`;
  const ratio = Math.round((1 - result.compressedSize / result.originalSize) * 100);
  const change = ratio >= 0 ? `%${ratio} küçüldü` : `%${-ratio} büyüdü`;
  return { sizes, change };
}

function showResult(row, result, doneLabel) {
  if (!result.outputPath) {
    setRow(row, "skipped", formatSize(result.originalSize), "Daha fazla küçültülemedi");
    return;
  }
  const { sizes, change } = sizeChange(result);
  setRow(row, "done", sizes, doneLabel ? `${doneLabel} • ${change}` : change);
  row.querySelector(".badge").textContent = extensionOf(result.outputPath).slice(0, 4).toUpperCase();
  row.querySelector(".file-name").textContent = fileName(result.outputPath);
  row.querySelector(".file-name").title = result.outputPath;
  const reveal = row.querySelector(".reveal");
  reveal.hidden = false;
  reveal.addEventListener("click", () => revealItemInDir(result.outputPath));
}

// `command`'ı sıraya alır; `doneLabel` başarı durumunun önüne eklenir.
function enqueue(row, command, args, doneLabel) {
  setRow(row, "queued", "", "Sırada");
  queue = queue.then(async () => {
    if (!row.isConnected) return;
    setRow(row, "working", "", "Başlıyor…");
    const onProgress = new Channel();
    onProgress.onmessage = (progress) => showProgress(row, progress);
    try {
      showResult(row, await invoke(command, { ...args, onProgress }), doneLabel);
    } catch (error) {
      setRow(row, "error", "", String(error));
    }
  });
}

// --- Sıkıştırma ekranı -----------------------------------------------------

const COMPRESSIBLE = ["pdf", "jpg", "jpeg", "png", "webp", "mp4"];

async function addCompressions(paths) {
  const list = panels.compress.list;
  const level = checkedValue("level");
  for (const path of paths) {
    const extension = extensionOf(path);
    const row = createRow(list, path);
    if (!COMPRESSIBLE.includes(extension)) {
      setRow(row, "error", "", "Desteklenmeyen dosya türü");
    } else if (extension === "mp4" && !(await ffmpegReady())) {
      setRow(row, "waiting", "", "FFmpeg bekleniyor");
      waitingVideos.push({ row, path, level });
    } else {
      enqueue(row, "compress_file", { path, level });
    }
  }
}

// --- Dönüştür ekranı -------------------------------------------------------

const CONVERSIONS = {
  webp: { from: "png", label: "WebP'ye dönüştürüldü", hint: "PNG dosyaları WebP biçimine dönüştürülür." },
  png: { from: "webp", label: "PNG'ye dönüştürüldü", hint: "WebP dosyaları PNG biçimine dönüştürülür." },
};

const losslessOption = document.getElementById("lossless-option");
const losslessInput = document.getElementById("lossless");
const convertHint = document.getElementById("convert-hint");

function updateConvertOptions() {
  const to = checkedValue("conversion");
  convertHint.textContent = CONVERSIONS[to].hint;
  // Kayıpsız/kayıplı seçimi yalnızca WebP hedefinde anlamlı; PNG zaten kayıpsız.
  losslessOption.hidden = to !== "webp";
}

for (const input of document.querySelectorAll('input[name="conversion"]')) {
  input.addEventListener("change", updateConvertOptions);
}
updateConvertOptions();

function addConversions(paths) {
  const list = panels.convert.list;
  const to = checkedValue("conversion");
  const lossless = losslessInput.checked;
  const { from, label } = CONVERSIONS[to];
  for (const path of paths) {
    const row = createRow(list, path);
    if (extensionOf(path) !== from) {
      setRow(row, "error", "", `Bu seçenek yalnızca ${from.toUpperCase()} dosyalarını dönüştürür`);
    } else {
      enqueue(row, "convert_file", { path, to, lossless }, label);
    }
  }
}

// --- Ekranlar (sekmeler) ---------------------------------------------------

const panels = {
  compress: {
    tab: document.getElementById("tab-compress"),
    element: document.getElementById("panel-compress"),
    add: addCompressions,
    filter: () => ({ name: "Desteklenen dosyalar", extensions: COMPRESSIBLE }),
  },
  convert: {
    tab: document.getElementById("tab-convert"),
    element: document.getElementById("panel-convert"),
    add: addConversions,
    filter: () => {
      const { from } = CONVERSIONS[checkedValue("conversion")];
      return { name: from.toUpperCase(), extensions: [from] };
    },
  },
};
let active = panels.compress;

for (const panel of Object.values(panels)) {
  panel.dropzone = panel.element.querySelector(".dropzone");
  panel.list = panel.element.querySelector(".files");
  const browse = panel.element.querySelector(".browse");
  browse.addEventListener("click", async () => {
    const picked = await open({ multiple: true, directory: false, filters: [panel.filter()] });
    if (picked) panel.add(Array.isArray(picked) ? picked : [picked]);
  });
  panel.dropzone.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      browse.click();
    }
  });
  panel.tab.addEventListener("click", () => selectPanel(panel));
}

function selectPanel(panel) {
  active = panel;
  for (const p of Object.values(panels)) {
    const selected = p === panel;
    p.tab.setAttribute("aria-selected", String(selected));
    p.tab.tabIndex = selected ? 0 : -1;
    p.element.hidden = !selected;
  }
}

// Sekmeler arasında ok tuşlarıyla gezinme (WAI-ARIA tabs deseni).
document.querySelector(".tabs").addEventListener("keydown", (e) => {
  if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
  const list = Object.values(panels);
  const next = list[(list.indexOf(active) + (e.key === "ArrowRight" ? 1 : list.length - 1)) % list.length];
  selectPanel(next);
  next.tab.focus();
});

// Tauri, dosya yollarını yalnızca kendi sürükle-bırak olayıyla verir;
// tarayıcının `drop` olayında gerçek dosya yolu bulunmaz. Bırakılan
// dosyalar açık olan ekrana gider.
getCurrentWebview().onDragDropEvent(({ payload }) => {
  if (payload.type === "enter" || payload.type === "over") {
    active.dropzone.classList.add("over");
  } else if (payload.type === "drop") {
    active.dropzone.classList.remove("over");
    active.add(payload.paths);
  } else {
    active.dropzone.classList.remove("over");
  }
});

// --- FFmpeg (MP4) ---------------------------------------------------------
// MP4 için sistemdeki FFmpeg kullanılır. Yoksa kullanıcıya kurulum sunulur;
// FFmpeg hazır olunca bekleyen videolar sıraya alınır.

const notice = document.getElementById("ffmpeg");
const noticeTitle = notice.querySelector(".notice-title");
const noticeText = notice.querySelector(".notice-text");
const noticeCommand = notice.querySelector(".notice-command");
const installButton = document.getElementById("ffmpeg-install");
const checkButton = document.getElementById("ffmpeg-check");
const siteLink = document.getElementById("ffmpeg-site");

const waitingVideos = [];
let ffmpeg = null;
let pollTimer = null;

async function refreshFfmpeg() {
  ffmpeg = await invoke("ffmpeg_status");
  if (ffmpeg.available) onFfmpegReady();
  return ffmpeg;
}

async function ffmpegReady() {
  if (ffmpeg?.available) return true;
  const status = await refreshFfmpeg();
  if (!status.available) showMissingNotice(status);
  return status.available;
}

function showMissingNotice(status) {
  notice.hidden = false;
  notice.classList.remove("ready");
  noticeTitle.textContent = "MP4 için FFmpeg gerekli";
  checkButton.hidden = false;
  const manual = status.installMethod === "manual";
  installButton.hidden = manual;
  installButton.disabled = Boolean(pollTimer);
  noticeCommand.hidden = !(manual && status.manualCommand);
  noticeCommand.textContent = status.manualCommand ?? "";
  siteLink.hidden = !manual;
  if (pollTimer) return;
  noticeText.textContent = {
    winget: "FFmpeg bu bilgisayarda bulunamadı. Windows'un winget aracıyla tek tıkla kurabilirsiniz.",
    homebrew: "FFmpeg bu bilgisayarda bulunamadı. Homebrew ile Terminal'de kurulabilir.",
    manual: "FFmpeg bu bilgisayarda bulunamadı. Aşağıdaki komutla kurup “Tekrar kontrol et”e basın.",
  }[status.installMethod];
}

function onFfmpegReady() {
  clearInterval(pollTimer);
  pollTimer = null;
  if (!notice.hidden) {
    notice.classList.add("ready");
    noticeTitle.textContent = "FFmpeg hazır";
    noticeText.textContent = `Sürüm ${ffmpeg.version}. Bekleyen videolar sıkıştırılıyor.`;
    installButton.hidden = checkButton.hidden = siteLink.hidden = noticeCommand.hidden = true;
    setTimeout(() => {
      notice.hidden = true;
    }, 6000);
  }
  for (const { row, path, level } of waitingVideos.splice(0)) {
    if (row.isConnected) enqueue(row, "compress_file", { path, level });
  }
}

installButton.addEventListener("click", async () => {
  try {
    const method = await invoke("install_ffmpeg");
    if (method === "manual") return showMissingNotice(ffmpeg);
    installButton.disabled = true;
    noticeText.textContent =
      "Kurulum ayrı bir pencerede sürüyor. Bittiğinde uygulama FFmpeg'i kendiliğinden algılayacak.";
    // Kurulum birkaç dakika sürebilir; 15 dakika boyunca yoklanır.
    const started = Date.now();
    pollTimer = setInterval(async () => {
      const status = await refreshFfmpeg();
      if (!status.available && Date.now() - started > 15 * 60 * 1000) {
        clearInterval(pollTimer);
        pollTimer = null;
        installButton.disabled = false;
        noticeText.textContent = "FFmpeg hâlâ bulunamadı. Kurulum penceresini kontrol edip tekrar deneyin.";
      }
    }, 4000);
  } catch (error) {
    noticeText.textContent = String(error);
  }
});

checkButton.addEventListener("click", async () => {
  checkButton.disabled = true;
  const status = await refreshFfmpeg();
  checkButton.disabled = false;
  if (!status.available && !pollTimer) {
    noticeText.textContent = "FFmpeg hâlâ bulunamadı.";
  }
});
