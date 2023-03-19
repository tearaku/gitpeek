use gitpeek::config::{self, CliArgs};
use std::{env, fs, path::PathBuf};

#[derive(Debug)]
enum SeekRes {
    NormalDir(PathBuf),
    GitDir(PathBuf),
}

fn get_dir_list(tar_dir: &PathBuf) -> Result<Vec<SeekRes>, String> {
    let tar_dir = tar_dir.to_owned();
    let _path: PathBuf = [tar_dir.clone(), ".git".into()].iter().collect();

    let dir = fs::read_dir(&tar_dir).map_err(|e| format!("Failed to read directory: {:?}", e))?;
    let dir_iter = dir.flatten().filter_map(|entry| -> Option<PathBuf> {
        let f_path = entry.path();
        match fs::metadata(&f_path) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    Some(f_path)
                } else {
                    None
                }
            }
            Err(_e) => None,
        }
    });
    let dir_col: Vec<PathBuf> = dir_iter.collect();
    match dir_col.contains(&_path) {
        true => Ok(vec![SeekRes::GitDir(tar_dir)]),
        false => Ok(dir_col
            .iter()
            .map(|d| SeekRes::NormalDir(d.to_owned()))
            .collect()),
    }
}

fn recursive_search(args: CliArgs, cur_dir: PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut git_dir: Vec<PathBuf> = Vec::new();
    let mut search_list = get_dir_list(&args.dir.unwrap_or(cur_dir))?;

    let mut cur_layer_res: Vec<SeekRes> = Vec::new();
    for _ in 0..=args.max_depth {
        let cur_layer_len = search_list.len();

        search_list
            .drain(0..cur_layer_len)
            .try_for_each(|item| -> Result<(), String> {
                match item {
                    SeekRes::NormalDir(dir) => {
                        cur_layer_res.append(&mut get_dir_list(&dir)?);
                        Ok(())
                    }
                    SeekRes::GitDir(dir) => {
                        git_dir.push(dir);
                        Ok(())
                    }
                }
            })?;

        search_list.append(&mut cur_layer_res);
    }
    Ok(git_dir)
}

fn main() {
    let args = match config::parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Err: {}.", e);
            std::process::exit(1);
        }
    };
    let cur_dir = env::current_dir().unwrap_or_default();
    match recursive_search(args, cur_dir) {
        Ok(git_paths) => {
            git_paths.iter().for_each(|path| println!("{:?}", path));
        }
        Err(msg) => println!("Error in gitpeek-ing: {}", msg),
    };
}
