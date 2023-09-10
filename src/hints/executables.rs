lazy_static::lazy_static! {
    static ref EXECUTABLES: Vec<String> = get_executables();
}

fn get_executables() -> Vec<String> {
    let output = std::process::Command::new("where")
        .args(["*.exe"])
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let mut executables = output
        .split("\r\n")
        .filter(|line| !line.is_empty())
        .map(|line| {
            let split = line.split("\\").collect::<Vec<&str>>();
            split.last().unwrap()[..split.last().unwrap().len() - 4].to_string()
        })
        .collect::<Vec<String>>();
    executables.sort();
    executables
}

pub fn make_executables_hint() -> super::Hint<'static> {
    super::Hint::new(EXECUTABLES.clone(), None)
}
