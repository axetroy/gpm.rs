#![deny(warnings)]
extern crate path_absolutize;

use eyre::Report;
use inquire::Confirm;
use path_absolutize::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process;

#[derive(Serialize, Deserialize, Debug)]
pub struct Configure {
    #[serde(skip)]
    pub file_path: String, // this is configure file path and only got value in runtime struct
    pub root: Vec<String>, // the root of the repository
}

pub fn new(gpm_rc_file_path: &Path) -> Result<Configure, Report> {
    let mut rc_file = match File::open(gpm_rc_file_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(eyre::Report::from(e));
        }
    };

    let mut file_content = String::new();

    match rc_file.read_to_string(&mut file_content) {
        Ok(_) => {}
        Err(e) => {
            return Err(eyre::Report::from(e));
        }
    };

    drop(rc_file);

    let mut rc: Configure = match serde_json::from_str(&file_content) {
        Ok(r) => r,
        Err(e) => return Err(eyre::Report::from(e)),
    };

    let file_path = gpm_rc_file_path.as_os_str().to_str().unwrap().to_string();

    rc.file_path = file_path;

    Ok(rc)
}

impl Configure {
    fn update_file(&self) -> Result<(), Report> {
        let serialized = serde_json::to_string(&self).unwrap();

        fs::write(Path::new(&self.file_path), serialized)
            .unwrap_or_else(|e| panic!("can not write to '{}': {:?}", &self.file_path, e));
        Ok(())
    }

    pub fn add_field(&mut self, field: &str, value: &str) -> Result<(), Report> {
        let result = match field {
            "root" => {
                let value_normal = &value.replace('/', &std::path::MAIN_SEPARATOR.to_string());
                let add_abs_root_path = Path::new(value_normal).absolutize().unwrap();

                if !add_abs_root_path.exists() {
                    let ans = Confirm::new("The target folder not exist, do you want to create?")
                        .with_default(false)
                        .with_help_message(
                            add_abs_root_path
                                .as_os_str()
                                .to_os_string()
                                .to_str()
                                .unwrap(),
                        )
                        .prompt();

                    match ans {
                        Ok(true) => {
                            fs::create_dir(&add_abs_root_path).expect("can not create folder")
                        }
                        Ok(false) => process::exit(0x0),
                        Err(_) => process::exit(0x0),
                    };
                } else if !add_abs_root_path.is_dir() {
                    panic!("The target filepath is not a folder.")
                }

                let new_roo_str = &add_abs_root_path
                    .as_os_str()
                    .to_os_string()
                    .to_str()
                    .unwrap()
                    .to_string();

                if !self.root.contains(new_roo_str) {
                    println!("Added '{}' to root of configure.", new_roo_str);
                    self.root.push(new_roo_str.to_owned());
                }

                Option::Some(true)
            }
            _ => Option::None,
        };

        if result.is_none() {
            Err(Report::msg(format!(
                "unknown field '{}' of configure",
                field
            )))
        } else {
            self.update_file()
        }
    }

    pub fn set_field(&mut self, field: &str, value: &str) -> Result<(), Report> {
        let result = match field {
            "root" => {
                let value_normal = &value.replace('/', &std::path::MAIN_SEPARATOR.to_string());
                let add_abs_root_path = Path::new(value_normal).absolutize().unwrap();

                if !add_abs_root_path.exists() {
                    let ans = Confirm::new("The target folder not exist, do you want to create?")
                        .with_default(false)
                        .with_help_message(
                            add_abs_root_path
                                .as_os_str()
                                .to_os_string()
                                .to_str()
                                .unwrap(),
                        )
                        .prompt();

                    match ans {
                        Ok(true) => {
                            fs::create_dir(&add_abs_root_path).expect("can not create folder")
                        }
                        Ok(false) => process::exit(0x0),
                        Err(_) => process::exit(0x0),
                    };
                } else if !add_abs_root_path.is_dir() {
                    panic!("The target filepath is not a folder.")
                }

                let new_roo_str = &add_abs_root_path
                    .as_os_str()
                    .to_os_string()
                    .to_str()
                    .unwrap()
                    .to_string();

                println!("Set '[{}]' to root of configure.", new_roo_str);

                self.root = vec![new_roo_str.to_owned()];
                Option::Some(true)
            }
            _ => Option::None,
        };

        if result.is_none() {
            Err(Report::msg(format!(
                "unknown field '{}' of configure",
                field
            )))
        } else {
            self.update_file()
        }
    }

    pub fn remove_field(&mut self, field: &str) -> Result<(), Report> {
        let result = match field {
            "root" => {
                self.root = vec![];
                Option::Some(true)
            }
            _ => Option::None,
        };

        if result.is_none() {
            Err(Report::msg(format!(
                "unknown field '{}' of configure",
                field
            )))
        } else {
            self.update_file()
        }
    }

    pub fn reset(&mut self) -> Result<(), Report> {
        self.root = vec![];

        self.update_file()
    }
}

impl fmt::Display for Configure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let serialized = serde_json::to_string(self).unwrap();
        write!(f, "{}", serialized)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::configure;

    #[test]
    fn test_empty_configure() {
        let gpm_rc = env::current_dir()
            .unwrap()
            .join("__test__")
            .join("config")
            .join(".gpmrc-default.json");

        let rc = configure::new(&gpm_rc).unwrap();

        assert_eq!(rc.file_path, gpm_rc.into_os_string().to_str().unwrap());
        assert_eq!(rc.root, Vec::<String>::new());
    }

    #[test]
    fn test_configure_not_exist() {
        let gpm_rc = env::current_dir()
            .unwrap()
            .join("__test__")
            .join("config")
            .join(".gpmrc-not-exist.json");

        let rc = configure::new(&gpm_rc);

        assert!(rc.is_err());
    }

    #[test]
    fn test_configure_empty_content_file() {
        let gpm_rc = env::current_dir()
            .unwrap()
            .join("__test__")
            .join("config")
            .join(".gpmrc-empty.json");

        let rc = configure::new(&gpm_rc);

        assert!(rc.is_err());
    }

    #[test]
    fn test_configure_add_field() {
        let gpm_rc = env::current_dir()
            .unwrap()
            .join("__test__")
            .join("config")
            .join(".gpmrc-add.json");

        let rc = configure::new(&gpm_rc);

        assert!(rc.is_ok());

        let mut config = rc.unwrap();

        assert_eq!(config.root, Vec::<String>::new());

        {
            let r1 = config.add_field("field", "value");

            assert!(r1.is_err());
        }

        {
            let r1 = config.add_field("root", "./src");

            assert!(r1.is_ok());

            let cwd = env::current_dir().unwrap();

            let target_dir = cwd.join("src").as_os_str().to_str().unwrap().to_string();

            let root = vec![target_dir.clone()];

            assert_eq!(config.root, root);

            assert_eq!(
                format!("{}", config),
                format!(r#"{{"root":["{}"]}}"#, target_dir.replace('\\', "\\\\"))
            );

            config.reset().unwrap();

            assert_eq!(format!("{}", config), "{\"root\":[]}");
        }
    }
}
