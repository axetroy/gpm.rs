#![deny(warnings)]

use core::result::Result;
use eyre::Report;
use git_url_parse::GitUrl;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ChildProcess;

// git url to a file path
pub fn url_to_path(root: &str, url: &str) -> Result<PathBuf, Report> {
    match GitUrl::parse(url) {
        Ok(r) => {
            let mut dir = PathBuf::new();

            dir.push(root);
            dir.push(r.host.expect("invalid repository host"));
            dir.push(r.owner.expect("invalid repository owner"));
            dir.push(r.name);

            Ok(dir.to_path_buf())
        }
        Err(e) => Err(e),
    }
}

pub fn clone(url: &str, dest: &Path, args: Vec<&str>) -> io::Result<bool> {
    match ChildProcess::new("git")
        .arg("clone")
        .arg(url)
        .arg(dest.to_str().unwrap())
        .args(args)
        .spawn()
    {
        Ok(mut child) => match child.wait() {
            Ok(state) => Ok(state.success()),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use crate::git;

    #[test]
    fn test_url_to_path_when_empty() {
        let url1 = "https://github.com/axetroy/gpm.rs";

        let r1 = git::url_to_path("", url1);

        assert!(!r1.is_err());
        assert!(r1.is_ok());

        let p1 = r1.ok().unwrap();

        #[cfg(target_family = "unix")]
        let result1: &str = "github.com/axetroy/gpm.rs";
        #[cfg(target_family = "windows")]
        let result1: &str = "github.com\\axetroy\\gpm.rs";

        assert_eq!(p1.as_os_str().to_str().unwrap(), result1)
    }
}
