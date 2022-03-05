#![deny(warnings)]

use std::path::Path;
use std::process::Command as ChildProcess;

// https://stackoverflow.com/questions/66485945/with-rust-open-explorer-on-a-file
#[cfg(target_os = "macos")]
static OPEN_COMMAND: &str = "open";
#[cfg(target_os = "linux")]
static OPEN_COMMAND: &str = "xdg-open";
#[cfg(target_os = "windows")]
static OPEN_COMMAND: &str = "explorer";

pub fn open(folder: &Path) {
    ChildProcess::new(OPEN_COMMAND)
        .arg(folder.as_os_str().to_str().unwrap())
        .spawn()
        .unwrap();
}
