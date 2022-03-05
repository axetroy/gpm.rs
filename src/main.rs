#![deny(warnings)]
mod file_explorer;
mod git;
mod util;
use clap::{arg, Arg, Command};
use inquire::{error::InquireError, Confirm, Select, Text};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process;

#[derive(Serialize, Deserialize, Debug)]
struct Preset {
    root: Vec<String>,
}

fn main() {
    let mut app = Command::new("gpm")
        .version("v0.1.4")
        .author("Axetroy <axetroy.dev@gmail.com>")
        .about("A command line tool, manage your hundreds of repository, written with Rust")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .allow_invalid_utf8_for_external_subcommands(true)
        .subcommand(
            Command::new("clone")
                .about("Clones repository")
                .arg(arg!(<REMOTE> "The remote Git URL to clone"))
                .arg(
                    Arg::new("OPTIONS")
                        .required(false)
                        .multiple_occurrences(true)
                        .help("The git clone flags. eg. --progress --recursive"),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("config")
                .about("The operation of configure, print the configure if command not provide.")
                .subcommand(
                    Command::new("add")
                        .about("Add configure for a field")
                        .arg(
                            Arg::new("FIELD")
                                .possible_value("root")
                                .required(true)
                                .help("The field of configure"),
                        )
                        .arg(arg!(<VALUE> "The value of the field"))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("set")
                        .about("Set configure for a field")
                        .arg(
                            Arg::new("FIELD")
                                .possible_value("root")
                                .required(true)
                                .help("The field of configure"),
                        )
                        .arg(arg!(<VALUE> "The value of the field"))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("remove")
                        .about("Remove configure for a field")
                        .arg(
                            Arg::new("FIELD")
                                .possible_value("root")
                                .required(true)
                                .help("The field of configure"),
                        )
                        .arg_required_else_help(true),
                )
                .subcommand(Command::new("reset").about("Reset configure")),
        );

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

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        Some(("clone", sub_matches)) => {
            let url = sub_matches.value_of("REMOTE").expect("required");

            let clone_args = match sub_matches.values_of("OPTIONS") {
                Some(s) => s.collect::<Vec<&str>>(),
                _ => vec![],
            };

            let gpm_root: &str = if rc.root.is_empty() {
                println!("Can not found root folder in the configure for clone. Try running the following command:\n\n    gpm config add root <folder>\n");
                process::exit(0x1);
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

            let mut dest_dir = git::url_to_path(gpm_root, url).unwrap();

            // if project exist
            if dest_dir.exists() {
                let options: Vec<&str> = vec!["Auto", "Override", "Rename", "Open", "Cancel"];

                let ans: Result<&str, InquireError> =
                    Select::new("The project exist, then you want: ", options).prompt();

                dest_dir = match ans {
                    Ok("Auto") => util::find_available_path(dest_dir),
                    Ok("Override") => {
                        let ans = Confirm::new("Override means that the original project will be deleted, are you sure you want to continue??")
                            .with_default(false)
                            .with_help_message(
                                "[DANGER]: The data cannot be restored.",
                            )
                            .prompt();

                        match ans {
                            Ok(true) => fs::remove_dir_all(dest_dir.clone()).unwrap(),
                            Ok(false) => process::exit(0x0),
                            Err(_) => process::exit(0x0),
                        };

                        dest_dir
                    }
                    Ok("Rename") => {
                        let mut new_dest_dir = dest_dir.clone();

                        while new_dest_dir.exists() {
                            let input = Text::new("Enter the new name:")
                                .with_help_message("The project name is exists");

                            new_dest_dir = match input.prompt() {
                                Ok(name) => new_dest_dir.parent().unwrap().join(name),
                                Err(_) => process::exit(0x0),
                            }
                        }

                        new_dest_dir
                    }
                    Ok("Open") => {
                        file_explorer::open(&dest_dir);

                        process::exit(0x0)
                    }
                    Ok(_) => process::exit(0x0),
                    Err(_) => process::exit(0x0),
                }
            }

            let dest_dir_will_be_removed = dest_dir.clone();

            // remove temp dir when cancel the action
            ctrlc::set_handler(move || {
                if dest_dir_will_be_removed.exists() {
                    fs::remove_dir_all(dest_dir_will_be_removed.as_path()).unwrap();
                }
            })
            .unwrap_or_else(|e| println!("Error setting Ctrl-C handler: {}", e));

            match git::clone(url, &dest_dir, clone_args) {
                Ok(true) => file_explorer::open(&dest_dir),
                _ => {
                    if dest_dir.exists() {
                        fs::remove_dir_all(dest_dir).unwrap();
                        process::exit(0x1);
                    }
                }
            }
        }
        Some(("config", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("add", sub_matches)) => {
                    let field = sub_matches.value_of("FIELD").expect("required");
                    let value = sub_matches.value_of("VALUE").expect("required");

                    match field {
                        "root" => {
                            let add_abs_root_path =
                                &util::get_absolute_path(&Path::new(value)).unwrap();

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
                                    Ok(false) => process::exit(0x0),
                                    Err(_) => process::exit(0x0),
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
                            let add_abs_root_path =
                                &util::get_absolute_path(&Path::new(value)).unwrap();

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
                                    Ok(false) => process::exit(0x0),
                                    Err(_) => process::exit(0x0),
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
                _ => {
                    let serialized = serde_json::to_string(&rc).unwrap();

                    println!("{}", serialized);
                    process::exit(0x0);
                }
            }

            let serialized = serde_json::to_string(&rc).unwrap();

            fs::write(gpm_rc, serialized).expect("can not write to $HOME/.gpmrc");
        }
        Some((ext, sub_matches)) => {
            let args = sub_matches
                .values_of_os("")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            println!("Unknown the command {:?} with argument {:?}", ext, args);
            app.print_help().unwrap();
            process::exit(0x1);
        }
        _ => unreachable!(),
    }

    // Continued program logic goes here...
}
