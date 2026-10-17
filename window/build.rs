//! Build script: embeds the interface (ui/), the app icon, the version
//! information and the application manifest into keepclip.exe.
//!
//! The icon is drawn by code (`build/icon.rs`), so no image file has to be
//! kept in sync with it.

use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "build/icon.rs"]
mod icon;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build/icon.rs");
    println!("cargo:rerun-if-changed=build/keepclip.manifest");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    let ico_path = out_dir.join("keepclip.ico");
    let ico = icon::ico(|raw| miniz_oxide::deflate::compress_to_vec_zlib(raw, 10));
    fs::write(&ico_path, ico).expect("could not write the app icon");

    // The manifest: never asks for administrator rights, made for Windows 10
    // and 11, current controls. The version comes from Cargo.toml and the
    // other version fields from tauri.conf.json.
    let windows = tauri_build::WindowsAttributes::new()
        .window_icon_path(&ico_path)
        .app_manifest(include_str!("build/keepclip.manifest"));
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("could not build the window's resources");
}
