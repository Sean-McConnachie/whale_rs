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

// https://shane-o.dev/blog/binary-search-rust
pub fn binary_search<T: Ord>(k: T, items: &[T]) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let mut low: usize = 0;
    let mut high: usize = items.len() - 1;

    while low <= high {
        let middle = (high + low) / 2;
        if let Some(current) = items.get(middle) {
            if *current == k {
                return Some(middle);
            }
            if *current > k {
                if middle == 0 {
                    return None;
                }
                high = middle - 1
            }
            if *current < k {
                low = middle + 1
            }
        }
    }
    None
}

// https://shane-o.dev/blog/binary-search-rust
pub fn binary_search_with_exclude<T, R: Ord + ?Sized>(
    k: &R,
    value_func: impl Fn(&T) -> &R,
    items: &[T],
    exclude: &[usize],
) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let mut low: usize = 0;
    let mut high: usize = items.len() - 1;

    while low <= high {
        let middle = (high + low) / 2;
        if let Some(current) = items.get(middle) {
            if exclude.contains(&middle) { // TODO: Test if BS here is faster
                if middle == 0 {
                    return None;
                }
                high = middle - 1;
                continue;
            }
            let v = value_func(current);
            if v == k {
                return Some(middle);
            }
            if v > k {
                if middle == 0 {
                    return None;
                }
                high = middle - 1
            }
            if v < k {
                low = middle + 1
            }
        }
    }
    None
}
