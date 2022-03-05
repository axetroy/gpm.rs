#![deny(warnings)]

use std::path::PathBuf;

pub fn find_path(mut filepath: PathBuf) -> PathBuf {
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
