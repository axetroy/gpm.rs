#![deny(warnings)]

use core::result::Result;
use eyre::Report;
use std::path::Path;
use std::process::Command as ChildProcess;
use which::which;

#[cfg(target_os = "macos")]
static DEFAULT_CODE_PATH: &str =
    "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code";
#[cfg(target_os = "linux")]
static DEFAULT_CODE_PATH: &str = "/usr/share/code/bin/code";
#[cfg(target_os = "freebsd")]
static DEFAULT_CODE_PATH: &str = "/usr/share/code/bin/code";
#[cfg(target_os = "windows")]
static DEFAULT_CODE_PATH: &str = "C:\\Program Files\\Microsoft VS Code";

// Open a path in file explorer
pub fn open(folder: &Path) -> Result<(), Report> {
    let open_command = match which("code") {
        Ok(p) => Ok(p),
        Err(_) => {
            // Try to find VS Code in the default install location
            let p = Path::new(DEFAULT_CODE_PATH);

            if p.exists() {
                Ok(p.to_path_buf())
            } else {
                Err(Report::msg("Visual Studio Code is not installed"))
            }
        }
    }?;

    ChildProcess::new(open_command)
        .arg(folder.as_os_str().to_str().unwrap())
        .spawn()?;

    Ok(())
}
