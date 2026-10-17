//! The commands the interface calls, and what the window keeps between
//! them. Slow work runs in a thread of its own and reports with events:
//! "stage" while reading a link, "progress" while downloading, "install"
//! on the first run and "thumb" when a thumbnail arrives.

use crate::view::{self, Failure, Install, LinkView, Video};
use crate::{shell, thumbs};
use keepclip_core::config::Config;
use keepclip_core::download::Request;
use keepclip_core::link;
use keepclip_core::progress::Event;
use keepclip_core::units;
use keepclip_system::args::Args;
use keepclip_system::files::{self, Moved};
use keepclip_system::stop::Stop;
use keepclip_system::tools::{self, Step};
use keepclip_system::{clipboard, notify, settings};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};
use tauri::window::{ProgressBarState, ProgressBarStatus};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

pub const WINDOW: &str = "main";

/// How often the interface hears about a download that is going on.
const UPDATE_EVERY: Duration = Duration::from_millis(120);

/// What the window keeps.
pub struct State {
    args: Args,
    settings_file: PathBuf,
    config: Mutex<Config>,
    /// Where the downloaded programs are.
    pub bin: PathBuf,
    /// The link read last, with its videos.
    link: Mutex<Option<ReadLink>>,
    /// Stops the work going on now, for Cancel.
    stop: Mutex<Stop>,
    /// A finished download whose name turned out to be taken, waiting for
    /// the answer to "Replace the file?".
    pending: Mutex<Option<Pending>>,
    /// The file saved last, for Open folder and Play.
    saved: Mutex<Option<PathBuf>>,
    /// Counts the links read, so the thumbnails of an earlier one are
    /// ignored.
    reads: AtomicU64,
}

struct ReadLink {
    url: String,
    /// What `yt-dlp -J` printed, for the name of the file.
    json: String,
    videos: Vec<Video>,
}

struct Pending {
    file: PathBuf,
    folder: PathBuf,
}

/// Locks a mutex. The values behind them are replaced whole, so a panic
/// elsewhere cannot leave one half changed.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl State {
    pub fn new(args: Args, settings_file: PathBuf, config: Config, bin: PathBuf) -> State {
        State {
            args,
            settings_file,
            config: Mutex::new(config),
            bin,
            link: Mutex::new(None),
            stop: Mutex::new(Stop::new()),
            pending: Mutex::new(None),
            saved: Mutex::new(None),
            reads: AtomicU64::new(0),
        }
    }

    /// A new handle for Cancel, for the work about to start.
    fn new_stop(&self) -> Stop {
        let stop = Stop::new();
        *lock(&self.stop) = stop.clone();
        stop
    }

    fn folder(&self) -> Option<PathBuf> {
        lock(&self.config).download_dir.clone()
    }

    fn use_folder(&self, folder: &Path) -> Result<(), Failure> {
        let mut config = lock(&self.config);
        settings::use_folder(&self.settings_file, &mut config, folder)?;
        Ok(())
    }

    fn read_link(&self, window: &WebviewWindow, url: &str) -> Result<LinkView, Failure> {
        let read = self.reads.fetch_add(1, Ordering::SeqCst) + 1;
        let url = link::find(url)
            .ok_or("That is not a web link (http:// or https://).")?
            .to_string();
        if tools::update_due(&self.bin) {
            let _ = window.emit("stage", "update");
            // A failed update is no reason to stop: the installed copy may
            // still work.
            let _ = tools::update(&self.bin);
            let _ = window.emit("stage", "read");
        }
        let stop = self.new_stop();
        let (info, json) = tools::ytdlp(&self.bin).info(&url, &stop)?;
        let videos = view::videos(&info)?;
        let link = view::link(read, &info, &videos);
        let pictures = videos
            .iter()
            .map(|v| {
                v.info
                    .thumbnail_urls(thumbs::WIDTH)
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            })
            .collect();
        *lock(&self.link) = Some(ReadLink { url, json, videos });
        thumbs::fetch(window.clone(), read, pictures);
        Ok(link)
    }

    /// The name of the file a choice makes, when the download folder has
    /// it already. `None` also when yt-dlp could not say.
    fn existing_name(&self, video: usize, choice: usize) -> Option<String> {
        let folder = self.folder()?;
        let work_folder = files::work_folder();
        std::fs::create_dir_all(&work_folder).ok()?;
        let link = lock(&self.link);
        let link = link.as_ref()?;
        let video = link.videos.get(video)?;
        let request = Request {
            url: &link.url,
            choice: video.choices.get(choice)?,
            work_folder: &work_folder,
            playlist_item: video.playlist_item,
        };
        let name = tools::ytdlp(&self.bin).final_name(&link.json, &request)?;
        folder.join(&name).exists().then_some(name)
    }

    fn download(
        &self,
        window: &WebviewWindow,
        video: usize,
        choice: usize,
        replace: bool,
    ) -> Result<Done, Failure> {
        let stop = self.new_stop();
        let folder = self.folder().ok_or("Choose a download folder first.")?;
        let work_folder = files::work_folder();
        for dir in [&folder, &work_folder] {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Could not create the folder {}: {e}", dir.display()))?;
        }
        let (url, picked, playlist_item) = {
            let link = lock(&self.link);
            let link = link.as_ref().ok_or("Read a link first.")?;
            let video = link.videos.get(video).ok_or("That video is not there.")?;
            let picked = video
                .choices
                .get(choice)
                .ok_or("That choice is not there.")?;
            (link.url.clone(), picked.clone(), video.playlist_item)
        };
        let request = Request {
            url: &url,
            choice: &picked,
            work_folder: &work_folder,
            playlist_item,
        };
        let mut saved = None;
        let mut last_update: Option<Instant> = None;
        tools::ytdlp(&self.bin).download(&request, &stop, |event| {
            if let Event::Saved(path) = &event {
                saved = Some(PathBuf::from(path));
                return;
            }
            let Some(progress) = view::progress(&picked, &event) else {
                return;
            };
            // Each change of state goes out; the figures, a few times a
            // second.
            let routine = matches!(
                progress,
                view::Progress::Part {
                    finished: false,
                    ..
                }
            );
            if routine && last_update.is_some_and(|at| at.elapsed() < UPDATE_EVERY) {
                return;
            }
            last_update = Some(Instant::now());
            let _ = window.emit("progress", progress);
        })?;
        let file = saved.ok_or("yt-dlp finished without saying where the file is.")?;
        match files::move_into(&file, &folder, replace)? {
            Moved::Saved(path) => Ok(self.saved(window, path)),
            Moved::Exists(target) => {
                *lock(&self.pending) = Some(Pending { file, folder });
                Ok(Done::Exists {
                    name: file_name(&target),
                })
            }
        }
    }

    /// The answer to "Replace the file?" asked after the download.
    fn finish(&self, window: &WebviewWindow, replace: bool) -> Result<Done, Failure> {
        let pending = lock(&self.pending)
            .take()
            .ok_or("There is no download waiting.")?;
        if !replace {
            let _ = std::fs::remove_file(&pending.file);
            return Ok(Done::Kept);
        }
        match files::move_into(&pending.file, &pending.folder, true)? {
            Moved::Saved(path) => Ok(self.saved(window, path)),
            Moved::Exists(_) => Err("Could not replace the file.".into()),
        }
    }

    fn saved(&self, window: &WebviewWindow, file: PathBuf) -> Done {
        // The window shows it already when it is in front.
        if !window.is_focused().unwrap_or(false) {
            notify::saved(&file);
        }
        let done = Done::Saved {
            name: file_name(&file),
            folder: file
                .parent()
                .map_or(String::new(), |f| f.display().to_string()),
        };
        *lock(&self.saved) = Some(file);
        done
    }

    fn install(&self, window: &WebviewWindow, folder: &Path) -> Result<(), Failure> {
        self.use_folder(folder)?;
        let stop = self.new_stop();
        for tool in tools::missing(&self.bin) {
            let started = Instant::now();
            let mut last_update: Option<Instant> = None;
            let mut on_step = |step: Step| {
                let update = match step {
                    Step::Download { done, total } => {
                        let finished = Some(done) == total;
                        if !finished && last_update.is_some_and(|at| at.elapsed() < UPDATE_EVERY) {
                            return;
                        }
                        last_update = Some(Instant::now());
                        let seconds = started.elapsed().as_secs_f64();
                        Install {
                            tool: tool.name,
                            phase: "download",
                            fraction: total.map(|t| done as f64 / t.max(1) as f64),
                            done: Some(units::bytes(done)),
                            total: total.map(units::bytes),
                            speed: (seconds > 0.5).then(|| units::speed(done as f64 / seconds)),
                        }
                    }
                    Step::Check => install_phase(tool.name, "check"),
                    Step::Unpack => install_phase(tool.name, "unpack"),
                };
                let _ = window.emit("install", update);
            };
            tools::install(&self.bin, tool, &mut on_step, &stop)?;
            let _ = window.emit("install", install_phase(tool.name, "done"));
        }
        Ok(())
    }
}

fn install_phase(tool: &'static str, phase: &'static str) -> Install {
    Install {
        tool,
        phase,
        fraction: None,
        done: None,
        total: None,
        speed: None,
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or(String::new(), |n| n.to_string_lossy().into_owned())
}

/// How a download ended.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Done {
    Saved {
        name: String,
        folder: String,
    },
    /// The name was taken: ask, then call `finish`.
    Exists {
        name: String,
    },
    /// The user kept the old file; the new one was deleted.
    Kept,
}

/// Runs slow work away from the window's thread.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, Failure> + Send + 'static,
) -> Result<T, Failure> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| Failure::from(format!("keepclip stopped unexpectedly: {e}")))?
}

/// What the window opens with.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartView {
    /// "read", "paste", "setup" or "settings".
    screen: &'static str,
    link: Option<String>,
    /// What was given on the command line, when it is not a link.
    not_a_link: Option<String>,
    folder: Option<String>,
    /// `--choose-folder` with a link: pick the folder, then read the link.
    pick_folder_first: bool,
    version: &'static str,
}

#[tauri::command]
fn start(state: tauri::State<'_, State>) -> StartView {
    let args = &state.args;
    let folder = state.folder();
    let mut start = StartView {
        screen: "paste",
        link: None,
        not_a_link: None,
        folder: folder.as_ref().map(|f| f.display().to_string()),
        pick_folder_first: false,
        version: env!("CARGO_PKG_VERSION"),
    };
    if args.choose_folder && args.link.is_none() && !args.clipboard {
        start.screen = "settings";
        return start;
    }
    match &args.link {
        Some(given) => match link::find(given) {
            Some(url) => start.link = Some(url.to_string()),
            None => start.not_a_link = Some(given.clone()),
        },
        None => {
            start.link = clipboard::text()
                .as_deref()
                .and_then(link::find)
                .map(String::from)
        }
    }
    let setup = folder.is_none() || !tools::missing(&state.bin).is_empty();
    if setup {
        start.screen = "setup";
    } else if start.link.is_some() {
        start.screen = "read";
        start.pick_folder_first = args.choose_folder;
    }
    start
}

/// The link in the clipboard, if it holds one.
#[tauri::command]
fn clipboard_link() -> Option<String> {
    clipboard::text()
        .as_deref()
        .and_then(link::find)
        .map(String::from)
}

/// The link in a text the user pasted or typed.
#[tauri::command]
fn find_link(text: String) -> Option<String> {
    link::find(&text).map(String::from)
}

#[tauri::command]
async fn read_link(window: WebviewWindow, url: String) -> Result<LinkView, Failure> {
    blocking(move || {
        let state = window.state::<State>();
        state.read_link(&window, &url)
    })
    .await
}

#[tauri::command]
async fn existing_name(
    app: AppHandle,
    video: usize,
    choice: usize,
) -> Result<Option<String>, Failure> {
    blocking(move || Ok(app.state::<State>().existing_name(video, choice))).await
}

#[tauri::command]
async fn download(
    window: WebviewWindow,
    video: usize,
    choice: usize,
    replace: bool,
) -> Result<Done, Failure> {
    blocking(move || {
        let state = window.state::<State>();
        state.download(&window, video, choice, replace)
    })
    .await
}

#[tauri::command]
async fn finish(window: WebviewWindow, replace: bool) -> Result<Done, Failure> {
    blocking(move || {
        let state = window.state::<State>();
        state.finish(&window, replace)
    })
    .await
}

/// Stops the download or the installation going on.
#[tauri::command]
fn cancel(state: tauri::State<'_, State>) {
    lock(&state.stop).stop();
}

/// Asks for a folder with the system's own dialog, starting at `start`.
fn pick(window: &WebviewWindow, start: Option<PathBuf>) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let mut dialog = window.dialog().file().set_parent(window);
    if let Some(start) = start.filter(|s| s.is_dir()) {
        dialog = dialog.set_directory(start);
    }
    dialog.blocking_pick_folder()?.into_path().ok()
}

/// Picks a folder without saving it, for the first run.
#[tauri::command]
async fn pick_folder(window: WebviewWindow, start: String) -> Result<Option<String>, Failure> {
    blocking(move || {
        let start = (!start.is_empty()).then(|| PathBuf::from(start));
        Ok(pick(&window, start).map(|p| p.display().to_string()))
    })
    .await
}

/// Picks another download folder and saves it. `None` when cancelled.
#[tauri::command]
async fn change_folder(window: WebviewWindow) -> Result<Option<String>, Failure> {
    blocking(move || {
        let state = window.state::<State>();
        let Some(folder) = pick(&window, state.folder()) else {
            return Ok(None);
        };
        state.use_folder(&folder)?;
        Ok(Some(folder.display().to_string()))
    })
    .await
}

/// The first run: the folder to suggest and the programs to download.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupView {
    folder: String,
    bin: String,
    programs: Vec<ProgramView>,
    megabytes: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramView {
    name: &'static str,
    role: &'static str,
    megabytes: u32,
    source: &'static str,
}

#[tauri::command]
fn setup_view(state: tauri::State<'_, State>) -> SetupView {
    let missing = tools::missing(&state.bin);
    let folder = settings::suggested_folder(&lock(&state.config))
        .map(|f| f.display().to_string())
        .unwrap_or_default();
    SetupView {
        folder,
        bin: state.bin.display().to_string(),
        megabytes: missing.iter().map(|t| t.megabytes).sum(),
        programs: missing
            .into_iter()
            .map(|t| ProgramView {
                name: t.name,
                role: t.role,
                megabytes: t.megabytes,
                source: t.source,
            })
            .collect(),
    }
}

#[tauri::command]
async fn install(window: WebviewWindow, folder: String) -> Result<(), Failure> {
    blocking(move || {
        let state = window.state::<State>();
        state.install(&window, Path::new(&folder))
    })
    .await
}

/// The Settings screen, without the versions, which take a moment.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    folder: Option<String>,
    bin: String,
    /// When yt-dlp was last updated: "checked 2 days ago".
    checked: Option<String>,
    programs: Vec<&'static str>,
}

#[tauri::command]
fn settings_view(state: tauri::State<'_, State>) -> SettingsView {
    let checked = tools::last_check(&state.bin)
        .and_then(|time| SystemTime::now().duration_since(time).ok())
        .map(|age| view::checked(age.as_secs() / (24 * 60 * 60)));
    SettingsView {
        folder: state.folder().map(|f| f.display().to_string()),
        bin: state.bin.display().to_string(),
        checked,
        programs: tools::downloaded().iter().map(|t| t.name).collect(),
    }
}

#[derive(Serialize)]
pub struct VersionView {
    name: &'static str,
    version: Option<String>,
}

/// The version of each program, as it reports it.
#[tauri::command]
async fn versions(app: AppHandle) -> Result<Vec<VersionView>, Failure> {
    blocking(move || {
        let bin = app.state::<State>().bin.clone();
        Ok(tools::versions(&bin)
            .into_iter()
            .map(|(name, version)| VersionView { name, version })
            .collect())
    })
    .await
}

/// Opens the download folder in File Explorer.
#[tauri::command]
fn open_folder(state: tauri::State<'_, State>) -> Result<(), String> {
    let folder = state.folder().ok_or("There is no download folder yet.")?;
    shell::open(&folder)
}

/// Opens File Explorer with the saved file selected, and closes keepclip.
#[tauri::command]
fn show_file(app: AppHandle, state: tauri::State<'_, State>) -> Result<(), String> {
    let file = lock(&state.saved).clone().ok_or("Nothing was saved yet.")?;
    shell::show_in_folder(&file)?;
    app.exit(0);
    Ok(())
}

/// Plays the saved file in the default player, and closes keepclip.
#[tauri::command]
fn play(app: AppHandle, state: tauri::State<'_, State>) -> Result<(), String> {
    let file = lock(&state.saved).clone().ok_or("Nothing was saved yet.")?;
    shell::open(&file)?;
    app.exit(0);
    Ok(())
}

/// The window's title, and the progress on its taskbar button: `progress`
/// from 0 to 1, or `busy` for work of unknown length.
#[tauri::command]
fn taskbar(window: WebviewWindow, title: String, progress: Option<f64>, busy: bool) {
    let _ = window.set_title(&title);
    let state = if busy {
        ProgressBarState {
            status: Some(ProgressBarStatus::Indeterminate),
            progress: None,
        }
    } else if let Some(fraction) = progress {
        ProgressBarState {
            status: Some(ProgressBarStatus::Normal),
            progress: Some((fraction.clamp(0.0, 1.0) * 100.0).round() as u64),
        }
    } else {
        ProgressBarState {
            status: Some(ProgressBarStatus::None),
            progress: None,
        }
    };
    let _ = window.set_progress_bar(state);
}

/// The interface is drawn: show the window, in front, with the focus.
#[tauri::command]
fn ready(window: WebviewWindow) {
    crate::fit(&window);
    let _ = window.show();
    let _ = window.set_focus();
}

/// Closes keepclip. A download going on ends with it.
#[tauri::command]
fn close(app: AppHandle) {
    app.exit(0);
}

pub fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        start,
        clipboard_link,
        find_link,
        read_link,
        existing_name,
        download,
        finish,
        cancel,
        pick_folder,
        change_folder,
        setup_view,
        install,
        settings_view,
        versions,
        open_folder,
        show_file,
        play,
        taskbar,
        ready,
        close
    ]
}
