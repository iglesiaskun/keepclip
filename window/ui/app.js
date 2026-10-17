// keepclip's window: the screens, and talking to the Rust side
// (src/commands.rs) through Tauri's `invoke` and its events.

const invoke = (command, args) => window.__TAURI__.core.invoke(command, args);
const listen = (event, fn) => window.__TAURI__.event.listen(event, fn);

const $ = (id) => document.getElementById(id);

/** Text for HTML. Everything that comes from a site goes through it. */
const esc = (text) => String(text ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

/* ---------- Icons (Lucide, ISC: icons/LICENSE-LUCIDE) ---------- */

const ICONS = [
  'chevron-left', 'chevron-right', 'circle-alert', 'circle-check', 'clipboard-paste', 'clock', 'copy',
  'download', 'file-video', 'film', 'folder', 'folder-open', 'history', 'image', 'info', 'link', 'list',
  'loader-circle', 'minus', 'music', 'play', 'rotate-ccw', 'settings', 'shield-check', 'user', 'x',
];
const icons = new Map();

/** Reads the icons once, at start, so every screen can draw them at once. */
async function loadIcons() {
  await Promise.all(ICONS.map(async (name) => {
    const text = await (await fetch(`icons/${name}.svg`)).text();
    const start = text.indexOf('>', text.indexOf('<svg')) + 1;
    icons.set(name, text.slice(start, text.lastIndexOf('</svg>')).trim());
  }));
}

const ic = (name, cls = '') =>
  `<svg class="ui ${cls}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${icons.get(name) ?? ''}</svg>`;

/* ---------- What the window shows ---------- */

const LOGIN_HINT = 'The site may want you to be logged in to see this video. keepclip cannot log in to sites yet.';
const PARTS_KEPT = 'The parts downloaded so far are kept. Try again picks up where it stopped.';
const NOT_A_LINK = 'That is not a web link (http:// or https://).';
const COUNT = ['no', 'one', 'two', 'three', 'four'];

let M = {
  view: 'reading',
  version: '',
  /** Opened like any app: a copied link fills the field and waits. */
  askFirst: false,
  folder: null,
  link: null,
  /** Reading: yt-dlp's weekly update comes first. */
  updating: false,
  /** What the link offers (LinkView), and the video shown. */
  data: null,
  video: 0,
  /** Choosing: the choice whose file name is being checked. */
  checking: null,
  /** Downloading: the choice, the parts and the step after them. */
  choice: null,
  replace: false,
  parts: [],
  step: null,
  saved: null,
  error: null,
  dialog: null,
  setup: null,
  tools: null,
  installError: null,
  settings: null,
  versions: null,
  pasteText: '',
  pasteMsg: '',
  /** The field holds the copied link, as it was put there. */
  pasteFilled: false,
  /** Settings was opened from the paste screen, and Done goes back to it. */
  settingsBack: false,
};

/** Thumbnails by "<reading>:<video>": a data: address, or null when none came. */
const thumbs = new Map();

const video = () => M.data?.videos[M.video];
const choice = () => video()?.choices[M.choice];
const thumbKey = (i) => `${M.data?.read}:${i}`;

function thumbBox(i, small = false) {
  const key = thumbKey(i);
  const cls = small ? 'thumb sm' : 'thumb';
  if (!thumbs.has(key)) return `<div class="${cls} sk" data-thumb="${i}"></div>`;
  const src = thumbs.get(key);
  if (!src) return `<div class="${cls} none" data-thumb="${i}" title="No thumbnail">${ic('image')}</div>`;
  return `<div class="${cls}" data-thumb="${i}"><img src="${esc(src)}" alt="" draggable="false"></div>`;
}

function card(i) {
  const v = M.data.videos[i];
  const sub = [];
  if (v.author) sub.push(`<span>${ic('user')}<span class="cut">${esc(v.author)}</span></span>`);
  if (v.duration) sub.push(`<span>${ic('clock')}${esc(v.duration)}</span>`);
  if (v.position) sub.push(`<span>${ic('list')}${esc(v.position)}</span>`);
  return `<div class="vc">${thumbBox(i)}<div class="vmeta"><div class="vtitle" title="${esc(v.title)}">${esc(v.title)}</div><div class="vsub">${sub.join('')}</div></div></div>`;
}

const linkChip = (link, prefix = '') =>
  `<div class="linkchip" title="${esc(link)}">${ic('link')}<span>${prefix}${esc(link)}</span></div>`;

const footer = (off = false, settings = false) => M.folder
  ? `<div class="kf">${ic('folder')}<span class="path" title="${esc(M.folder)}">Saving to <b>${esc(M.folder)}</b></span><button type="button" class="btn subtle sm" data-act="change-folder" ${off ? 'disabled' : ''}>Change</button>${settings ? `<button type="button" class="btn subtle icon sm" data-act="settings" aria-label="Settings" title="Settings">${ic('settings')}</button>` : ''}</div>`
  : '';

const PASTE_HINT = 'Copy the link of a video and paste it here.';
const PASTE_FILLED = 'The link you copied is below. Read it, or paste another.';

const VIEWS = {
  reading() {
    const [head, text] = M.updating
      ? ['Checking for a newer yt-dlp…', 'Once a week. Sites change often, and an old yt-dlp stops working.']
      : ['Reading the link…', 'yt-dlp is asking the site what this link offers. This takes a few seconds.'];
    return `${linkChip(M.link)}
      <div class="vc" style="border-bottom:0"><span class="thumb sk"></span><div class="vmeta"><span class="sk" style="width:92%;height:13px"></span><span class="sk" style="width:64%;height:13px"></span><span class="sk" style="width:38%;height:11px;margin-top:4px"></span></div></div>
      <div class="center"><span class="reading-ic">${ic('loader-circle', 'spin')}</span><b>${head}</b><p>${text}</p><button type="button" class="btn" data-act="close">Cancel</button></div>${footer()}`;
  },
  paste() {
    const head = M.askFirst ? 'Download a video' : 'No link in the clipboard';
    return `<div class="center"><span class="bigic dim">${ic('clipboard-paste')}</span><h2 class="kh">${head}</h2><p id="pastesub">${M.pasteFilled ? PASTE_FILLED : PASTE_HINT}</p>
      <form class="pasteform" id="pasteform" autocomplete="off"><input class="input mono" id="pasteinput" placeholder="https://" aria-label="Link" spellcheck="false" value="${esc(M.pasteText)}"><button type="submit" class="btn primary">Read link</button></form>
      <span class="pastemsg" id="pastemsg" role="alert">${esc(M.pasteMsg)}</span></div>${footer(false, true)}`;
  },
  choose() {
    const v = video();
    const rows = v.choices.map((o, i) => {
      const sep = o.mp3 && i > 0 ? '<div class="optsep" role="presentation"></div>' : '';
      const tip = o.estimated ? ' title="Estimated from the bitrate and the length"' : '';
      const busy = M.checking === i;
      const end = busy ? ic('loader-circle', 'spin') : ic('download', 'go');
      return `${sep}<button type="button" class="opt${busy ? ' busy' : ''}" data-act="pick" data-i="${i}"><span class="k">${i < 9 ? i + 1 : ''}</span>${ic(o.mp3 ? 'music' : 'film')}<span class="lb">${esc(o.label)}</span><span class="sz${o.unknown ? ' unk' : ''}"${tip}>${esc(o.size)}</span>${end}</button>`;
    }).join('');
    const several = M.data.videos.length;
    const back = several > 1
      ? `<div class="backrow"><button type="button" class="backlink" data-act="back">${ic('chevron-left')}All ${several} videos</button></div>`
      : '';
    return `${back}${card(M.video)}<div class="opts${M.checking != null ? ' checking' : ''}">${rows}</div>${footer()}`;
  },
  multi() {
    const rows = M.data.videos.map((v, i) =>
      `<button type="button" class="ent" data-act="entry" data-i="${i}"><span class="k">${i < 9 ? i + 1 : ''}</span>${thumbBox(i, true)}<span class="et"><span class="t" title="${esc(v.title)}">${esc(v.title)}</span><span class="d">${v.duration ? `${ic('clock')}${esc(v.duration)}` : ''}</span></span>${ic('chevron-right', 'go')}</button>`).join('');
    const author = M.data.author ? `<p>${ic('user')}<span>${esc(M.data.author)}</span></p>` : '';
    return `<div class="mhead"><h2 class="kh">This link has ${M.data.videos.length} videos</h2>${author}</div><div class="opts">${rows}</div>${footer()}`;
  },
  download() {
    const o = choice();
    return `${card(M.video)}<div class="dl"><div class="dlhead">${ic(o.mp3 ? 'music' : 'film')}<span>Downloading <b>${esc(o.label)}</b>${o.unknown ? '' : ` · ${esc(o.size)}`}</span></div>
      <div class="card" id="steps">${steps()}</div>
      <div class="dlfoot"><p class="fine">The parts wait in a temporary folder. The file moves to your folder when it is ready.</p><button type="button" class="btn" data-act="cancel-dl">Cancel</button></div></div>${footer(true)}`;
  },
  saved() {
    const s = M.saved;
    return `${card(M.video)}<div class="done"><span class="bigic ok">${ic('circle-check')}</span><h2 class="kh">Saved</h2>
      <div class="fname">${ic(choice()?.mp3 ? 'music' : 'file-video')}<span>${esc(s.name)}</span></div>
      <div class="infold">in ${esc(s.folder)}</div>
      <div class="btnrow"><button type="button" class="btn primary" data-act="show-file">${ic('folder-open')}Open folder</button><button type="button" class="btn" data-act="play">${ic('play')}Play</button><button type="button" class="btn" data-act="another">${ic('download')}Download another</button></div></div>
      <div class="cd" id="cd">${countdownHtml()}</div>`;
  },
  error() {
    const e = M.error;
    const top = e.during ? card(M.video) : linkChip(M.link ?? '');
    const hints = [];
    if (e.login) hints.push(`<div class="hint">${ic('info')}<span>${LOGIN_HINT}</span></div>`);
    if (e.during) hints.push(`<div class="hint">${ic('history')}<span>${PARTS_KEPT}</span></div>`);
    return `${top}<div class="err"><div class="errhead">${ic('circle-alert')}<div><h2 class="kh">${esc(e.title)}</h2></div></div>
      <div class="reason"><div class="reason-top"><span>${e.byYtdlp ? 'yt-dlp said' : 'What went wrong'}</span><button type="button" class="btn subtle icon sm" data-act="copy" aria-label="Copy" title="Copy">${ic('copy')}</button></div><div class="reason-text">${esc(e.said)}</div></div>
      ${hints.join('')}<div class="errfoot"><button type="button" class="btn" data-act="close">Close</button><button type="button" class="btn primary" data-act="retry">${ic('rotate-ccw')}Try again</button></div></div>`;
  },
  setup() {
    const s = M.setup;
    const n = s.programs.length;
    const what = n ? `a folder for your videos and ${COUNT[n] ?? n} free program${n === 1 ? '' : 's'}` : 'a folder for your videos';
    const tools = s.programs.map((t) => `<div class="tool"><div class="tn"><b>${esc(t.name)}</b><span>${esc(t.role)}</span></div><span class="mb">about ${t.megabytes} MB</span><span class="src">${esc(t.source)}</span></div>`).join('');
    const programs = n
      ? `<section class="sec"><h3 class="sec-t">Programs keepclip downloads</h3><div class="card">${tools}</div>
        <p class="fine">${ic('shield-check')}<span>Saved in <code>${esc(s.bin)}</code>. Each file is checked against the checksum its project publishes before keepclip uses it.</span></p></section>`
      : '';
    const go = n ? `${ic('download')}Download ${s.megabytes} MB` : 'Continue';
    return `<div class="setup"><div><h2 class="kh">Before the first download</h2><p class="sm-p">keepclip needs ${what}. This happens once.</p></div>
      ${M.link ? linkChip(M.link, 'Waiting: ') : ''}
      <section class="sec"><h3 class="sec-t">Save downloads in</h3><div class="folderrow"><div class="input ro" title="${esc(s.folder)}">${ic('folder')}<span>${esc(s.folder)}</span></div><button type="button" class="btn" data-act="browse">Browse…</button></div></section>
      ${programs}</div>
      <div class="kfoot"><button type="button" class="btn" data-act="close">Not now</button><button type="button" class="btn primary" data-act="setup-go" ${s.folder ? '' : 'disabled'}>${go}</button></div>`;
  },
  installing() {
    const s = M.setup;
    const allDone = s.programs.every((t) => M.tools[t.name].phase === 'done');
    const failed = M.installError != null;
    const head = allDone ? 'All set' : failed ? 'A download failed' : 'Getting the programs';
    const sub = allDone ? (M.link ? 'Reading your link now.' : 'Ready for your link.') : `Into <code>${esc(s.bin)}</code>`;
    const alert = failed ? `<div class="alert">${ic('circle-alert')}<span>${esc(M.installError)}</span></div>` : '';
    const buttons = failed
      ? '<button type="button" class="btn" data-act="close">Close</button><button type="button" class="btn primary" data-act="install-retry">Try again</button>'
      : `<button type="button" class="btn" data-act="install-cancel" ${allDone ? 'disabled' : ''}>Cancel</button>`;
    return `<div class="setup"><div><h2 class="kh">${head}</h2><p class="sm-p">${sub}</p></div>${M.link ? linkChip(M.link, 'Waiting: ') : ''}<div class="card" id="tools">${toolRows()}</div>${alert}</div><div class="kfoot">${buttons}</div>`;
  },
  settings() {
    const s = M.settings;
    const rows = s.programs.map((name) => {
      const version = M.versions == null ? '…' : (M.versions.find((v) => v.name === name)?.version ?? 'not installed');
      const right = name === 'yt-dlp' && s.checked ? esc(s.checked) : '';
      return `<div class="tool"><div class="tn"><b>${esc(name)}</b><span>${esc(version)}</span></div><span class="mb">${right}</span></div>`;
    }).join('');
    const programs = s.programs.length
      ? `<section class="sec"><h3 class="sec-t">Programs</h3><div class="card">${rows}</div><p class="fine">${ic('shield-check')}<span>In <code>${esc(s.bin)}</code>. yt-dlp updates itself once a week.</span></p></section>`
      : '';
    const folder = M.folder ?? 'Not chosen yet';
    return `<div class="setup"><h2 class="kh">Settings</h2>
      <section class="sec"><h3 class="sec-t">Save downloads in</h3><div class="folderrow"><div class="input ro" title="${esc(folder)}">${ic('folder')}<span>${esc(folder)}</span></div><button type="button" class="btn" data-act="settings-change">Change…</button><button type="button" class="btn icon" data-act="open-folder" aria-label="Open the folder" title="Open the folder" ${M.folder ? '' : 'disabled'}>${ic('folder-open')}</button></div></section>
      ${programs}</div>
      <div class="kfoot"><span class="ver">keepclip ${esc(M.version)}</span><button type="button" class="btn primary" data-act="settings-done">Done</button></div>`;
  },
};

function render() {
  $('kb').innerHTML = VIEWS[M.view]();
  const scrim = $('kscrim');
  if (M.dialog) {
    const d = M.dialog;
    const note = d.after
      ? 'The new download is ready. If you keep the old file, the new one is deleted.'
      : 'It stays as it is until the new one is ready.';
    scrim.innerHTML = `<div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="dlg-t"><div class="dlg-head"><h2 id="dlg-t">Replace the file?</h2></div>
      <div class="dlg-body"><p>The folder already has a file with this name:</p><div class="fname">${ic(video()?.choices[d.choice]?.mp3 ? 'music' : 'file-video')}<span>${esc(d.name)}</span></div><p>${note}</p></div>
      <div class="dlg-foot"><button type="button" class="btn" data-act="replace-yes">Replace</button><button type="button" class="btn primary" data-act="replace-no" id="keep-btn">Keep the old one</button></div></div>`;
    scrim.hidden = false;
    $('keep-btn').focus({ preventScroll: true });
  } else {
    scrim.hidden = true;
    scrim.innerHTML = '';
  }
  if (M.view === 'paste') wirePaste();
  syncTaskbar();
}

/* ---------- Title and taskbar ---------- */

let shownTitle = '';
let shownProgress = '';

/** The window's title and its taskbar button follow the download. */
function syncTaskbar() {
  let title = 'keepclip';
  let progress = null;
  let busy = false;
  if (M.view === 'download' && !M.dialog) {
    const o = overall();
    title = o.title;
    progress = o.fraction;
    busy = o.busy;
  } else if (M.view === 'installing' && M.installError == null) {
    const o = installOverall();
    if (o != null) {
      title = `${Math.floor(o * 100)}% · keepclip`;
      progress = o;
    }
  }
  const key = `${busy}:${progress == null ? '' : Math.floor(progress * 100)}`;
  if (title === shownTitle && key === shownProgress) return;
  shownTitle = title;
  shownProgress = key;
  $('ttl').textContent = title;
  invoke('taskbar', { title, progress, busy });
}

/** The whole download, from 0 to 1, or busy when it cannot be told. */
function overall() {
  if (M.step) return { title: `${M.step}… · keepclip`, busy: true, fraction: null };
  const o = choice();
  const parts = M.parts;
  let fraction = null;
  // A video's size is the size of its parts; an MP3's is the size of the
  // converted file, not of the sound downloaded.
  if (o && o.bytes && !o.mp3) {
    fraction = Math.min(0.99, parts.reduce((sum, p) => sum + (p.bytes ?? 0), 0) / o.bytes);
  } else if (parts.every((p) => p.finished || p.fraction != null)) {
    fraction = parts.reduce((sum, p) => sum + (p.finished ? 1 : p.fraction), 0) / parts.length;
  }
  if (fraction == null) return { title: 'Downloading… · keepclip', busy: true, fraction: null };
  return { title: `${Math.floor(fraction * 100)}% · keepclip`, busy: false, fraction };
}

/* ---------- Reading a link ---------- */

async function read(url) {
  M = { ...M, view: 'reading', link: url, updating: false, data: null, video: 0, error: null, dialog: null, checking: null };
  render();
  try {
    const data = await invoke('read_link', { url });
    if (M.view !== 'reading' || M.link !== url) return;
    M.data = data;
    M.view = data.videos.length > 1 ? 'multi' : 'choose';
    render();
  } catch (failure) {
    if (M.view !== 'reading' || M.link !== url) return;
    showError(failure, false);
  }
}

function showPaste(message = '', text = '', filled = false) {
  M = { ...M, view: 'paste', pasteMsg: message, pasteText: text, pasteFilled: filled, data: null };
  render();
  $('pasteinput')?.focus();
  if (M.askFirst && !text) fillFromClipboard();
}

/** Opened like any app: the copied link goes in the field, without reading
 * it. What the user typed stays; a link put there before gives way. */
async function fillFromClipboard() {
  const url = await invoke('clipboard_link');
  const input = $('pasteinput');
  if (!url || M.view !== 'paste' || !input || input.value === url) return;
  if (input.value.trim() && !M.pasteFilled) return;
  M = { ...M, pasteText: url, pasteFilled: true, pasteMsg: '' };
  render();
  $('pasteinput').focus();
  $('pasteinput').select();
}

function wirePaste() {
  const form = $('pasteform');
  const input = $('pasteinput');
  input.addEventListener('input', () => {
    M.pasteText = input.value;
    if (M.pasteFilled) {
      M.pasteFilled = false;
      $('pastesub').textContent = PASTE_HINT;
    }
  });
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const url = await invoke('find_link', { text: input.value });
    if (!url) {
      M.pasteMsg = NOT_A_LINK;
      $('pastemsg').textContent = NOT_A_LINK;
      return;
    }
    read(url);
  });
}

function showError(failure, during) {
  const f = typeof failure === 'string' ? { said: failure } : failure;
  M = {
    ...M,
    view: 'error',
    dialog: null,
    checking: null,
    error: {
      title: during ? 'The download stopped' : 'Could not read this link',
      said: f.said ?? String(failure),
      byYtdlp: !!f.byYtdlp,
      login: !!f.login,
      during,
    },
  };
  render();
}

/* ---------- Choosing and downloading ---------- */

async function pick(i) {
  if (M.view !== 'choose' || M.checking != null || !video()?.choices[i]) return;
  M.checking = i;
  render();
  let existing = null;
  try {
    existing = await invoke('existing_name', { video: M.video, choice: i });
  } catch (_) {
    // Without the name, the question comes when the file is ready.
  }
  M.checking = null;
  if (M.view !== 'choose') return;
  if (existing) {
    M.dialog = { name: existing, after: false, choice: i };
    render();
    return;
  }
  download(i, false);
}

async function download(i, replace) {
  const o = video().choices[i];
  M = { ...M, view: 'download', choice: i, replace, parts: o.parts.map((name) => ({ name })), step: null, dialog: null };
  render();
  try {
    ended(await invoke('download', { video: M.video, choice: i, replace }));
  } catch (failure) {
    if (M.view !== 'download') return;
    if (failure?.stopped) {
      M = { ...M, view: 'choose', parts: [], step: null };
      render();
      toast('Stopped. Pick it again to resume.');
      return;
    }
    showError(failure, true);
  }
}

function ended(done) {
  if (done.kind === 'saved') {
    M = { ...M, view: 'saved', saved: done, dialog: null, step: null };
    startCountdown();
    render();
  } else if (done.kind === 'exists') {
    M.parts.forEach((p) => { p.finished = true; });
    M.step = null;
    M.dialog = { name: done.name, after: true, choice: M.choice };
    render();
  } else {
    M = { ...M, view: 'choose', dialog: null, parts: [], step: null };
    render();
    toast('Kept the file that was there; the new download was deleted.');
  }
}

/** The answer to "Replace the file?". */
async function answerReplace(replace) {
  const d = M.dialog;
  if (!d) return;
  if (!d.after) {
    M.dialog = null;
    if (replace) download(d.choice, true);
    else {
      render();
      toast('Kept the file that was there.');
    }
    return;
  }
  M.dialog = null;
  render();
  try {
    ended(await invoke('finish', { replace }));
  } catch (failure) {
    showError(failure, true);
  }
}

function onProgress(p) {
  if (M.view !== 'download') return;
  if (p.kind === 'step') {
    M.parts.forEach((part) => { part.finished = true; });
    M.step = p.words;
  } else {
    const part = M.parts.find((x) => x.name === p.part);
    if (!part) return;
    Object.assign(part, p, { name: part.name, started: true });
  }
  const box = $('steps');
  if (box) box.innerHTML = steps();
  syncTaskbar();
}

function steps() {
  const active = M.step ? -1 : M.parts.findIndex((p) => !p.finished);
  const spin = ic('loader-circle', 'st-ic run spin');
  const rows = M.parts.map((p, i) => {
    if (p.finished) {
      return `<div class="step">${ic('circle-check', 'st-ic ok')}<span class="nm">${p.name}</span><span class="pct">${esc((p.total ?? p.done ?? '').replace(/^~/, ''))}</span></div>`;
    }
    if (i !== active) return `<div class="step wait"><span class="dot"></span><span class="nm">${p.name}</span><span class="pct">Waiting</span></div>`;
    const speed = p.speed ? ` · ${esc(p.speed)}` : '';
    if (p.fraction != null) {
      const pct = p.fraction * 100;
      const left = p.left ? ` · ${esc(p.left)} left` : '';
      return `<div class="step">${spin}<span class="nm">${p.name}</span><span class="pct">${Math.floor(pct)}%</span><div class="bar"><i style="width:${pct.toFixed(1)}%"></i></div><span class="det">${esc(p.done ?? '0 bytes')} of ${esc(p.total)}${speed}${left}</span></div>`;
    }
    const detail = p.started ? `${esc(p.done ?? '0 bytes')} so far${speed} · size unknown` : 'Starting…';
    return `<div class="step">${spin}<span class="nm">${p.name}</span><span class="pct"></span><div class="bar ind"><i></i></div><span class="det">${detail}</span></div>`;
  });
  const last = choice()?.lastStep;
  if (M.step) {
    rows.push(`<div class="step">${spin}<span class="nm">${esc(M.step)}…</span><span class="pct"></span><div class="bar ind"><i></i></div></div>`);
  } else if (last) {
    rows.push(`<div class="step wait"><span class="dot"></span><span class="nm">${esc(last)}</span><span class="pct">Then</span></div>`);
  }
  return rows.join('');
}

/* ---------- Saved: closes by itself in 5 s ---------- */

const CLOSE_AFTER = 5;
let countdown = null;

function startCountdown() {
  clearInterval(countdown?.timer);
  countdown = { left: CLOSE_AFTER, kept: false, timer: 0 };
  hovering = document.documentElement.matches(':hover');
  countdown.timer = setInterval(() => {
    if (countdown.kept || hovering) return;
    countdown.left -= 0.1;
    if (countdown.left <= 0) {
      clearInterval(countdown.timer);
      invoke('close');
      return;
    }
    updateCountdown();
  }, 100);
}

function countdownHtml() {
  if (!countdown) return '';
  let text = `Closing in ${Math.ceil(countdown.left)} s`;
  if (countdown.kept) text = 'Stays open. Close it with ✕ or Esc.';
  else if (hovering) text = 'Stays open while the mouse is over the window.';
  const width = countdown.kept ? 0 : (countdown.left / CLOSE_AFTER) * 100;
  return `<div class="cdbar"><i style="width:${width}%"></i></div><span>${text}</span>`;
}

function updateCountdown() {
  const cd = $('cd');
  if (!cd) return;
  cd.className = `cd${countdown?.kept || hovering ? ' paused' : ''}`;
  cd.innerHTML = countdownHtml();
}

/* ---------- First run ---------- */

async function showSetup() {
  const setup = await invoke('setup_view');
  // A folder picked before keeps showing.
  if (M.setup?.folder) setup.folder = M.setup.folder;
  M = { ...M, view: 'setup', setup, installError: null };
  render();
}

async function install() {
  const s = M.setup;
  if (!s.programs.length) {
    // Only the folder was missing.
    try {
      await invoke('install', { folder: s.folder });
    } catch (failure) {
      toast(failure?.said ?? String(failure));
      return;
    }
    M.folder = s.folder;
    if (M.link) read(M.link);
    else showPaste();
    return;
  }
  M.tools = Object.fromEntries(s.programs.map((t) => [t.name, M.tools?.[t.name]?.phase === 'done' ? M.tools[t.name] : { phase: 'wait' }]));
  M = { ...M, view: 'installing', installError: null };
  render();
  try {
    await invoke('install', { folder: s.folder });
  } catch (failure) {
    if (M.view !== 'installing') return;
    if (failure?.stopped) {
      await showSetup();
      return;
    }
    const running = Object.values(M.tools).find((t) => t.phase !== 'wait' && t.phase !== 'done');
    if (running) running.phase = 'fail';
    M.installError = failure?.said ?? String(failure);
    render();
    return;
  }
  M.folder = s.folder;
  render();
  setTimeout(() => (M.link ? read(M.link) : showPaste()), 1300);
}

function onInstall(p) {
  if (M.view !== 'installing' || !M.tools?.[p.tool]) return;
  M.tools[p.tool] = p;
  const box = $('tools');
  if (!box) return;
  const allDone = M.setup.programs.every((t) => M.tools[t.name].phase === 'done');
  if (allDone) render();
  else {
    box.innerHTML = toolRows();
    syncTaskbar();
  }
}

function toolRows() {
  const spin = ic('loader-circle', 'st-ic run spin');
  return M.setup.programs.map((t) => {
    const s = M.tools[t.name];
    const name = esc(t.name);
    switch (s.phase) {
      case 'done': return `<div class="step">${ic('circle-check', 'st-ic ok')}<span class="nm">${name}</span><span class="pct">Installed</span></div>`;
      case 'fail': return `<div class="step fail">${ic('circle-alert', 'st-ic bad')}<span class="nm">${name}</span><span class="pct">Not installed</span></div>`;
      case 'check':
      case 'unpack': return `<div class="step">${spin}<span class="nm">${name}</span><span class="pct"></span><div class="bar ind"><i></i></div><span class="det">${s.phase === 'check' ? 'Checking the checksum…' : 'Unpacking…'}</span></div>`;
      case 'download': {
        const speed = s.speed ? ` · ${esc(s.speed)}` : '';
        if (s.fraction == null) {
          return `<div class="step">${spin}<span class="nm">${name}</span><span class="pct"></span><div class="bar ind"><i></i></div><span class="det">${esc(s.done)} so far${speed} · from ${esc(t.source)}</span></div>`;
        }
        const pct = s.fraction * 100;
        return `<div class="step">${spin}<span class="nm">${name}</span><span class="pct">${Math.floor(pct)}%</span><div class="bar"><i style="width:${pct.toFixed(1)}%"></i></div><span class="det">${esc(s.done)} of ${esc(s.total)}${speed} · from ${esc(t.source)}</span></div>`;
      }
      default: return `<div class="step wait"><span class="dot"></span><span class="nm">${name}</span><span class="pct">about ${t.megabytes} MB</span></div>`;
    }
  }).join('');
}

/** The installation, from 0 to 1 by the programs' sizes, or null. */
function installOverall() {
  const programs = M.setup?.programs ?? [];
  const total = programs.reduce((sum, t) => sum + t.megabytes, 0);
  if (!total || programs.every((t) => M.tools[t.name].phase === 'done')) return null;
  const done = programs.reduce((sum, t) => {
    const s = M.tools[t.name];
    const part = s.phase === 'done' || s.phase === 'check' || s.phase === 'unpack' ? 1 : s.phase === 'download' ? (s.fraction ?? 0) : 0;
    return sum + part * t.megabytes;
  }, 0);
  return done / total;
}

/* ---------- Settings ---------- */

async function showSettings(back = false) {
  M = { ...M, view: 'settings', settings: await invoke('settings_view'), versions: null, settingsBack: back };
  render();
  // The versions take a moment: the screen shows "…" until they come.
  invoke('versions').catch(() => []).then((versions) => {
    M.versions = versions;
    if (M.view === 'settings') render();
  });
}

/** Done in Settings: back to the paste screen it came from, or closes. */
function leaveSettings() {
  if (M.settingsBack) showPaste(M.pasteMsg, M.pasteText, M.pasteFilled);
  else invoke('close');
}

/** From the Saved screen: stays open for the next link, which waits to be
 * read, as when keepclip is opened by itself. */
function another() {
  clearInterval(countdown?.timer);
  countdown = null;
  M.askFirst = true;
  showPaste();
}

/* ---------- Actions ---------- */

let toastTimer = 0;
function toast(text) {
  const t = $('ktoast');
  t.textContent = text;
  t.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.classList.remove('show'), Math.max(2800, text.length * 50));
}

async function changeFolder() {
  try {
    const folder = await invoke('change_folder');
    if (!folder) return;
    M.folder = folder;
    render();
    toast(`Downloads now go to ${folder}`);
  } catch (failure) {
    toast(failure?.said ?? String(failure));
  }
}

function openVideo(i) {
  if (!M.data?.videos[i]) return;
  M = { ...M, view: 'choose', video: i };
  render();
}

const ACTIONS = {
  min: () => window.__TAURI__.window.getCurrentWindow().minimize(),
  close: () => invoke('close'),
  pick: (el) => pick(Number(el.dataset.i)),
  entry: (el) => openVideo(Number(el.dataset.i)),
  back: () => { M = { ...M, view: 'multi' }; render(); },
  'change-folder': changeFolder,
  'settings-change': changeFolder,
  settings: () => showSettings(true),
  'settings-done': leaveSettings,
  another,
  'open-folder': () => invoke('open_folder').catch((e) => toast(String(e))),
  'replace-yes': () => answerReplace(true),
  'replace-no': () => answerReplace(false),
  'cancel-dl': () => invoke('cancel'),
  'show-file': () => invoke('show_file').catch((e) => toast(String(e))),
  play: () => invoke('play').catch((e) => toast(String(e))),
  copy: () => {
    const done = () => toast('Copied.');
    navigator.clipboard.writeText(M.error?.said ?? '').then(done, () => toast('Could not copy.'));
  },
  retry: () => {
    if (M.error?.during) download(M.choice, M.replace);
    else read(M.link);
  },
  browse: async () => {
    const folder = await invoke('pick_folder', { start: M.setup.folder }).catch(() => null);
    if (!folder) return;
    M.setup.folder = folder;
    render();
  },
  'setup-go': install,
  'install-cancel': () => invoke('cancel'),
  'install-retry': install,
};

document.addEventListener('click', (e) => {
  const el = e.target.closest('[data-act]');
  // A click on the Saved screen keeps the window open.
  if (M.view === 'saved' && countdown && !countdown.kept && !(el && ['show-file', 'play', 'another', 'close', 'min'].includes(el.dataset.act))) {
    countdown.kept = true;
    updateCountdown();
  }
  if (!el || el.disabled) return;
  ACTIONS[el.dataset.act]?.(el);
});

let hovering = false;
document.documentElement.addEventListener('mouseenter', () => { hovering = true; updateCountdown(); });
document.documentElement.addEventListener('mouseleave', () => { hovering = false; updateCountdown(); });

// Pictures that are taller than wide stay whole.
document.addEventListener('load', (e) => {
  const img = e.target;
  if (img instanceof HTMLImageElement && img.naturalWidth < img.naturalHeight * 1.2) img.classList.add('tall');
}, true);

const typing = () => !!document.activeElement?.matches('input, textarea');

document.addEventListener('keydown', (e) => {
  const key = e.key.toLowerCase();
  // Browser shortcuts that make no sense in this window.
  if (['f3', 'f5', 'f7'].includes(key) || (e.ctrlKey && ['r', 'p', 'f', 'g', 'j', 'u', 's', 'o'].includes(key))) {
    e.preventDefault();
    return;
  }
  if (M.dialog) {
    if (e.key === 'Escape') { e.preventDefault(); answerReplace(false); }
    return;
  }
  if (e.key === 'Escape') {
    e.preventDefault();
    if (M.view === 'settings') leaveSettings();
    else invoke('close');
    return;
  }
  if (typing() || e.ctrlKey || e.altKey || e.metaKey || !/^[1-9]$/.test(e.key)) return;
  const i = Number(e.key) - 1;
  if (M.view === 'choose') { e.preventDefault(); pick(i); }
  else if (M.view === 'multi') { e.preventDefault(); openVideo(i); }
});
document.addEventListener('contextmenu', (e) => {
  if (!typing() && !e.target.closest('.reason-text, .fname')) e.preventDefault();
});
// The mouse's Back and Forward buttons must never navigate the page.
for (const type of ['mousedown', 'mouseup', 'auxclick']) {
  document.addEventListener(type, (e) => { if (e.button === 3 || e.button === 4) e.preventDefault(); });
}

window.addEventListener('focus', async () => {
  document.body.classList.remove('inactive');
  if (M.view !== 'paste') return;
  // Opened like any app, a link copied since goes in the field.
  if (M.askFirst) {
    fillFromClipboard();
    return;
  }
  // Opened for the link in the clipboard, a link copied since is read.
  const url = await invoke('clipboard_link');
  if (url && M.view === 'paste') read(url);
});
window.addEventListener('blur', () => document.body.classList.add('inactive'));

/* ---------- Start ---------- */

async function start() {
  await loadIcons();
  $('tb-ic').innerHTML = ic('download');
  $('cap-min').innerHTML = ic('minus');
  $('cap-close').innerHTML = ic('x');
  await listen('stage', (e) => {
    if (M.view !== 'reading') return;
    M.updating = e.payload === 'update';
    render();
  });
  await listen('progress', (e) => onProgress(e.payload));
  await listen('install', (e) => onInstall(e.payload));
  await listen('thumb', (e) => {
    const { read: reading, video: i, src } = e.payload;
    thumbs.set(`${reading}:${i}`, src);
    if (reading !== M.data?.read) return;
    for (const box of document.querySelectorAll(`[data-thumb="${i}"]`)) {
      box.outerHTML = thumbBox(i, box.classList.contains('sm'));
    }
  });

  const s = await invoke('start');
  M.version = s.version;
  M.folder = s.folder;
  M.link = s.link;
  M.askFirst = s.askFirst;
  if (s.screen === 'settings') await showSettings();
  else if (s.screen === 'setup') await showSetup();
  else if (s.screen === 'read') {
    if (s.pickFolderFirst) {
      read(s.link);
      await invoke('ready');
      await changeFolder();
      return;
    }
    read(s.link);
  } else showPaste(s.notALink ? NOT_A_LINK : '', s.notALink ?? '');
  if (!document.body.classList.contains('inactive') && M.view === 'paste') $('pasteinput')?.focus();
}

// The window starts hidden; it appears even if something above failed, so
// a problem never leaves an invisible window behind.
start().catch((e) => {
  if (!M.error) showError(String(e?.said ?? e), false);
}).finally(() => invoke('ready'));
