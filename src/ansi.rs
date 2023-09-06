use std::io::Write;

pub const ESCAPE_CODE: &str = "\x1B";
static mut LINE_NUMBER: u16 = 1;

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
    print!("{}c", ESCAPE_CODE);
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

pub fn move_to(line: u16, column: u16) {
    print!("{}[{};{}H", ESCAPE_CODE, line, column);
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

pub fn cursor_pos() -> crossterm::Result<(u16, u16)> {
    print!("{}[6n", ESCAPE_CODE);
    flush();

    let mut key;
    let mut col = String::new();
    let mut row = String::new();
    let mut read_col = false;
    loop {
        key = crossterm::event::read()?;
        if let crossterm::event::Event::Key(key_event) = key {
            if let crossterm::event::KeyCode::Char(c) = key_event.code {
                if c == 'R' {
                    break;
                }
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    if c == ';' {
                        read_col = true;
                        continue;
                    }
                    if c == '[' {
                        continue;
                    }
                    if read_col {
                        col.push(c);
                    } else {
                        row.push(c);
                    }
                }
            }
        }
    }
    let row = row.parse::<u16>().unwrap_or(1);
    let col = col.parse::<u16>().unwrap_or(1);

    Ok((row, col))
}

