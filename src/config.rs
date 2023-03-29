use std::{
    env::var,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use config::{Config, ConfigError};
use pico_args::Arguments;

pub struct CliArgs {
    pub dir: PathBuf,
    pub max_depth: i8,
    pub ignore_list: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Settings {
    max_depth: i8,
    ignore_list: Vec<String>,
}

enum CmdResult {
    GetConfig,
    SetConfig,
    GitPeek(PathBuf),
}

const CMD_FLAGS: &[&str] = &["-d", "-e"];

const HELP_TXT: &str = "\
gitpeek

USAGE:
    gitpeek [COMMAND ... | TARGET] [OPTIONS] 

Recursively searches for git directories starting from TARGET & prints out their branch names.
If not specified, TARGET defaults to current directory.
Option defaults are stored in config file. Command-line arguments take precedence over config settings.

OPTIONS:
    -d MAX_DEPTH    Sets max dept of directories to recursively search, starting from TARGET. Default: 1.
    -e IGNORE_LIST  Comma-separated list of directory names to ignore. Default: [].

COMMAND:
    getconfig                  Displays the values stored in config file.
    setconfig [FIELD] [VALUE]  Sets the config field FIELD to the value VALUE.

EXAMPLE:
    gitpeek setconfig max_depth 3
    gitpeek ~/Documents
";

fn parse_config_file() -> Result<Settings, ConfigError> {
    let settings_res = build_config();

    if let Err(ConfigError::Foreign(ref err)) = settings_res {
        println!(
            "{}.\nWriting empty config file at specified location...",
            err
        );

        write_to_config_file(
            &Settings {
                max_depth: 1,
                ignore_list: vec![],
            },
            true,
        )
        .unwrap();

        build_config().unwrap().try_deserialize::<Settings>()
    } else {
        settings_res?.try_deserialize::<Settings>()
    }
}

fn process_command(pargs: &mut Arguments, settings: &mut Settings) -> Result<CmdResult, String> {
    // To get successive values --> call this multiple times (they're drained)
    if let Ok(Some(cmd)) = pargs.opt_free_from_str::<String>() {
        match cmd.as_str() {
            "getconfig" => {
                let value = serde_json::to_string(settings).unwrap();
                println!("{}", value);
                Ok(CmdResult::GetConfig)
            }
            "setconfig" => {
                let field = pargs.free_from_str::<String>().map_err(|e| e.to_string())?;
                let value = pargs.free_from_str::<String>().map_err(|e| e.to_string())?;
                settings.mutate(&field, &value)?;
                write_to_config_file(settings, false)?;
                println!("Config updated.");
                Ok(CmdResult::SetConfig)
            }
            cmd_str => {
                if CMD_FLAGS.contains(&cmd_str) {
                    Err(
                        "Incorrect usage. TARGET must be specified & placed before using OPTIONS"
                            .to_string(),
                    )
                } else {
                    let target_dir = cmd;
                    // from_str here is of type infallible
                    Ok(CmdResult::GitPeek(PathBuf::from_str(&target_dir).unwrap()))
                }
            }
        }
    } else {
        Ok(CmdResult::GitPeek(PathBuf::from_str(".").unwrap()))
    }
}

pub fn parse_args() -> Result<CliArgs, pico_args::Error> {
    let mut config_settings = parse_config_file().unwrap();

    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP_TXT);
        std::process::exit(0);
    }

    let cmd_res = process_command(&mut pargs, &mut config_settings)
        .map_err(|e| pico_args::Error::ArgumentParsingFailed { cause: e })?;

    if let CmdResult::GitPeek(tar_dir) = cmd_res {
        let arg = CliArgs {
            dir: tar_dir,
            max_depth: pargs
                .opt_value_from_fn("-d", |arg_str: &str| -> Result<i8, &str> {
                    arg_str.parse().map_err(|_| "not a number")
                })?
                .unwrap_or(config_settings.max_depth),
            ignore_list: pargs
                .opt_value_from_fn("-e", |arg_str: &str| -> Result<Vec<String>, &str> {
                    Ok(arg_str.to_owned().split(',').map(String::from).collect())
                })?
                .unwrap_or(config_settings.ignore_list),
        };

        Ok(arg)
    } else {
        std::process::exit(0);
    }
}

impl Settings {
    fn mutate(&mut self, field: &str, value: &str) -> Result<(), String> {
        match field {
            "ignore_list" => {
                self.ignore_list = value.split(',').map(String::from).collect();
                Ok(())
            }
            "max_depth" => {
                self.max_depth = str::parse::<i8>(value)
                    .map_err(|e| format!("`max_depth` value should be integer. ({})", e))?;
                Ok(())
            }
            cmd => Err(format!("Subcommand {} does not exist.", cmd)),
        }
    }
}

/*
 * Utility functions
 */
// TODO: add support for other OSes later
fn get_config_file_path() -> Result<PathBuf, String> {
    var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME"))
        .map(|root| Path::new(&root).join(".config").join("gitpeek.json"))
        .map_err(|e| format!("Unable to find config home directory! Err: {}", e))
}

fn write_to_config_file(value: &Settings, create_new: bool) -> Result<(), String> {
    let config_path = get_config_file_path().unwrap();
    OpenOptions::new()
        .write(true)
        .create_new(create_new)
        .open(config_path)
        .unwrap()
        .write_all(&serde_json::to_vec(value).unwrap())
        .map_err(|e| format!("Unable to write to config file: {}", e))
}

fn build_config() -> Result<Config, ConfigError> {
    let config_path = get_config_file_path().unwrap();
    config::Config::builder()
        .add_source(config::File::new(
            config_path.to_str().unwrap_or_default(),
            config::FileFormat::Json,
        ))
        .build()
}
