use std::io::{Read, Write};

pub const ESCAPE_CODE: &str = "\x1B";
static mut LINE_NUMBER: u16 = 1;

pub type TerminalXY = (u16, u16);

pub fn line_number_get() -> u16 {
    unsafe { LINE_NUMBER }
}

pub fn line_number_set(line_number: u16) {
    if line_number <= 0 {
        unsafe {
            LINE_NUMBER = 1;
        }
    }
    unsafe {
        LINE_NUMBER = line_number;
    }
}

pub fn reset() {
    print!("{}[0m", ESCAPE_CODE);
}

pub fn flush() {
    std::io::stdout().flush().unwrap();
}

pub fn move_left(n: u16) {
    print!("{}[{}D", ESCAPE_CODE, n);
}

pub fn move_right(n: u16) {
    print!("{}[{}C", ESCAPE_CODE, n);
}

pub fn move_up(n: u16) {
    print!("{}[{}A", ESCAPE_CODE, n);
}

pub fn move_down(n: u16) {
    print!("{}[{}B", ESCAPE_CODE, n);
}

pub fn move_to_column(column: u16) {
    print!("{}[{}G", ESCAPE_CODE, column);
}

pub fn move_to_line(line: u16) {
    print!("{}[{};0H", ESCAPE_CODE, line);
}

/// 0 indexed
pub fn move_to((x, y): TerminalXY) {
    print!("{}[{};{}H", ESCAPE_CODE, y + 1, x + 1);
}

pub fn erase_line() {
    print!("{}[2K", ESCAPE_CODE);
}

pub fn erase_line_from_cursor() {
    print!("{}[0K", ESCAPE_CODE);
}

pub fn erase_screen_from_cursor() {
    print!("{}[0J", ESCAPE_CODE);
}

pub fn erase_screen() {
    print!("{}[2J", ESCAPE_CODE);
}

pub fn cursor_hide() {
    print!("{}[?25l", ESCAPE_CODE);
}

pub fn cursor_show() {
    print!("{}[?25h", ESCAPE_CODE);
}

pub fn cursor_save() {
    print!("{}[s", ESCAPE_CODE);
}

pub fn cursor_restore() {
    print!("{}[u", ESCAPE_CODE);
}

/// 0-indexed
pub fn cursor_pos() -> Result<TerminalXY, std::io::Error> {
    print!("{}[6n", ESCAPE_CODE);
    flush();

    let mut c: char;
    let mut col = String::new();
    let mut row = String::new();
    let mut read_col = false;

    loop {
        // a generic error
        c = std::io::stdin().bytes().next().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not read from stdin",
        ))?? as char;
        if c == 'R' {
            break;
        }

        match c {
            ';' => read_col = true,
            '[' => continue,
            _ => {
                if !c.is_digit(10) { continue; }
                if read_col {
                    col.push(c);
                } else {
                    row.push(c);
                }
            }
        }
    }
    let row = row.parse::<u16>().unwrap_or(1);
    let col = col.parse::<u16>().unwrap_or(1);

    Ok((col - 1, row - 1))
}


