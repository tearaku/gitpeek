use std::{
    env::var,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use config::ConfigError;

pub struct CliArgs {
    pub dir: Option<PathBuf>,
    pub max_depth: i8,
    pub ignore_list: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Settings {
    max_depth: i8,
    ignore_list: Vec<String>,
}

const HELP_TXT: &str = "\
gitpeek

Recursively searches for git directories & prints out their branch names.
Option defaults are stored in config file. Command-line arguments take precedence over
config settings.

USAGE:
    gitpeek [OPTIONS]

OPTIONS:
    -t TARGET       Sets the target directory to begin gitpeek. Default: current directory.
    -d MAX_DEPTH    Sets max dept of directories to recursively search, starting from TARGET. Default: 1.
    -e IGNORE_LIST  Comma-separated list of directory names to ignore. Default: [].
";

fn parse_config_file() -> Result<Settings, ConfigError> {
    // TODO: add support for other OSes later
    let config_path = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME"))
        .map(|root| Path::new(&root).join(".config").join("gitpeek.json"))
        .map_err(|e| format!("Unable to find config home directory! Err: {}", e))
        .unwrap();

    let settings_res = config::Config::builder()
        .add_source(config::File::new(
            config_path.to_str().unwrap_or_default(),
            config::FileFormat::Json,
        ))
        .build();

    if let Err(ConfigError::Foreign(ref err)) = settings_res {
        println!(
            "{}.\nWriting empty config file at specified location...",
            err
        );

        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&config_path)
            .unwrap()
            .write_all(
                &serde_json::to_vec(&Settings {
                    max_depth: 1,
                    ignore_list: vec![],
                })
                .unwrap(),
            )
            .unwrap();

        config::Config::builder()
            .add_source(config::File::new(
                config_path.to_str().unwrap_or_default(),
                config::FileFormat::Json,
            ))
            .build()
            .unwrap()
            .try_deserialize::<Settings>()
    } else {
        settings_res?.try_deserialize::<Settings>()
    }
}

pub fn parse_args() -> Result<CliArgs, pico_args::Error> {
    let config_settings = parse_config_file().unwrap();

    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP_TXT);
        std::process::exit(0);
    }

    let arg = CliArgs {
        dir: pargs.opt_value_from_fn("-t", PathBuf::from_str)?,
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
}
