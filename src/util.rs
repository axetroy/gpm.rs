use std::env;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// get absolute path
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
}
