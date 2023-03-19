use super::config::CliArgs;
use core::fmt;
use std::{env, fmt::Display, fs, io, path::PathBuf};

pub struct SeekRes {
    pub dir: PathBuf,
    pub git_ref: String,
}

enum DirType {
    NormalDir(PathBuf),
    GitDir(PathBuf),
}

pub fn fetch_git_dir(args: CliArgs) -> Result<Vec<SeekRes>, String> {
    let cur_dir = env::current_dir().unwrap_or_default();
    let seek_res_list: Vec<SeekRes> = recursive_search(args, cur_dir)?
        .iter()
        .map(|path| {
            let git_ref = get_git_ref(path.to_path_buf()).unwrap_or_default();
            SeekRes {
                dir: path.to_owned(),
                git_ref,
            }
        })
        .collect();
    Ok(seek_res_list)
}

fn recursive_search(args: CliArgs, cur_dir: PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut git_dir: Vec<PathBuf> = Vec::new();
    let mut search_list = get_dir_list(&args.dir.unwrap_or(cur_dir))?;

    let mut cur_layer_res: Vec<DirType> = Vec::new();
    for _ in 0..=args.max_depth {
        let cur_layer_len = search_list.len();

        search_list
            .drain(0..cur_layer_len)
            .try_for_each(|item| -> Result<(), String> {
                match item {
                    DirType::NormalDir(dir) => {
                        cur_layer_res.append(&mut get_dir_list(&dir)?);
                        Ok(())
                    }
                    DirType::GitDir(dir) => {
                        git_dir.push(dir);
                        Ok(())
                    }
                }
            })?;

        search_list.append(&mut cur_layer_res);
    }
    Ok(git_dir)
}

fn get_git_ref(path: PathBuf) -> io::Result<String> {
    let path: PathBuf = [path, ".git".into()].iter().collect();
    let dir_res = fs::read_dir(path)?;
    let git_ref_str: String = dir_res
        .filter_map(|f| {
            if let Ok(file) = f {
                if file.file_name() == "HEAD" {
                    return fs::read_to_string(file.path()).ok();
                }
            }
            None::<String>
        })
        // If multiple Some's are returned, then they'll be concatenated!
        .collect();
    let git_branch = git_ref_str
        // Trimming newline
        .strip_suffix('\n')
        .or_else(|| git_ref_str.strip_suffix("\r\n"))
        .unwrap_or(&git_ref_str)
        // Feetching branch name
        .split("ref: refs/heads/")
        .collect::<Vec<&str>>()
        .get(1)
        .unwrap_or(&"")
        .to_string();

    Ok(git_branch)
}

fn get_dir_list(tar_dir: &PathBuf) -> Result<Vec<DirType>, String> {
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
        true => Ok(vec![DirType::GitDir(tar_dir)]),
        false => Ok(dir_col
            .iter()
            .map(|d| DirType::NormalDir(d.to_owned()))
            .collect()),
    }
}

impl Display for SeekRes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.dir.to_str().unwrap_or(""), self.git_ref)
    }
}
