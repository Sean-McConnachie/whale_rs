use std::cmp::Ordering;
use std::{
    fs,
    io::{self, BufRead},
    path,
};

#[cfg(target_os = "windows")]
const DELIMITER: char = '\\';
#[cfg(target_os = "linux")]
const DELIMITER: char = '/';

pub fn short_path(full_path: &path::PathBuf) -> String {
    let mut output = full_path.to_str().unwrap()[0..2].to_string();
    output.push(DELIMITER);
    let dirs = full_path
        .iter()
        .map(|dir| dir.to_str().unwrap())
        .collect::<Vec<&str>>();
    for i in 2..dirs.len() - 1 {
        let dir = dirs[i];
        output.push_str(&dir[0..1]);
        output.push(DELIMITER);
    }
    output.push_str(dirs.last().unwrap());
    output.push_str(" > ");
    output
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where
    P: AsRef<path::Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn appendable_file<P>(filename: P) -> Result<fs::File, io::Error>
where
    P: AsRef<path::Path>,
{
    fs::OpenOptions::new()
        .read(false)
        .append(true)
        .create(false)
        .open(filename)
}

pub fn first_item(array: &[String], target: &str) -> Option<usize> {
    // TODO: This is probably slow
    for (i, item) in array.iter().enumerate() {
        if item.starts_with(target) {
            return Some(i);
        }
    }
    None
}

pub fn binary_search<T: Ord>(array: &[T], target: T) -> Option<usize> {
    let mut low = 0;
    let mut high = array.len() - 1;
    while low <= high {
        let mid = (low + high) / 2;
        match array[mid].cmp(&target) {
            Ordering::Less => low = mid + 1,
            Ordering::Equal => return Some(mid),
            Ordering::Greater => high = mid - 1,
        }
    }
    None
}
