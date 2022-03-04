#![deny(warnings)]

use clap::{arg, Command};
use git_url_parse::GitUrl;
use inquire::{error::InquireError, Select, Text};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command as ChildProcess;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
struct Preset {
    root: Vec<String>,
}

fn main() {
    let matches = Command::new("gpm")
        .about("A cli for manager you project with Golang style")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .allow_invalid_utf8_for_external_subcommands(true)
        .subcommand(
            Command::new("clone")
                .about("Clones repos")
                .arg(arg!(<REMOTE> "The remote to clone"))
                .arg_required_else_help(true),
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
    }

    let mut file = File::open(gpm_rc.as_path()).unwrap();

    let mut file_content = String::new();
    file.read_to_string(&mut file_content).unwrap();

    let rc: Preset = serde_json::from_str(&file_content).unwrap();

    println!("{:?}", rc);

    match matches.subcommand() {
        Some(("clone", sub_matches)) => {
            let mut dest_dir = PathBuf::new();
            let url = sub_matches.value_of("REMOTE").expect("required");
            let gpm_root: &str = if rc.root.is_empty() {
                panic!("did not found root folder in profile.");
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

            let u = GitUrl::parse(url).unwrap();

            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            let temp_clone_dir = env::temp_dir()
                .as_path()
                .join("gpm_".to_owned() + u.name.as_str() + "_" + &since_the_epoch.to_string())
                .join(u.name.as_str());

            let name = u.name.clone();

            dest_dir.push(gpm_root);
            dest_dir.push(u.host.unwrap());
            dest_dir.push(u.owner.unwrap());
            dest_dir.push(u.name);

            // if project has exist
            // then try to rename
            if Path::new(dest_dir.as_os_str()).exists() {
                let options: Vec<&str> = vec!["Override", "Rename", "Cancel"];

                let ans: Result<&str, InquireError> =
                    Select::new("Project has exist, I want to:", options).prompt();

                match ans {
                    Ok("Override") => {}
                    Ok("Rename") => {
                        let new_project_name = Text::new("Please enter the new name of project:")
                            .with_default(&(name + "-1"))
                            .prompt();

                        match new_project_name {
                            Ok(val) => {
                                dest_dir = dest_dir.parent().unwrap().join(val);
                            }
                            Err(_) => process::exit(0x0),
                        };
                    }
                    Ok("Cancel") => process::exit(0x0),
                    Ok(_) => process::exit(0x0),
                    Err(_) => process::exit(0x0),
                };
            }

            println!("{}", temp_clone_dir.to_str().unwrap());

            let temp_clone_dir_will_be_removed = temp_clone_dir.clone();

            // remove temp dir when cancel the action
            ctrlc::set_handler(move || {
                fs::remove_dir_all(temp_clone_dir_will_be_removed.as_path()).unwrap_err();
                println!("received Ctrl+C!");
            })
            .expect("Error setting Ctrl-C handler");

            let mut child = ChildProcess::new("git")
                .arg("clone")
                .arg("git@github.com:axetroy/prune.rs.git")
                .arg(temp_clone_dir.to_str().unwrap())
                .spawn()
                .expect("failed to execute child");

            let ecode = child.wait().expect("failed to wait on child");

            println!("rename to {}", dest_dir.to_str().unwrap());

            let temp_clone_dir_a = &temp_clone_dir.clone();
            let temp_clone_parent_dir = temp_clone_dir_a.as_path().parent();

            if !ecode.success() {
                // remove clone temp dir
                fs::remove_dir_all(temp_clone_parent_dir.unwrap()).unwrap_err();
                process::exit(ecode.code().unwrap_or(1));
            } else {
                // rename to dest
                match fs::rename(temp_clone_dir, dest_dir) {
                    Ok(_) => {
                        // remove clone temp dir
                        fs::remove_dir_all(temp_clone_parent_dir.unwrap()).unwrap_err();
                    }
                    Err(e) => {
                        // remove clone temp dir
                        fs::remove_dir_all(temp_clone_parent_dir.unwrap()).unwrap_err();

                        panic!("{}", e);
                    }
                }
            }
        }
        Some(("init", sub_matches)) => {
            println!(
                "Pushing to {}",
                sub_matches.value_of("REMOTE").expect("required")
            );
        }
        Some((ext, sub_matches)) => {
            let args = sub_matches
                .values_of_os("")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            println!("Calling out to {:?} with {:?}", ext, args);
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }

    // Continued program logic goes here...
}
