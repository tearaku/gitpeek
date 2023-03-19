use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use gitpeek::config::{self};
use gitpeek::gitseek::fetch_git_dir;

fn main() {
    let args = match config::parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Err: {}.", e);
            std::process::exit(1);
        }
    };

    let git_list = fetch_git_dir(args);
    if let Err(e) = git_list {
        eprintln!("Error in gitpeeking: {}", e);
        return ();
    }
    let git_list = git_list.unwrap();

    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the git directory to copy to clipboard (`q` to quit).")
        .default(0)
        .items(&git_list)
        .interact_opt()
        .unwrap();

    if let Some(selected) = selected {
        println!("Copied to clipboard! {}", git_list[selected]);
    } else {
        println!("Nothing selected, exiting...");
    }
}
