use std::{fs, io, path::Path, path::PathBuf};

// Walk gpm root folder
pub fn walk_root(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut repositories: Vec<PathBuf> = vec![];

    let root_dir = fs::read_dir(dir)?;

    for source in root_dir.flatten().map(|s| s.path()) {
        if !source.is_dir() {
            continue;
        }

        let owner_entry = fs::read_dir(source)?;

        for owner in owner_entry.flatten().map(|s| s.path()) {
            if !owner.is_dir() {
                continue;
            }

            let repo_entry = fs::read_dir(owner)?;

            for repo in repo_entry.flatten().map(|s| s.path()) {
                if !repo.is_dir() {
                    continue;
                }

                repositories.push(repo);
            }
        }
    }

    Ok(repositories)
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::walk;

    #[test]
    fn test_walk_root() {
        let test_gpm_root = env::current_dir().unwrap().join("__test__").join("gpm");

        let r1 = walk::walk_root(&test_gpm_root);

        assert!(!r1.is_err());

        let r1 = r1
            .ok()
            .unwrap()
            .into_iter()
            .map(|s| s.clone().as_os_str().to_str().unwrap().to_owned())
            .collect::<Vec<String>>();

        assert_eq!(
            vec![
                env::current_dir()
                    .unwrap()
                    .join("__test__")
                    .join("gpm",)
                    .join("github.com")
                    .join("another_owner")
                    .join("project")
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string(),
                env::current_dir()
                    .unwrap()
                    .join("__test__")
                    .join("gpm",)
                    .join("github.com")
                    .join("axetroy")
                    .join("gpm.rs")
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ],
            r1
        );
    }
}
