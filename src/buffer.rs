use std::cell::RefCell;
use crate::{config::command, enums, hints, parser, state};
use std::path;
use std::rc::Rc;
use crate::hints::Disregard;
use crate::history::ux_layer;

const BUFFER_LENGTH: usize = 8192;

pub type BufferPosition = usize;

#[derive(Debug, PartialEq)]
pub enum Side {
    Left,
    Right,
    Neither,
}

#[derive(Debug)]
pub enum CursorType {
    Main,
    Secondary,
}

#[derive(Debug, PartialEq)]
pub struct Cursor {
    position: BufferPosition,
    active: bool,
}

impl Cursor {
    pub fn new(position: usize, active: bool) -> Self {
        Self { position, active }
    }

    pub fn position(&self) -> BufferPosition {
        self.position
    }

    pub fn active(&self) -> bool {
        self.active
    }
}

#[derive(Debug)]
pub struct InputBuffer {
    buffer: [char; BUFFER_LENGTH],
    input_length: usize,

    main_cursor: Cursor,
    secondary_cursor: Cursor,

    /// 1D array of start and stop of arguments.
    /// For all args, argstart = index * 2, argstop = index * 2 + 1
    split_locs: Vec<BufferPosition>,

    quote_locs: Vec<BufferPosition>,

    program_state: Rc<RefCell<state::ProgramState>>,
    argument_hints: Vec<(enums::ArgType, hints::Hint)>,

    history: ux_layer::History,

    curr_arg: usize,
}

impl InputBuffer {
    pub fn init(program_state: Rc<RefCell<state::ProgramState>>) -> Self {
        let history = ux_layer::History::init(program_state.clone());
        Self {
            buffer: ['\0'; BUFFER_LENGTH],
            input_length: 0,
            main_cursor: Cursor::new(0, true),
            secondary_cursor: Cursor::new(0, false),
            split_locs: Vec::new(),
            quote_locs: Vec::new(),
            argument_hints: Vec::new(),
            history,
            program_state,
            curr_arg: 0,
        }
    }

    pub fn get_buffer(&self) -> &[char] {
        &self.buffer[..self.input_length]
    }

    pub fn get_buffer_mut(&mut self) -> &mut [char] {
        &mut self.buffer[..self.input_length]
    }

    pub fn get_buffer_str(&self, (start, stop): (BufferPosition, BufferPosition)) -> String {
        self.buffer[start..stop].iter().collect()
    }

    pub fn get_buffer_range(&self, start: BufferPosition, stop: BufferPosition) -> &[char] {
        &self.buffer[start..stop]
    }

    pub fn get_quotes(&self) -> &[BufferPosition] {
        &self.quote_locs
    }

    pub fn get_argument_hints(&self) -> &[(enums::ArgType, hints::Hint)] {
        &self.argument_hints
    }

    pub fn set_closest_match_on_hint(&mut self, arg: usize, s: String) {
        self.argument_hints[arg].1.set_closest_match(s);
    }

    pub fn get_splits(&self) -> &[BufferPosition] {
        &self.split_locs
    }

    pub fn get_curr_arg(&self) -> usize {
        self.curr_arg
    }

    pub fn len(&self) -> usize {
        self.input_length
    }

    pub fn arg_locs_iterator(&self) -> impl Iterator<Item=(BufferPosition, BufferPosition)> + '_ {
        (0..self.num_args())
            .into_iter()
            .map(|k| (self.split_locs[k * 2], self.split_locs[k * 2 + 1]))
    }

    pub fn arg_locs(&self, arg_i: usize) -> (BufferPosition, BufferPosition) {
        let start = self.split_locs[arg_i * 2];
        let stop = self.split_locs[arg_i * 2 + 1];
        (start, stop)
    }

    pub fn num_args(&self) -> usize {
        self.split_locs.len() / 2
    }

    fn out_of_range_or_different(&self, i: usize, target: enums::ArgType) -> bool {
        if i >= self.argument_hints.len() {
            true
        } else {
            self.argument_hints[i].0 != target
        }
    }

    fn push_or_replace(&mut self, i: usize, val: (enums::ArgType, hints::Hint)) {
        if i < self.argument_hints.len() {
            self.argument_hints[i] = val;
        } else {
            self.argument_hints.push(val);
        }
    }

    // TODO: Fix indexing to prevent these checks
    pub fn get_curr_hint_safe(&self) -> Option<(String, &hints::Hint)> {
        if self.num_args() != 0 {
            let mut curr_arg = self.get_curr_arg();
            if curr_arg == self.num_args() {
                curr_arg -= 1;
            }
            let arg = if curr_arg < self.num_args() {
                self.get_buffer_str(self.arg_locs(curr_arg))
            } else {
                String::new()
            };
            let (_arg_type, hint) = &self.get_argument_hints()[curr_arg];
            Some((arg, hint))
        } else {
            None
        }
    }

    // TODO: Fix the `Hint`ing system... These return types are just stupid
    fn arg_to_path(
        &self,
        s: &str,
    ) -> Option<(path::PathBuf, Disregard, String)> {
        let fp = path::PathBuf::from(s);

        let last = if !s.is_empty() {
            fp.iter().last().unwrap().len()
        } else {
            0
        };
        let disregard = s.len() - last;

        let fp = match fp.is_relative() {
            true => self.program_state.borrow().current_working_directory.join(fp),
            false => fp,
        };

        let mut cleaned_path = path::PathBuf::new();
        for dir in fp.iter() {
            if dir == ".." {
                let _ = cleaned_path.pop();
            } else {
                cleaned_path.push(dir);
            }
        }

        if cleaned_path.is_dir() {
            return Some((cleaned_path, disregard, s[disregard..].to_string()));
        }
        if let Some(p) = cleaned_path.parent() {
            if p.is_dir() {
                return Some((cleaned_path.parent().unwrap().to_path_buf(), disregard, s[disregard..].to_string()));
            }
        }
        None
    }

    fn process_hint<T>(
        &mut self,
        ind: usize,
        arg_type_func: impl Fn(&T) -> enums::ArgType,
        inlay_func: impl Fn(&T) -> &str,
        argument: &T,
    ) {
        let arg_type = arg_type_func(argument);
        let arg = self.get_buffer_str(self.arg_locs(ind));
        if self.out_of_range_or_different(ind, arg_type) {
            let hint = match arg_type {
                enums::ArgType::Executable => {
                    hints::executables::make_executables_hint(&arg)
                }
                enums::ArgType::Path => hints::filesystem::make_directory_hints(
                    self.arg_to_path(&arg),
                    Some(inlay_func(argument).to_string()),
                ),
                enums::ArgType::Text => hints::Hint::default(),
            };
            self.push_or_replace(ind, (arg_type, hint));
        } else {
            if arg_type == enums::ArgType::Path {
                hints::filesystem::update_directory_hints(
                    &self.arg_to_path(&arg),
                    &mut self.argument_hints[ind].1,
                );
            } else if arg_type == enums::ArgType::Executable {
                hints::executables::update_executables_hint(
                    &arg,
                    &mut self.argument_hints[ind].1,
                );
            }
        }
    }

    pub fn first_arg(&self) -> Option<String> {
        if self.num_args() > 0 {
            Some(self.get_buffer_str(self.arg_locs(0)))
        } else {
            None
        }
    }

    pub fn update_arguments(&mut self, arg_parser: &parser::ArgumentParser) {
        let args = self.arg_locs_iterator()
            .map(|range| self.get_buffer_str(range))
            .collect::<Vec<_>>();

        if !arg_parser.has_command() {
            if self.out_of_range_or_different(0, enums::ArgType::Executable) {
                let hint = hints::executables::make_executables_hint(arg_parser.first_arg());
                self.push_or_replace(0, (enums::ArgType::Executable, hint));
            } else {
                hints::executables::update_executables_hint(
                    arg_parser.first_arg(),
                    &mut self.argument_hints[0].1,
                );
            }

            for arg_i in 1..self.num_args() {
                let arg = self.get_buffer_str(self.arg_locs(arg_i));
                let path = self.arg_to_path(&arg);
                if self.out_of_range_or_different(arg_i, enums::ArgType::Path) {
                    let hint = hints::filesystem::make_directory_hints(path, None);
                    self.push_or_replace(arg_i, (enums::ArgType::Path, hint));
                    continue;
                }
                hints::filesystem::update_directory_hints(&path, &mut self.argument_hints[arg_i].1);
            }
            return;
        }

        let mut iter = parser::ArgumentIterator::new(&arg_parser);
        iter.reinit(args);
        let mut i = 0;
        for arg in iter {
            match arg {
                parser::Argument::Other => {
                    if self.out_of_range_or_different(i, enums::ArgType::Text) {
                        let hint = hints::Hint::default();
                        self.push_or_replace(i, (enums::ArgType::Text, hint));
                    }
                }
                parser::Argument::ArgFlag(arg_flag) => {
                    if self.out_of_range_or_different(i, enums::ArgType::Text) {
                        let hint = hints::Hint::default();
                        self.push_or_replace(i, (enums::ArgType::Text, hint));
                    }
                    i += 1;
                    self.process_hint(
                        i,
                        command::FlagArgPair::arg_type,
                        command::FlagArgPair::arg_hint,
                        unsafe { &*arg_flag },
                    )
                }
                parser::Argument::Flag(_flag) => {
                    if self.out_of_range_or_different(i, enums::ArgType::Text) {
                        self.push_or_replace(i, (enums::ArgType::Text, hints::Hint::default()));
                    }
                }
                parser::Argument::Arg(arg) => {
                    self.process_hint(
                        i,
                        command::SingleArg::arg_type,
                        command::SingleArg::arg_hint,
                        unsafe { &*arg },
                    )
                }
            }
            i += 1;
        }
    }

    pub fn update(&mut self) {
        self.split_locs.clear();
        self.quote_locs.clear();

        let mut in_quote = false;
        let mut last_split = 0;
        self.curr_arg = 0;
        for (i, c) in self.buffer[..self.input_length].iter().enumerate() {
            if *c == '"' {
                in_quote = !in_quote;
                self.quote_locs.push(i);
            } else if *c == ' ' && !in_quote {
                if self.main_cursor.position > last_split {
                    self.curr_arg += 1;
                }
                self.split_locs.push(last_split);
                self.split_locs.push(i);
                last_split = i + 1;
            }
        }
        if last_split <= self.input_length {
            self.split_locs.push(last_split);
        }
        if self.split_locs.len() % 2 == 1 {
            self.split_locs.push(self.input_length);
        }
    }

    /// Returns index of closest split_loc
    pub fn closest_split(&self, pos: BufferPosition) -> (usize, Side) {
        let mut pos_gt = self.split_locs.len();
        for (i, split) in self.split_locs.iter().enumerate() {
            if *split == pos {
                return (i, Side::Neither);
            }
            if *split > pos {
                pos_gt = i;
                break;
            }
        }

        let prev_dist = pos - self.split_locs[pos_gt - 1];

        let curr_dist = if pos_gt == self.split_locs.len() {
            self.input_length - pos
        } else {
            self.split_locs[pos_gt] - pos
        } as BufferPosition;

        if curr_dist == 0 {
            return (pos_gt, Side::Neither);
        }

        match curr_dist < prev_dist {
            true => (pos_gt, Side::Right),
            false => (pos_gt - 1, Side::Left),
        }
    }

    pub fn jump(&self, side: Side, cursor: &Cursor) -> BufferPosition {
        if !cursor.active {
            panic!("Cursor is not active");
        }
        let (arg_i, split_side) = self.closest_split(cursor.position);
        if split_side == side {
            return self.split_locs[arg_i];
        }

        match side {
            Side::Left => self.split_locs[(arg_i as i64 - 1).max(0) as usize],
            Side::Right => {
                if arg_i >= self.split_locs.len() - 1 {
                    self.input_length
                } else {
                    self.split_locs[arg_i + 1]
                }
            }
            _ => panic!("Side is neither"),
        }
    }

    pub fn main_cur(&self) -> &Cursor {
        &self.main_cursor
    }

    pub fn sec_cur(&self) -> &Cursor {
        &self.secondary_cursor
    }

    pub fn main_cur_set(&mut self, p: BufferPosition) {
        self.main_cursor.position = p;
    }

    pub fn sec_cur_set(&mut self, p: BufferPosition, active: bool) {
        self.secondary_cursor.position = p;
        self.secondary_cursor.active = active;
    }

    pub fn enable_sec_cur_if_not_active(&mut self) {
        if !self.secondary_cursor.active {
            self.secondary_cursor.active = true;
            self.secondary_cursor.position = self.main_cursor.position;
        }
    }

    pub fn del_jump(&mut self, side: Side) {
        if !self.secondary_cursor.active || self.secondary_cursor == self.main_cursor {
            self.secondary_cursor.active = true;
            self.secondary_cursor.position = self.main_cursor.position;
            let new_pos = self.jump(side, &self.secondary_cursor);
            self.secondary_cursor.position = new_pos;
        }
        self.del_betw_curs();
    }

    pub fn del_n(&mut self, side: Side, n: BufferPosition) {
        if !self.secondary_cursor.active || (self.secondary_cursor.active && self.secondary_cursor == self.main_cursor) {
            self.secondary_cursor.active = true;
            self.secondary_cursor.position = self.main_cursor.position;
            let new_pos = self.move_n(side, n, &self.secondary_cursor);
            self.secondary_cursor.position = new_pos;
        }
        self.del_betw_curs();
    }

    pub fn move_n(&self, side: Side, n: BufferPosition, cursor: &Cursor) -> BufferPosition {
        if !cursor.active {
            panic!("Cursor is not active");
        }

        match side {
            Side::Left => (cursor.position as i64 - n as i64).max(0i64) as BufferPosition,
            Side::Right => (cursor.position + n).min(self.input_length),
            _ => panic!("Side is neither"),
        }
    }

    pub fn insert_char_main_cursor(&mut self, c: char) {
        if self.input_length == BUFFER_LENGTH {
            return; // Buffer is full
        }
        // Move all chars after cursor to the right
        for i in (self.main_cursor.position..self.input_length).rev() {
            self.buffer[i + 1] = self.buffer[i];
        }
        self.buffer[self.main_cursor.position] = c;
        self.main_cursor.position += 1;
        self.input_length += 1;
    }

    pub fn insert_str_main_cursor(&mut self, s: &str) {
        if self.input_length + s.len() > BUFFER_LENGTH {
            return; // Buffer is full
        }

        // Move all chars after cursor to the right
        for i in (self.main_cursor.position..self.input_length).rev() {
            self.buffer[i + s.len()] = self.buffer[i];
        }
        for (i, c) in s.chars().enumerate() {
            self.buffer[self.main_cursor.position + i] = c;
        }
        self.main_cursor.position += s.len();
        self.input_length += s.len();
    }

    pub fn del_betw_curs(&mut self) {
        let (start, stop) = self.cursor_range();
        for i in stop..self.input_length {
            self.buffer[start + i - stop] = self.buffer[i];
        }
        self.input_length -= stop - start;
        self.main_cursor.position = start;
        self.secondary_cursor.active = false;
    }

    pub fn unset_secondary_cursor(&mut self) {
        self.secondary_cursor.active = false;
    }

    pub fn cursor_range(&self) -> (BufferPosition, BufferPosition) {
        if self.secondary_cursor.active {
            if self.secondary_cursor.position < self.main_cursor.position {
                (self.secondary_cursor.position, self.main_cursor.position)
            } else {
                (self.main_cursor.position, self.secondary_cursor.position)
            }
        } else {
            (self.main_cursor.position, self.main_cursor.position)
        }
    }

    pub fn clear_all(&mut self) {
        self.input_length = 0;
        self.quote_locs.clear();
        self.split_locs.clear();
        self.argument_hints.clear();
        self.main_cursor.position = 0;
        self.secondary_cursor.active = false;
    }

    pub fn history_older(&mut self) {
        if let Some(older) = self
            .history
            .get_older_history(&self.buffer[..self.input_length])
        {
            let older = older.command();
            for (i, c) in older.chars().enumerate() {
                self.buffer[i] = c;
            }
            self.input_length = older.len();
            self.main_cursor.position = older.len();
            self.secondary_cursor.active = false;
        }
    }

    pub fn history_newer(&mut self) {
        if let Some(newer) = self.history.get_newer_history() {
            let newer = newer.command();
            for (i, c) in newer.chars().enumerate() {
                self.buffer[i] = c;
            }
            self.input_length = newer.len();
            self.main_cursor.position = newer.len();
            self.secondary_cursor.active = false;
        }
    }

    pub fn history_push_current(&mut self) {
        if self.len() == 0 { return; }
        let cmd = self.get_buffer().iter().collect::<String>();
        self.history.add_to_history(cmd).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::{config, parser, state};

    fn default_program_state() -> state::ProgramState {
        state::ProgramState::init(config::FullConfig::default(), std::path::PathBuf::new(), enums::Shell::default())
    }

    #[test]
    fn test_buffer_insert_char() {
        let program_state = Rc::new(RefCell::new(default_program_state()));
        let mut buffer = super::InputBuffer::init(program_state);
        assert_eq!(buffer.len(), 0);

        buffer.insert_char_main_cursor('a');
        assert_eq!(buffer.len(), 1);
        buffer.insert_char_main_cursor('b');
        buffer.insert_char_main_cursor('c');

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_buffer(), &['a', 'b', 'c']);

        buffer.main_cursor.position = 0;
        buffer.insert_char_main_cursor('d');

        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.get_buffer(), &['d', 'a', 'b', 'c']);
    }

    #[test]
    fn test_buffer_insert_str() {
        let program_state = Rc::new(RefCell::new(default_program_state()));
        let mut buffer = super::InputBuffer::init(program_state);
        assert_eq!(buffer.len(), 0);

        buffer.insert_str_main_cursor("abc");

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_buffer(), &['a', 'b', 'c']);

        buffer.main_cursor.position = 0;
        buffer.insert_str_main_cursor("def");

        assert_eq!(buffer.len(), 6);
        assert_eq!(buffer.get_buffer(), &['d', 'e', 'f', 'a', 'b', 'c']);
    }

    #[test]
    fn test_buffer_delete_between_cursors() {
        let program_state = Rc::new(RefCell::new(default_program_state()));
        let mut buffer = super::InputBuffer::init(program_state);
        buffer.insert_str_main_cursor("abcdef");
        buffer.main_cursor.position = 0;
        buffer.secondary_cursor.position = 3;
        buffer.secondary_cursor.active = true;

        buffer.del_betw_curs();

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_buffer(), &['d', 'e', 'f']);
    }

    #[test]
    fn test_buffer_update() {
        let program_state = Rc::new(RefCell::new(default_program_state()));
        let mut buffer = super::InputBuffer::init(program_state);
        buffer.insert_str_main_cursor("abc def ghi");
        buffer.update();

        assert_eq!(buffer.quote_locs.len(), 0);
        assert_eq!(buffer.num_args(), 3);
        assert_eq!(buffer.arg_locs(0), (0, 3));
        assert_eq!(buffer.arg_locs(1), (4, 7));
        assert_eq!(buffer.arg_locs(2), (8, 11));

        buffer.insert_str_main_cursor(" \"jkl mno\" pqr");
        buffer.update();

        assert_eq!(buffer.quote_locs.len(), 2);
        assert_eq!(buffer.num_args(), 5);
        assert_eq!(buffer.arg_locs(0), (0, 3));
        assert_eq!(buffer.arg_locs(1), (4, 7));
        assert_eq!(buffer.arg_locs(2), (8, 11));
        assert_eq!(buffer.arg_locs(3), (12, 21));
        assert_eq!(buffer.arg_locs(4), (22, 25));
    }

    #[test]
    fn test_buffer_closest_split() {
        let program_state = Rc::new(RefCell::new(default_program_state()));
        let mut buffer = super::InputBuffer::init(program_state);
        buffer.insert_str_main_cursor("abc defg hi jklm");
        buffer.update();

        assert_eq!(buffer.closest_split(0), (0, super::Side::Neither));
        assert_eq!(buffer.closest_split(1), (0, super::Side::Left));
        assert_eq!(buffer.closest_split(2), (1, super::Side::Right));
        assert_eq!(buffer.closest_split(3), (1, super::Side::Neither));

        assert_eq!(buffer.closest_split(4), (2, super::Side::Neither));
        assert_eq!(buffer.closest_split(5), (2, super::Side::Left));
        assert_eq!(buffer.closest_split(6), (2, super::Side::Left));
        assert_eq!(buffer.closest_split(7), (3, super::Side::Right));
        assert_eq!(buffer.closest_split(8), (3, super::Side::Neither));

        assert_eq!(buffer.closest_split(9), (4, super::Side::Neither));
        assert_eq!(buffer.closest_split(10), (4, super::Side::Left));
        assert_eq!(buffer.closest_split(11), (5, super::Side::Neither));

        assert_eq!(buffer.closest_split(12), (6, super::Side::Neither));
        assert_eq!(buffer.closest_split(13), (6, super::Side::Left));
        assert_eq!(buffer.closest_split(14), (6, super::Side::Left));
        assert_eq!(buffer.closest_split(15), (7, super::Side::Right));
        assert_eq!(buffer.closest_split(16), (7, super::Side::Neither));
    }

    #[test]
    fn test_buffer_jump() {
        let program_state = Rc::new(RefCell::new(default_program_state()));
        let mut buffer = super::InputBuffer::init(program_state);
        buffer.insert_str_main_cursor("abc defg hi jklm");
        buffer.update();

        buffer.main_cursor.position = 0;
        assert_eq!(buffer.jump(super::Side::Left, &buffer.main_cursor), 0);
        assert_eq!(buffer.jump(super::Side::Right, &buffer.main_cursor), 3);

        buffer.main_cursor.position = 5;
        assert_eq!(buffer.jump(super::Side::Left, &buffer.main_cursor), 4);
        assert_eq!(buffer.jump(super::Side::Right, &buffer.main_cursor), 8);

        buffer.main_cursor.position = 7;
        assert_eq!(buffer.jump(super::Side::Left, &buffer.main_cursor), 4);
        assert_eq!(buffer.jump(super::Side::Right, &buffer.main_cursor), 8);

        buffer.main_cursor.position = 8;
        assert_eq!(buffer.jump(super::Side::Left, &buffer.main_cursor), 4);
        assert_eq!(buffer.jump(super::Side::Right, &buffer.main_cursor), 9);

        buffer.main_cursor.position = 12;
        assert_eq!(buffer.jump(super::Side::Left, &buffer.main_cursor), 11);
        assert_eq!(buffer.jump(super::Side::Right, &buffer.main_cursor), 16);
    }

    use crate::{enums, config::command};

    #[test]
    fn test_argument_types() {
        let program_state = Rc::new(RefCell::new(default_program_state()));
        let mut arg_parser = parser::ArgumentParser::new(program_state.clone());
        let mv_cmd = command::ConfigCommand {
            exe_name: "mv".to_string(),
            exe_to: "move".to_string(),
            execute_before: None,
            execute_after: None,
            args: vec![
                command::SingleArg {
                    arg_type: enums::ArgType::Executable,
                    arg_hint: "exe".to_string(),
                    arg_pos: 1,
                },
                command::SingleArg {
                    arg_type: enums::ArgType::Path,
                    arg_hint: "src".to_string(),
                    arg_pos: 1,
                },
                command::SingleArg {
                    arg_type: enums::ArgType::Path,
                    arg_hint: "dst".to_string(),
                    arg_pos: 2,
                },
            ],
            flags: vec![
                command::Flag {
                    flag_name: "-f".to_string(),
                    flag_to: "--force".to_string(),

                    execute_before: None,
                    execute_after: None,
                }
            ],
            arg_flags: vec![
                command::FlagArgPair {
                    flag_name: "-h".to_string(),
                    flag_to: "--help".to_string(),

                    arg_type: enums::ArgType::Executable,
                    arg_hint: "subcommand".to_string(),

                    execute_before: None,
                    execute_after: None,
                }
            ],
        };
        program_state.borrow_mut().config.commands = vec![mv_cmd];
        let mut buffer = super::InputBuffer::init(program_state);
        buffer.insert_str_main_cursor("mv somewhere tohere");
        buffer.update();
        arg_parser.reinit(buffer.first_arg());
        buffer.update_arguments(&arg_parser);

        assert_eq!(buffer.argument_hints[0].0, enums::ArgType::Executable);
        assert_eq!(buffer.argument_hints[1].0, enums::ArgType::Path);
        assert_eq!(buffer.argument_hints[2].0, enums::ArgType::Path);

        buffer.clear_all();
        buffer.insert_str_main_cursor("mv -f somewhere -h aahhhhh tohere");
        buffer.update();
        arg_parser.reinit(buffer.first_arg());
        buffer.update_arguments(&arg_parser);

        assert_eq!(buffer.argument_hints[0].0, enums::ArgType::Executable);
        assert_eq!(buffer.argument_hints[1].0, enums::ArgType::Text);
        assert_eq!(buffer.argument_hints[2].0, enums::ArgType::Path);
        assert_eq!(buffer.argument_hints[3].0, enums::ArgType::Text);
        assert_eq!(buffer.argument_hints[4].0, enums::ArgType::Executable);
        assert_eq!(buffer.argument_hints[5].0, enums::ArgType::Path);
    }
}

