#![deny(warnings)]

mod configure;
mod file_explorer;
mod git;
mod util;
mod walker;

use clap::{arg, Arg, Command, PossibleValue};
use inquire::{error::InquireError, Confirm, Select, Text};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;

fn main() {
    let config_field_root = PossibleValue::new("root").help("The root of clones repository");

    let mut app = Command::new("gpm")
        .version(version.as_str())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
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
            Command::new("list")
                .alias("ls")
                .arg(
                    Arg::new("json")
                        .short('j')
                        .long("json")
                        .help("Print output as JSON format")
                        .takes_value(false),
                )
                .about("List cloned repositories"),
        )
        .subcommand(
            Command::new("open")
                .about("Open repository with file explorer")
                .arg(arg!(<REMOTE> "The remote Git URL to clone")),
        )
        .subcommand(
            Command::new("config")
                .about(
                    "The operation of configure, print the configure if sub-command not provide.",
                )
                .subcommand(
                    Command::new("add")
                        .about("Add configure for a field")
                        .arg(
                            Arg::new("FIELD")
                                .possible_value(config_field_root.to_owned())
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
                                .possible_value(config_field_root.to_owned())
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
                                .possible_value(config_field_root)
                                .required(true)
                                .help("The field of configure"),
                        )
                        .arg_required_else_help(true),
                )
                .subcommand(Command::new("reset").about("Reset configure")),
        );

    let gpm_rc = dirs::home_dir().unwrap().join(".gpmrc");

    let is_gpm_rc_exist = Path::new(gpm_rc.as_os_str()).exists();

    if !is_gpm_rc_exist {
        let mut file = File::create(gpm_rc.as_path()).expect("can not create a .gpmrc file");
        file.write_all(b"{\"root\": []}")
            .expect("can not write to $HOME/.gpmrc");
        drop(file);
    }

    let mut rc = configure::new(&gpm_rc).expect("invalid configure file");

    let matches = app.clone().get_matches();

    fn check_gpm_root(rc: &configure::Configure) {
        if rc.root.is_empty() {
            println!("Can not found root folder in the configure.\nTry running the following command to add a default folder:\n\n    gpm config add root $HOME/gpm\n\nOr set to a custom folder:\n\n    gpm config add root <folder>\n");
            process::exit(0x1);
        }
    }

    fn get_gpm_root(rc: &configure::Configure) -> &str {
        check_gpm_root(rc);

        if rc.root.len() == 1 {
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
        }
    }

    match matches.subcommand() {
        Some(("clone", sub_matches)) => {
            let url = sub_matches.value_of("REMOTE").expect("required");

            let clone_args = match sub_matches.values_of("OPTIONS") {
                Some(s) => s.collect::<Vec<&str>>(),
                _ => vec![],
            };

            let gpm_root: &str = get_gpm_root(&rc);

            let mut dest_dir = git::url_to_path(gpm_root, url).unwrap();

            // if project exist
            if dest_dir.exists() {
                let options: Vec<&str> = vec!["Auto", "Override", "Rename", "Open", "Cancel"];

                let ans: Result<&str, InquireError> =
                    Select::new("The project exist, then you want: ", options).prompt();

                dest_dir = match ans {
                    Ok("Auto") => util::find_available_path(dest_dir),
                    Ok("Override") => {
                        let ans = Confirm::new("Override means that the original project will be deleted, are you sure you want to continue?")
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
                            let dir_name = new_dest_dir
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string();

                            let help_msg =
                                format!("The project name '{}' already exists.", &dir_name);

                            let input =
                                Text::new("Enter the new name:").with_help_message(&help_msg);

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
                    _ => process::exit(0x0),
                }
            }

            let rm_dir = dest_dir.clone();

            // remove temp dir when cancel the action
            ctrlc::set_handler(move || {
                if rm_dir.exists() && fs::remove_dir_all(rm_dir.as_path()).is_ok() {}
            })
            .unwrap_or_else(|e| println!("Error setting Ctrl-C handler: {}", e));

            match git::clone(url, &dest_dir, clone_args) {
                Ok(()) => file_explorer::open(&dest_dir),
                _ => {
                    if dest_dir.exists() {
                        fs::remove_dir_all(dest_dir).unwrap();
                        process::exit(0x1);
                    }
                }
            }
        }
        Some(("config", sub_matches)) => match sub_matches.subcommand() {
            Some(("add", sub_matches)) => {
                let field = sub_matches.value_of("FIELD").expect("required");
                let value = sub_matches.value_of("VALUE").expect("required");

                rc.add_field(field, value, false).unwrap();
            }
            Some(("set", sub_matches)) => {
                let field = sub_matches.value_of("FIELD").expect("required");
                let value = sub_matches.value_of("VALUE").expect("required");
                rc.set_field(field, value, false).unwrap();
            }
            Some(("remove", _)) => {
                let field = sub_matches.value_of("FIELD").expect("required");
                rc.remove_field(field).unwrap();
            }
            Some(("reset", _)) => {
                rc.reset().unwrap();
            }
            _ => {
                println!("{}", rc);
                process::exit(0x0);
            }
        },
        Some(("list", sub_matches)) => {
            check_gpm_root(&rc);
            let is_output_as_json = sub_matches.is_present("json");

            if is_output_as_json {
                let mut repository_map: HashMap<String, Vec<String>> = HashMap::new();

                for gpm_root in rc.root {
                    let root = Path::new(&gpm_root);

                    let repositories = walker::walk_root(root)
                        .unwrap()
                        .into_iter()
                        .map(|v| v.as_os_str().to_str().unwrap().to_string())
                        .collect::<Vec<String>>();

                    repository_map.insert(gpm_root.clone(), repositories);
                }

                let serialized = serde_json::to_string(&repository_map).unwrap();

                println!("{}", serialized)
            } else {
                for gpm_root in rc.root {
                    let root = Path::new(&gpm_root);
                    let repositories = walker::walk_root(root).unwrap();
                    println!("{}", gpm_root);

                    for v in repositories {
                        println!("    {}", v.as_os_str().to_str().unwrap())
                    }
                }
            }
        }
        Some(("open", sub_matches)) => {
            let url = sub_matches.value_of("REMOTE").expect("required");
            let mut found: Vec<PathBuf> = vec![];

            for gpm_root in rc.root {
                let repo_dir = git::url_to_path(&gpm_root, url).unwrap();

                if repo_dir.exists() && repo_dir.is_dir() {
                    found.push(repo_dir.to_path_buf());
                }
            }

            if found.is_empty() {
                println!("Did not found the cloned repository '{}'", url);
                process::exit(0x1);
            }

            if found.len() == 1 {
                println!(
                    "Found the repository '{}'",
                    found[0].as_os_str().to_str().unwrap()
                );
                file_explorer::open(&found[0]);
                process::exit(0x0);
            }

            let options: Vec<&str> = found
                .iter()
                .map(|s| s.as_os_str().to_str().unwrap())
                .collect();

            let ans: Result<&str, InquireError> =
                Select::new("Select a repository to open:", options).prompt();

            match ans {
                Ok(choice) => file_explorer::open(Path::new(choice)),
                Err(_) => process::exit(0x0),
            }
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
