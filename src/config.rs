use std::{path::PathBuf, str::FromStr};

pub struct CliArgs {
    pub dir: Option<PathBuf>,
    pub max_depth: i8,
}

const HELP_TXT: &str = "\
gitpeek

Recursively searches for git directories & prints out their branch names.

USAGE:
    gitpeek [OPTIONS]

OPTIONS:
    -t TARGET    Sets the target directory to begin gitpeek. Default: current directory.
    -d MAX_DEPTH Sets max dept of directories to recursively search, starting from TARGET. Default: 1.
";

pub fn parse_args() -> Result<CliArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP_TXT);
        std::process::exit(0);
    }

    let arg = CliArgs {
        dir: pargs.opt_value_from_fn("-t", PathBuf::from_str)?,
        max_depth: pargs
            .value_from_fn("-d", |arg_str| -> Result<i8, &str> {
                arg_str.parse().map_err(|_| "not a number")
            })
            .unwrap_or(1),
    };

    Ok(arg)
}
