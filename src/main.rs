#![deny(warnings)]
mod find_path;
mod open;
use clap::{arg, Command};
use find_path::find_path;
use git_url_parse::GitUrl;
use inquire::{error::InquireError, Confirm, Select};
use open::open as open_in_folder;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command as ChildProcess;

fn get_absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        let r = env::current_dir();
        if let Ok(r) = r {
            Ok(r.join(path))
        } else {
            Err(r.err().unwrap())
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Preset {
    root: Vec<String>,
}

fn main() {
    let matches = Command::new("gpm")
        .version("v0.1.1")
        .author("Axetroy")
        .about("A command line tool, manage your hundreds of repository, written with Rust")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .allow_invalid_utf8_for_external_subcommands(true)
        .subcommand(
            Command::new("clone")
                .about("Clones repos")
                .arg(arg!(<REMOTE> "The remote to clone"))
                .arg(arg!([OPTIONS] "The options for git clone command"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("config")
                .about("Update configure")
                .subcommand(
                    Command::new("add")
                        .about("Add configure for a field")
                        .arg(arg!(<FIELD> "The field of configure"))
                        .arg(arg!(<VALUE> "The value of the field"))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("set")
                        .about("Set configure for a field")
                        .arg(arg!(<FIELD> "The field of configure"))
                        .arg(arg!(<VALUE> "The value of the field"))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("remove")
                        .about("Remove configure for a field")
                        .arg(arg!(<FIELD> "The field of configure"))
                        .arg_required_else_help(true),
                )
                .subcommand(Command::new("reset").about("Reset configure")),
        )
        .get_matches();

    let home_dir = dirs::home_dir().unwrap();
    let mut gpm_rc = home_dir;
    gpm_rc.push(".gpmrc");

    let is_gpm_rc_exist = Path::new(gpm_rc.as_os_str()).exists();

    if !is_gpm_rc_exist {
        let mut file = File::create(gpm_rc.as_path()).expect("can not create a .gpmrc file");
        file.write_all(b"{\"root\": []}")
            .expect("can not write to $HOME/.gpmrc");
        drop(file);
    }

    let mut rc_file = File::open(gpm_rc.as_path()).unwrap();

    let mut file_content = String::new();
    rc_file.read_to_string(&mut file_content).unwrap();
    drop(rc_file);

    let mut rc: Preset = serde_json::from_str(&file_content).unwrap();

    match matches.subcommand() {
        Some(("clone", sub_matches)) => {
            let url = sub_matches.value_of("REMOTE").expect("required");

            let clone_args = match sub_matches.value_of("OPTIONS") {
                Some(s) => s.split(" ").collect::<Vec<&str>>(),
                _ => vec![],
            };

            let mut dest_dir = PathBuf::new();
            let gpm_root: &str = if rc.root.is_empty() {
                panic!("did not found root config in profile.\ntry run the command 'gpm config add root <folder>'");
            } else if rc.root.len() == 1 {
                let s = &rc.root[0].as_str();
                s
            } else {
                let options: Vec<&str> = rc.root.iter().map(|s| &**s).collect();

                let ans: Result<&str, InquireError> =
                    Select::new("Select a root path for clone?", options).prompt();

                match ans {
                    Ok(choice) => choice,
                    Err(_) => process::exit(0x0),
                }
            };

            let repo_url = GitUrl::parse(url).expect("invalid repository URL");

            dest_dir.push(gpm_root);
            dest_dir.push(repo_url.host.expect("invalid repository host"));
            dest_dir.push(repo_url.owner.expect("invalid repository owner"));
            dest_dir.push(repo_url.name);
            dest_dir = find_path(dest_dir);

            let dest_dir_will_be_removed = dest_dir.clone();

            // remove temp dir when cancel the action
            ctrlc::set_handler(move || {
                fs::remove_dir_all(dest_dir_will_be_removed.as_path()).unwrap_err();
                println!("received Ctrl+C!");
            })
            .expect("Error setting Ctrl-C handler");

            let mut child = ChildProcess::new("git")
                .arg("clone")
                .arg(url)
                .arg(dest_dir.to_str().unwrap())
                .args(clone_args)
                .spawn()
                .expect("failed to execute child");

            let ecode = child.wait().expect("failed to wait on child");

            println!("Clone into '{}'", dest_dir.to_str().unwrap());

            if !ecode.success() {
                if dest_dir.exists() {
                    fs::remove_dir_all(dest_dir).unwrap();
                }
                process::exit(ecode.code().unwrap_or(1));
            } else {
                open_in_folder(&dest_dir);
            }
        }
        Some(("config", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("add", sub_matches)) => {
                    let field = sub_matches.value_of("FIELD").expect("required");
                    let value = sub_matches.value_of("VALUE").expect("required");

                    match field {
                        "root" => {
                            let add_abs_root_path = &get_absolute_path(&Path::new(value)).unwrap();

                            if !add_abs_root_path.exists() {
                                let ans = Confirm::new(
                                    "The target folder not exist, do you want to create?",
                                )
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
                                    Ok(true) => fs::create_dir(add_abs_root_path)
                                        .expect("can not create folder"),
                                    Ok(false) => process::exit(0x00),
                                    Err(_) => process::exit(0x00),
                                };
                            } else if !add_abs_root_path.is_dir() {
                                panic!("The target filepath is not a folder.")
                            }

                            let new_roo_str = add_abs_root_path.to_str().unwrap().to_string();

                            if !rc.root.contains(&new_roo_str) {
                                rc.root.push(new_roo_str);
                            }
                        }
                        _ => panic!("unknown configure field '{}' for add", field),
                    }
                }
                Some(("set", sub_matches)) => {
                    let field = sub_matches.value_of("FIELD").expect("required");
                    let value = sub_matches.value_of("VALUE").expect("required");

                    match field {
                        "root" => {
                            let add_abs_root_path = &get_absolute_path(&Path::new(value)).unwrap();

                            if !add_abs_root_path.exists() {
                                let ans = Confirm::new(
                                    "The target folder not exist, do you want to create?",
                                )
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
                                    Ok(true) => fs::create_dir(add_abs_root_path)
                                        .expect("can not create folder"),
                                    Ok(false) => process::exit(0x00),
                                    Err(_) => process::exit(0x00),
                                };
                            } else if !add_abs_root_path.is_dir() {
                                panic!("The target filepath is not a folder.")
                            }

                            let new_roo_str = add_abs_root_path.to_str().unwrap().to_string();

                            rc.root = vec![new_roo_str]
                        }
                        _ => panic!("unknown configure field '{}' for set", field),
                    }
                }
                Some(("remove", _)) => {
                    let field = sub_matches.value_of("FIELD").expect("required");

                    match field {
                        "root" => {
                            rc.root = vec![];
                        }
                        _ => panic!("unknown configure field '{}' for remove", field),
                    }
                }
                Some(("reset", _)) => {
                    rc.root = vec![];
                }
                _ => unreachable!(),
            }

            let serialized = serde_json::to_string(&rc).unwrap();

            fs::write(gpm_rc, serialized).expect("can not write to $HOME/.gpmrc");
        }
        Some((ext, sub_matches)) => {
            let args = sub_matches
                .values_of_os("")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            println!("Calling out to {:?} with {:?}", ext, args);
        }
        _ => unreachable!(),
    }

    // Continued program logic goes here...
}
