#![deny(warnings)]

extern crate path_absolutize;

use core::result::Result;
use eyre::Report;
use git_url_parse::GitUrl;
use path_absolutize::*;
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

            let abs = dir.absolutize()?;

            Ok(abs.to_path_buf())
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
    use std::env;

    #[test]
    fn test_url_to_path_when_empty() {
        let url1 = "https://github.com/axetroy/gpm.rs";

        let r1 = git::url_to_path("", url1);

        assert!(!r1.is_err());
        assert!(r1.is_ok());

        let p1 = r1.ok().unwrap();
        let cwd = env::current_dir().unwrap();

        #[cfg(target_family = "unix")]
        let result1: &str =
            &(cwd.as_os_str().to_str().unwrap().to_owned() + &"/github.com/axetroy/gpm.rs");
        #[cfg(target_family = "windows")]
        let result1: &str =
            &(cwd.as_os_str().to_str().unwrap().to_owned() + &"\\github.com\\axetroy\\gpm.rs");

        assert_eq!(p1.as_os_str().to_str().unwrap(), result1)
    }

    #[test]
    fn test_url_to_path_when_relative() {
        let url1 = "https://github.com/axetroy/gpm.rs";

        let r1 = git::url_to_path("gpm", url1);

        assert!(!r1.is_err());
        assert!(r1.is_ok());

        let p1 = r1.ok().unwrap();
        let cwd = env::current_dir().unwrap();

        #[cfg(target_family = "unix")]
        let result1: &str =
            &(cwd.as_os_str().to_str().unwrap().to_owned() + &"/gpm/github.com/axetroy/gpm.rs");
        #[cfg(target_family = "windows")]
        let result1: &str =
            &(cwd.as_os_str().to_str().unwrap().to_owned() + &"\\gpm\\github.com\\axetroy\\gpm.rs");

        assert_eq!(p1.as_os_str().to_str().unwrap(), result1)
    }

    #[test]
    fn test_url_to_path_when_dot_relative() {
        let url1 = "https://github.com/axetroy/gpm.rs";

        let r1 = git::url_to_path("./gpm", url1);

        assert!(!r1.is_err());
        assert!(r1.is_ok());

        let p1 = r1.ok().unwrap();
        let cwd = env::current_dir().unwrap();

        #[cfg(target_family = "unix")]
        let result1: &str =
            &(cwd.as_os_str().to_str().unwrap().to_owned() + &"/gpm/github.com/axetroy/gpm.rs");
        #[cfg(target_family = "windows")]
        let result1: &str =
            &(cwd.as_os_str().to_str().unwrap().to_owned() + &"\\gpm\\github.com\\axetroy\\gpm.rs");

        assert_eq!(p1.as_os_str().to_str().unwrap(), result1)
    }

    #[test]
    fn test_url_to_path_when_parent_relative() {
        let url1 = "https://github.com/axetroy/gpm.rs";

        let r1 = git::url_to_path("../gpm", url1);

        assert!(!r1.is_err());
        assert!(r1.is_ok());

        let p1 = r1.ok().unwrap();
        let cwd = env::current_dir().unwrap();

        #[cfg(target_family = "unix")]
        let result1: &str = &(cwd
            .parent()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned()
            + &"/gpm/github.com/axetroy/gpm.rs");
        #[cfg(target_family = "windows")]
        let result1: &str = &(cwd
            .parent()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned()
            + &"\\gpm\\github.com\\axetroy\\gpm.rs");

        assert_eq!(p1.as_os_str().to_str().unwrap(), result1)
    }

    #[test]
    fn test_url_to_path_with_gitlab() {
        let url1 = "https://gitlab.com/axetroy/gpm.rs";

        let r1 = git::url_to_path("../gpm", url1);

        assert!(!r1.is_err());
        assert!(r1.is_ok());

        let p1 = r1.ok().unwrap();
        let cwd = env::current_dir().unwrap();

        #[cfg(target_family = "unix")]
        let result1: &str = &(cwd
            .parent()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned()
            + &"/gpm/gitlab.com/axetroy/gpm.rs");
        #[cfg(target_family = "windows")]
        let result1: &str = &(cwd
            .parent()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned()
            + &"\\gpm\\gitlab.com\\axetroy\\gpm.rs");

        assert_eq!(p1.as_os_str().to_str().unwrap(), result1)
    }

    #[test]
    fn test_url_to_path_with_gitlab_sub_org() {
        let url1 = "https://gitlab.com/org/sub_org/gpm.rs";

        let r1 = git::url_to_path(".", url1);

        assert!(!r1.is_err());
        assert!(r1.is_ok());

        let p1 = r1.ok().unwrap();
        let cwd = env::current_dir().unwrap();

        #[cfg(target_family = "unix")]
        let result1: &str =
            &(cwd.as_os_str().to_str().unwrap().to_owned() + &"/gitlab.com/sub_org/gpm.rs");
        #[cfg(target_family = "windows")]
        let result1: &str =
            &(cwd.as_os_str().to_str().unwrap().to_owned() + &"\\gitlab.com\\sub_org\\gpm.rs");

        assert_eq!(p1.as_os_str().to_str().unwrap(), result1)
    }
}
