use std::cell::RefCell;
use std::path;
use std::rc::Rc;
use crate::ansi::TerminalXY;
use crate::buffer::InputBuffer;
use crate::gui::{ActionToTake, ActionType, ViewType};
use crate::gui::terminal::CursorPos;
use crate::input::InputEvent;
use crate::{ansi, state};
use crate::config::theme;

const NUM_COLS: usize = 3;

#[derive(Debug, Clone)]
struct DirInfo {
    scroll: usize,
    dir: path::PathBuf,
    entries: Vec<path::PathBuf>,
}

impl DirInfo {
    fn get_entries(dir: &path::PathBuf) -> Vec<path::PathBuf> {
        let mut files = Vec::new();
        for entry in dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            files.push(path);
        }
        files
    }

    fn new(dir: path::PathBuf) -> Self {
        let entries = Self::get_entries(&dir);
        Self {
            scroll: 0,
            dir,
            entries,
        }
    }

    fn new_to_parent(self) -> Self {
        let original = self.dir.clone();
        let parent = self.dir.parent().unwrap().to_path_buf();
        let mut s = Self::new(parent.clone());
        s.try_scroll_to(&original);
        s
    }

    fn try_scroll_to(&mut self, path: &path::PathBuf) {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry == path {
                self.scroll = i;
                return;
            }
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        } else {
            self.scroll = self.entries.len() - 1;
        }
    }

    fn scroll_down(&mut self) {
        if self.scroll < self.entries.len() - 1 {
            self.scroll += 1;
        } else {
            self.scroll = 0;
        }
    }

    fn get_entry(&self) -> &path::PathBuf {
        &self.entries[self.scroll]
    }

    fn draw(&self, start: TerminalXY, size: TerminalXY, theme: &theme::ConfigTheme) {
        fn ceil_div(a: usize, b: usize) -> usize {
            (a + b - 1) / b
        }

        let num_items = self.entries.len() as u16;
        let upper = size.1.min(num_items) as usize;
        let mid = ceil_div(upper, 2);

        for i in 0..upper {
            ansi::move_to((start.0, start.1 + i as u16));

            let ind = (i + self.scroll + mid) % self.entries.len();
            let entry = &self.entries[ind];
            let name = entry.file_name().unwrap().to_str().unwrap();

            let style = match entry.is_dir() {
                true => &theme.executable,
                false => &theme.console_secondary
            };

            let style_type = match ind == self.scroll {
                false => &style.normal,
                true => &style.highlighted,
            };

            super::output_str(style_type, name);
        }
    }
}

/// This struct stores all information as strings so it doesn't have to be reformatted on each draw.
struct FileInfo {
    path: path::PathBuf,

    // TODO: Make ftype an enum, then add additional_details. E.g. Png gives image resolution, mp3 gives length, etc.
    ftype: String,
    size: String,

    created_at: String,
    modified_at: String,
    accessed_at: String,

}

impl FileInfo {
    fn new(path: path::PathBuf) -> Self {
        let ftype = match path.extension() {
            Some(ext) => ext.to_str().unwrap().to_string(),
            None => String::new(),
        };

        let metadata = path.metadata().unwrap();

        let size = {
            let l = metadata.len();
            if l < 1024 {
                format!("{} B", l)
            } else if l < 1024 * 1024 {
                format!("{} KB", l / 1024)
            } else if l < 1024 * 1024 * 1024 {
                format!("{} MB", l / (1024 * 1024))
            } else {
                format!("{} GB", l / (1024 * 1024 * 1024))
            }
        };

        fn format_time(t: std::time::SystemTime) -> String {
            // yyyy-mm-dd hh:mm:s
            let t = t.duration_since(std::time::UNIX_EPOCH).unwrap();
            let t = t.as_secs();
            let t = chrono::NaiveDateTime::from_timestamp_opt(t as i64, 0);
            let t = t.unwrap();
            let t = t.format("%Y-%m-%d %H:%M:%S");
            t.to_string()
        }

        let created_at = format_time(metadata.created().unwrap());
        let modified_at = format_time(metadata.modified().unwrap());
        let accessed_at = format_time(metadata.accessed().unwrap());

        Self {
            path,
            ftype,
            size,
            created_at,
            modified_at,
            accessed_at,
        }
    }

    fn draw(&self, start: TerminalXY, theme: &theme::ConfigTheme) {
        let title = &theme.console_main.normal;
        let value = &theme.console_secondary.normal;
        let (x, y) = start;

        ansi::move_to((x, y + 0));
        super::output_str(title, "Path: ");
        super::output_str(value, self.path.to_str().unwrap());

        ansi::move_to((x, y + 2));
        super::output_str(title, "Type: ");
        super::output_str(value, &self.ftype);

        ansi::move_to((x, y + 4));
        super::output_str(title, "Size: ");
        super::output_str(value, &self.size);

        ansi::move_to((x, y + 6));
        super::output_str(title, "Created at: ");
        super::output_str(value, &self.created_at);

        ansi::move_to((x, y + 8));
        super::output_str(title, "Modified at: ");
        super::output_str(value, &self.modified_at);

        ansi::move_to((x, y + 10));
        super::output_str(title, "Accessed at: ");
        super::output_str(value, &self.accessed_at);
    }
}

enum FinalCol {
    Dir(DirInfo),
    File(FileInfo),
}

impl FinalCol {
    fn draw(&self, start: TerminalXY, size: TerminalXY, theme: &theme::ConfigTheme) {
        match self {
            Self::Dir(dir) => dir.draw(start, size, theme),
            Self::File(file) => file.draw(start, theme)
        }
    }
}

pub struct FileExplorerGUI {
    program_state: Rc<RefCell<state::ProgramState>>,

    num_rows: u16,
    col_width: usize,

    col_lef: DirInfo,
    col_mid: DirInfo,
    col_rig: FinalCol,
}

impl FileExplorerGUI {
    pub fn init(program_state: Rc<RefCell<state::ProgramState>>) -> Self {
        let curr_dir = program_state.borrow().current_working_directory.clone();
        let mut col_lef = DirInfo::new(curr_dir.parent().unwrap().to_path_buf());
        col_lef.try_scroll_to(&curr_dir);
        let col_mid = DirInfo::new(curr_dir.clone());
        let col_rig = Self::resolve_final_col(&col_mid);

        Self {
            col_width: 0,
            num_rows: 0,
            program_state,
            col_lef,
            col_mid,
            col_rig,
        }
    }

    fn resolve_final_col(dir_info: &DirInfo) -> FinalCol {
        let entry = dir_info.get_entry();
        match entry.is_dir() {
            true => FinalCol::Dir(DirInfo::new(entry.clone())),
            false => FinalCol::File(FileInfo::new(entry.clone())),
        }
    }
}

impl super::GUITrait for FileExplorerGUI{
    fn view_type(&self) -> ViewType {
        ViewType::Explorer
    }

    #[allow(unused_variables)]
    fn action_before_write(
        &mut self,
        event: InputEvent,
        buffer: &InputBuffer,
        term_size: TerminalXY,
        write_from_line: u16,
        cursor_pos: CursorPos,
        arg_pos: CursorPos,
    ) -> ActionToTake {
        self.num_rows = term_size.1 - write_from_line;
        self.col_width = (term_size.0 / NUM_COLS as u16).saturating_sub(1) as usize;

        let mut action_to_take = ActionToTake::WriteBuffer(ActionType::Standard);
        match event {
            InputEvent::ArrowUp => {
                self.col_mid.scroll_up();
                self.col_rig = Self::resolve_final_col(&self.col_mid);
                action_to_take = ActionToTake::BlockBuffer
            }
            InputEvent::ArrowDown => {
                self.col_mid.scroll_down();
                self.col_rig = Self::resolve_final_col(&self.col_mid);
                action_to_take = ActionToTake::BlockBuffer
            }
            InputEvent::ArrowLeft => {
                let tmp = self.col_mid.clone();
                self.col_mid = self.col_lef.clone();
                self.col_rig = FinalCol::Dir(tmp);
                self.col_lef = self.col_lef.clone().new_to_parent();
                action_to_take = ActionToTake::BlockBuffer
            }
            InputEvent::ArrowRight => {
                match &self.col_rig {
                    FinalCol::Dir(dir) => {
                        if !dir.entries.is_empty() {
                            let tmp = self.col_mid.clone();
                            self.col_mid = dir.clone();
                            self.col_rig = Self::resolve_final_col(&self.col_mid);
                            self.col_lef = tmp;
                        }
                    }
                    FinalCol::File(_) => {}
                }
                action_to_take = ActionToTake::BlockBuffer
            }
            _ => {}
        }
        action_to_take
    }

    #[allow(unused_variables)]
    fn write_output(
        &mut self,
        event: InputEvent,
        term_size: TerminalXY,
        write_from_line: u16,
        buf: &InputBuffer,
    ) {
        ansi::move_to((0, write_from_line));
        ansi::erase_screen_from_cursor();

        let program_state = self.program_state.borrow();
        let theme = &program_state.config.theme;

        self.col_lef.draw((0, write_from_line), (self.col_width as u16, self.num_rows), theme);
        self.col_mid.draw((self.col_width as u16, write_from_line), (self.col_width as u16, self.num_rows), theme);
        self.col_rig.draw((self.col_width as u16 * 2, write_from_line), (self.col_width as u16, self.num_rows), theme);
    }

    #[allow(unused_variables)]
    fn clear_output(&mut self, write_from_line: u16) -> () {}
}