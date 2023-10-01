use crate::enums::ArgType;

use serde::{Deserialize, Serialize};
use std::ffi;
use std::io::{Read, Write};
use std::path;

pub type CommandString = Option<String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCommand {
    /// The command name that should trigger this `Command` being used. I.e. `mv`. Note that this
    /// is generated using the file name.
    #[serde(skip, default)]
    pub exe_name: String,
    /// A direct mapping from the string that trigged using this command, to what should actually be
    /// executed. If left blank, the `exe_name` will be used.
    pub exe_to: String,

    /// A command that should be executed before executing the entered command.
    pub execute_before: CommandString,
    /// A command that should be executed after executing the entered command.
    pub execute_after: CommandString,

    #[serde(rename = "single_arg")]
    pub args: Vec<SingleArg>,
    #[serde(rename = "flag")]
    pub flags: Vec<Flag>,
    #[serde(rename = "flag_arg_pair")]
    pub arg_flags: Vec<FlagArgPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleArg {
    pub arg_type: ArgType,
    /// An inlay hint as to what the argument might do. E.g. the "<src>" in `mv <src>` before
    /// typing in the <src> field.
    pub arg_hint: String,
    /// Since arguments are unnamed parameters, and `.toml` files could be subject to reording, an
    /// explicit `arg_pos` must be given.
    pub arg_pos: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    pub flag_name: String,
    pub flag_to: String,

    pub execute_before: CommandString,
    pub execute_after: CommandString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagArgPair {
    pub flag_name: String,
    pub flag_to: String,

    pub arg_type: ArgType,
    #[serde(default)]
    pub arg_hint: String,

    pub execute_before: CommandString,
    pub execute_after: CommandString,
}

pub fn read_commands(command_dir: &path::PathBuf) -> Vec<ConfigCommand> {
    // iterate through each file in the command_dir
    // parse each file as a CommandConfig
    // return a vector of CommandConfig
    if !command_dir.exists() {
        std::fs::create_dir_all(&command_dir).unwrap();
        create_defaults(command_dir).unwrap();
    };
    let mut commands = vec![];
    for entry in command_dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == ffi::OsStr::new("toml") {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            let mut command: ConfigCommand = read_config(&path).unwrap();
            command.exe_name = file_name.to_string();

            { // `SingleArgs`
                let main_arg = SingleArg {
                    arg_type: ArgType::Executable,
                    arg_hint: "".to_string(),
                    arg_pos: 0,
                };
                command.args.push(main_arg);
                command.args.sort_by(|a, b| a.arg_pos.cmp(&b.arg_pos));
                // assert no duplicate arg_pos
                let mut prev_pos = if command.args.len() > 0 {
                    command.args[0].arg_pos
                } else {
                    0
                };
                for arg in command.args.iter().skip(1) {
                    assert!(arg.arg_pos > prev_pos);
                    prev_pos = arg.arg_pos;
                }
            }

            command.flags.sort_by(|a, b| a.flag_name.cmp(&b.flag_name));
            command
                .arg_flags
                .sort_by(|a, b| a.flag_name.cmp(&b.flag_name));
            commands.push(command);
        }
    }
    commands
}

fn read_config<P: AsRef<path::Path>, T: for<'de> Deserialize<'de>>(
    path: P,
) -> anyhow::Result<T> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(toml::from_str(&contents)?)
}

fn create_defaults(directory: &path::PathBuf) -> anyhow::Result<Vec<ConfigCommand>> {
    let move_default = ConfigCommand {
        exe_name: "mv".to_string(),
        exe_to: "move".to_string(),
        execute_before: None,
        execute_after: None,
        args: vec![
            SingleArg {
                arg_type: ArgType::Path,
                arg_hint: "src".to_string(),
                arg_pos: 1,
            },
            SingleArg {
                arg_type: ArgType::Path,
                arg_hint: "dst".to_string(),
                arg_pos: 2,
            },
        ],
        flags: vec![],
        arg_flags: vec![],
    };

    write_command_config(directory, &move_default)?;

    Ok(vec![])
}

fn write_command_config(directory: &path::PathBuf, command: &ConfigCommand) -> anyhow::Result<()> {
    let mut path = directory.join(&command.exe_name);
    path.set_extension(ffi::OsStr::new("toml"));
    let toml_str = toml::to_string_pretty(&command)?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(toml_str.as_bytes())?;
    Ok(())
}
