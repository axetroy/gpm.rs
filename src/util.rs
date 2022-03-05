use std::path::PathBuf;

/// Find an available path that does not exist in your system
pub(crate) fn find_available_path(mut filepath: PathBuf) -> PathBuf {
    if !filepath.exists() {
        return filepath;
    }

    let mut index: i32 = 1;
    let origin = &filepath.clone();
    let base_name = origin.file_name().unwrap().to_str().unwrap();

    while filepath.exists() {
        let new_name = format!("{}({})", base_name, index);

        filepath = filepath.parent().unwrap().join(new_name);

        if !filepath.exists() {
            break;
        }

        index += 1;
    }

    filepath
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::util;

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
