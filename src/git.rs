#![deny(warnings)]

use git_url_parse::GitUrl;
use std::io;
use std::path::PathBuf;
use std::path::Path;
use std::process::Command as ChildProcess;

// git url to a file path
pub fn url_to_path(root: &str, url: &str) -> PathBuf {
    let repo_url = GitUrl::parse(url).expect("invalid repository URL");

    let mut dir = PathBuf::new();

    dir.push(root);
    dir.push(repo_url.host.expect("invalid repository host"));
    dir.push(repo_url.owner.expect("invalid repository owner"));
    dir.push(repo_url.name);

    dir.to_path_buf()
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
