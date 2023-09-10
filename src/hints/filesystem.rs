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

pub fn make_directory_hints<'a>(dir: path::PathBuf, inlay: Option<&'a str>) -> super::Hint<'a> {
    let available_files = get_files_in_directory(&dir);
    let mut hint = super::Hint::new(available_files, inlay);
    hint.set_set_using(dir);
    hint
}

pub fn update_directory_hints(new_dir: &path::PathBuf, hints: &mut super::Hint) {
    if new_dir != hints.set_using() {
        let available_files = get_files_in_directory(new_dir);
        hints.set_selection(available_files);
        hints.set_set_using(new_dir.clone());
    }
}
