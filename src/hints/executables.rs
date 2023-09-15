use std::{path, process};

lazy_static::lazy_static! {
    static ref EXECUTABLES: Vec<String> = get_executables();
}

fn get_executables_windows() -> Result<process::Output, std::io::Error> {
    process::Command::new("where")
        .args(["*.exe"])
        .output()
}

fn get_executables_linux() -> Result<process::Output, std::io::Error> {
    process::Command::new("sh")
        .arg("-c")
        .arg("echo $PATH | tr ':' '\\n' | xargs -I {} find {} -maxdepth 1 -type f -executable -print")
        .output()

    // process::Command::new("echo")
    //     .args(["$PATH", "|", "tr", ":", "\n", "|", "xargs", "-I", "{}", "find", "{}", "-maxdepth", "1", "-type", "f", "-executable", "-print"])
    //     .output()
}

fn get_executables() -> Vec<String> {
    let (split, output) = if cfg!(target_os = "windows") {
        ("\r\n", get_executables_windows())
    } else if cfg!(target_os = "linux") {
        ("\n", get_executables_linux())
    } else {
        unimplemented!()
    };
    let output = output
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let mut executables = output
        .split(split)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let p = path::PathBuf::from(line);
            p.file_name().unwrap().to_str().unwrap().to_string()
        })
        .collect::<Vec<String>>();
    executables.sort();
    executables
}

pub fn make_executables_hint() -> super::Hint<'static> {
    super::Hint::new(EXECUTABLES.clone(), None)
}
