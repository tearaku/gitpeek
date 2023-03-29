use arboard::Clipboard;
use dialoguer::theme::ColorfulTheme;
use dialoguer::FuzzySelect;
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
        return;
    }
    let git_list = git_list.unwrap();

    let selected = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the git directory w/ array keys to copy to clipboard,\nor type for fuzzy finding (`Esc` to quit).")
        .default(0)
        .items(&git_list)
        .interact_opt()
        .unwrap();

    if let Some(selected) = selected {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard
            .set_text(["cd ".to_owned() + git_list[selected].dir.to_str().unwrap_or("")].join(" "))
            .unwrap();
        println!("Copied to clipboard!");
    } else {
        println!("Nothing selected, exiting...");
    }
}
