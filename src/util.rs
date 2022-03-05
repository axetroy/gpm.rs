use std::env;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Get absolute path
pub(crate) fn get_absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        match env::current_dir() {
            Ok(r) => Ok(r.join(path)),
            Err(e) => Err(e),
        }
    }
}

/// Find an available path that does not exist in your system
pub(crate) fn find_available_path(mut filepath: PathBuf) -> PathBuf {
    if !filepath.exists() {
        return filepath;
    }

    let mut index: i32 = 1;
    let origin = &filepath.clone();
    let base_name = origin.file_name().unwrap().to_str().unwrap();

    while filepath.exists() {
        let mut name = base_name.to_string();

        name.push('(');
        name.push_str(index.to_string().as_str());
        name.push(')');

        filepath = filepath.parent().unwrap().join(name);

        if !filepath.exists() {
            break;
        }

        index += 1;
    }

    filepath
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use crate::util;

    #[test]
    fn test_get_absolute_path_unix() {
        let p1 = PathBuf::new();
        let r1 = util::get_absolute_path(p1);

        assert!(!r1.is_err());
        assert!(r1.is_ok());

        assert_eq!(
            r1.unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .as_str()
                .trim_end_matches("/")
                .trim_end_matches("\\"),
            env::current_dir().ok().unwrap().to_str().unwrap()
        );

        let mut p2 = PathBuf::new();
        p2.push("__test__");
        let r2 = util::get_absolute_path(p2);

        assert!(!r2.is_err());
        assert!(r2.is_ok());

        assert_eq!(
            r2.unwrap().to_str().unwrap(),
            env::current_dir()
                .ok()
                .unwrap()
                .join("__test__")
                .to_str()
                .unwrap()
        );
    }

    #[test]
    fn test_find_available_path() {
        let p1 = env::current_dir().unwrap().join("__not_exist__");

        let r1 = util::find_available_path(p1);

        assert_eq!(
            r1.as_os_str().to_str(),
            env::current_dir()
                .ok()
                .unwrap()
                .join("__not_exist__")
                .as_os_str()
                .to_str()
        );

        let p1 = env::current_dir().unwrap().join(".github");

        let r1 = util::find_available_path(p1);

        assert_eq!(
            r1.as_os_str().to_str(),
            env::current_dir()
                .ok()
                .unwrap()
                .join(".github(1)")
                .as_os_str()
                .to_str()
        );
    }
}
