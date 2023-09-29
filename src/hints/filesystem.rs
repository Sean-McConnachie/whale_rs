use std::{fs, path};

fn get_files_in_directory(dir: &path::PathBuf) -> Vec<String> {
    match fs::read_dir(dir) {
        Err(_) => vec![],
        Ok(files) => files
            .into_iter()
            .map(|f| {
                f.unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect(),
    }
}

pub fn make_directory_hints(dir: Option<(path::PathBuf, super::Disregard, String)>, inlay: Option<String>) -> super::Hint {
    if dir.is_none() {
        return super::Hint::new(vec![], inlay);
    }
    let (dir, disregard, arg) = dir.unwrap();
    let available_files = get_files_in_directory(&dir);
    let mut hint = super::Hint::new(available_files, inlay);
    hint.set_set_using(dir);
    hint.set_disregard(disregard);
    hint.closest_match(&arg);
    hint
}

pub fn update_directory_hints(new_dir: &Option<(path::PathBuf, super::Disregard, String)>, hints: &mut super::Hint) {
    match new_dir {
        None => {
            hints.set_selection(vec![]);
            hints.set_set_using(path::PathBuf::new());
            hints.set_disregard(0);
        }
        Some((new_dir, disregard, arg)) => {
            hints.set_disregard(*disregard);
            if !new_dir.exists() {
                hints.set_selection(vec![]);
                hints.set_set_using(path::PathBuf::new());
            } else {
                let available_files = get_files_in_directory(new_dir);
                hints.set_selection(available_files);
                hints.set_set_using(new_dir.clone());
                hints.closest_match(&arg);
            }
        }
    }
    //
    // let new_dir = new_dir.as_ref().unwrap();
    // if new_dir != hints.set_using() {
    //     let available_files = get_files_in_directory(new_dir);
    //     hints.set_selection(available_files);
    //     hints.set_set_using(new_dir.clone());
    // }
}
