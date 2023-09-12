use crate::{config, config::command, enums, hints, state};
use std::path;

const BUFFER_LENGTH: usize = 8192;

pub type BufferPosition = usize;

#[derive(Debug, PartialEq)]
enum Skip {
    None,
    Once,
    Twice,
}

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

#[derive(Debug)]
pub struct Cursor {
    position: BufferPosition,
    active: bool,
}

impl Cursor {
    pub fn new(position: usize, active: bool) -> Self {
        Self { position, active }
    }
}

#[derive(Debug)]
pub struct InputBuffer<'a> {
    buffer: [char; BUFFER_LENGTH],
    input_length: usize,

    main_cursor: Cursor,
    secondary_cursor: Cursor,

    /// 1D array of start and stop of arguments.
    /// For all args, argstart = index * 2, argstop = index * 2 + 1
    split_locs: Vec<BufferPosition>,

    quote_locs: Vec<BufferPosition>,

    program_state: &'a state::ProgramState,
    current_command: Option<&'a command::ConfigCommand>,
    argument_hints: Vec<(enums::ArgType, hints::Hint<'a>)>,
}

impl<'a> InputBuffer<'a> {
    pub fn init(program_state: &'a state::ProgramState) -> Self {
        Self {
            buffer: ['\0'; BUFFER_LENGTH],
            input_length: 0,
            main_cursor: Cursor::new(0, true),
            secondary_cursor: Cursor::new(0, false),
            split_locs: Vec::new(),
            quote_locs: Vec::new(),
            argument_hints: Vec::new(),
            current_command: None,
            program_state,
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

    pub fn get_splits(&self) -> &[BufferPosition] {
        &self.split_locs
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
            self.argument_hints[i].0 == target
        }
    }

    fn push_or_replace(&mut self, i: usize, val: (enums::ArgType, hints::Hint<'a>)) {
        if i < self.argument_hints.len() {
            self.argument_hints[i] = val;
        } else {
            self.argument_hints.push(val);
        }
    }

    fn arg_to_path(s: &str) -> path::PathBuf {
        todo!()
    }

    fn process_arg_flags(
        &mut self,
        i: usize,
        arg: &str,
        cmd: &'a command::ConfigCommand,
        arg_flag_skips: &mut Vec<usize>,
    ) -> Skip {
        let mut skip = Skip::None;
        for (k, arg_flag) in cmd.arg_flags.iter().enumerate() {
            if arg_flag_skips.contains(&k) {
                continue;
            }
            if arg_flag.flag_name == arg {
                if self.out_of_range_or_different(i, enums::ArgType::Text) {
                    let hint = hints::Hint::default();
                    self.push_or_replace(i, (enums::ArgType::Text, hint));
                }
                if self.out_of_range_or_different(i + 1, arg_flag.arg_type.clone()) {
                    let hint = match arg_flag.arg_type {
                        enums::ArgType::Executable => {
                            hints::executables::make_executables_hint()
                        }
                        enums::ArgType::Path => hints::filesystem::make_directory_hints(
                            Self::arg_to_path(&arg),
                            Some(&arg_flag.arg_hint),
                        ),
                        enums::ArgType::Text => hints::Hint::default(),
                    };
                    self.push_or_replace(i + 1, (arg_flag.arg_type.clone(), hint));
                } else if arg_flag.arg_type == enums::ArgType::Path {
                    hints::filesystem::update_directory_hints(
                        &Self::arg_to_path(&arg),
                        &mut self.argument_hints[i + 1].1,
                    );
                }
                skip = Skip::Twice;
                arg_flag_skips.push(k);
                break;
            }
        }
        skip
    }

    fn process_flags(
        &mut self,
        i: usize,
        arg: &str,
        cmd: &'a command::ConfigCommand,
        flag_skips: &mut Vec<usize>,
    ) -> Skip {
        for (k, flag) in cmd.flags.iter().enumerate() {
            if flag_skips.contains(&k) {
                continue;
            }
            if flag.flag_name == arg {
                if self.out_of_range_or_different(i, enums::ArgType::Text) {
                    self.push_or_replace(i, (enums::ArgType::Text, hints::Hint::default()));
                }
                flag_skips.push(k);
                return Skip::Once;
            }
        }
        Skip::None
    }

    fn process_args(
        &mut self,
        i: usize,
        arg: &str,
        cmd: &'a command::ConfigCommand,
        arg_c: &mut usize,
        arg_skips: &mut Vec<usize>,
    ) -> Skip {
        for (k, single_arg) in cmd.args.iter().enumerate() {
            if arg_skips.contains(&k) {
                continue;
            }
            if single_arg.arg_pos == *arg_c {
                if self.out_of_range_or_different(i, single_arg.arg_type.clone()) {
                    let hint = match single_arg.arg_type {
                        enums::ArgType::Executable => {
                            hints::executables::make_executables_hint()
                        }
                        enums::ArgType::Path => hints::filesystem::make_directory_hints(
                            Self::arg_to_path(&arg),
                            Some(&single_arg.arg_hint),
                        ),
                        enums::ArgType::Text => hints::Hint::default(),
                    };
                    self.push_or_replace(i, (single_arg.arg_type.clone(), hint));
                }
                *arg_c += 1;
                arg_skips.push(k);
                return Skip::Once;
            }
        }
        Skip::None
    }

    fn update_arguments(&mut self) {
        if self.out_of_range_or_different(0, enums::ArgType::Executable) {
            let hint = hints::executables::make_executables_hint();
            self.push_or_replace(0, (enums::ArgType::Executable, hint));
        }

        let first_arg = {
            if self.num_args() == 0 {
                self.current_command = None;
                return;
            }
            self.get_buffer_str(self.arg_locs(0))
        };

        if !first_arg.is_empty() {
            if self.current_command.is_some() {
                if first_arg != self.current_command.unwrap().exe_name {
                    self.current_command = None;
                }
            }
            if self.current_command.is_none() {
                for cmd in &self.program_state.config.commands {
                    if cmd.exe_name == first_arg {
                        self.current_command = Some(&cmd);
                    }
                }
            }
        }

        if self.current_command.is_none() {
            for arg_i in 1..self.num_args() {
                let arg = self.get_buffer_str(self.arg_locs(arg_i));
                let path = Self::arg_to_path(&arg);
                if self.out_of_range_or_different(arg_i, enums::ArgType::Path) {
                    let hint = hints::filesystem::make_directory_hints(path, None);
                    self.push_or_replace(arg_i, (enums::ArgType::Path, hint));
                    continue;
                }
                hints::filesystem::update_directory_hints(&path, &mut self.argument_hints[arg_i].1);
            }
            return;
        }

        let cmd = self.current_command.unwrap();

        let mut flag_skips = Vec::with_capacity(cmd.flags.len());
        let mut arg_skips = Vec::with_capacity(cmd.args.len());
        let mut arg_flag_skips = Vec::with_capacity(cmd.arg_flags.len());

        let mut skip = Skip::None;
        let mut arg_c = 1;

        let iter = self
            .arg_locs_iterator()
            .enumerate()
            .map(|(i, range)| (i, self.get_buffer_str(range)))
            .collect::<Vec<_>>();

        'outer: for (i, arg) in iter
        {
            if skip == Skip::Once {
                skip = Skip::None;
                continue;
            }
            // TODO: Use binary searches instead
            skip = Self::process_arg_flags(self, i, &arg, cmd, &mut arg_flag_skips);

            if skip == Skip::Once {
                skip = Skip::None;
                continue 'outer;
            } else if skip == Skip::Twice {
                skip = Skip::Once;
                continue 'outer;
            }

            skip = Self::process_flags(self, i, &arg, cmd, &mut flag_skips);

            if skip == Skip::Once {
                skip = Skip::None;
                continue 'outer;
            } else if skip == Skip::Twice {
                skip = Skip::Once;
                continue 'outer;
            }

            skip = Self::process_args(self, i, &arg, cmd, &mut arg_c, &mut arg_skips);

            if skip == Skip::Once {
                skip = Skip::None;
                continue 'outer;
            } else if skip == Skip::Twice {
                skip = Skip::Once;
                continue 'outer;
            }
        }
    }

    pub fn update(&mut self) {
        self.split_locs.clear();
        self.quote_locs.clear();

        let mut in_quote = false;
        let mut last_split = 0;
        for (i, c) in self.buffer[..self.input_length].iter().enumerate() {
            if *c == '"' {
                in_quote = !in_quote;
                self.quote_locs.push(i);
            } else if *c == ' ' && !in_quote {
                self.split_locs.push(last_split);
                self.split_locs.push(i);
                last_split = i + 1;
            }
        }
        if last_split < self.input_length {
            self.split_locs.push(last_split);
        }
        if self.split_locs.len() % 2 == 1 {
            self.split_locs.push(self.input_length);
        }
        self.update_arguments();
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
                if arg_i == self.split_locs.len() - 1 {
                    self.input_length
                } else {
                    self.split_locs[arg_i + 1]
                }
            }
            _ => panic!("Side is neither"),
        }
    }

    pub fn move_n(&self, side: Side, n: BufferPosition, cursor: &Cursor) -> BufferPosition {
        if !cursor.active {
            panic!("Cursor is not active");
        }

        match side {
            Side::Left => (cursor.position as i64 - n as i64).max(0) as BufferPosition,
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

    pub fn delete_between_cursors(&mut self) {
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
}

#[cfg(test)]
mod tests {
    use crate::{config, state};

    fn default_program_state() -> state::ProgramState {
        state::ProgramState::init(config::FullConfig::default(), std::path::PathBuf::new())
    }

    #[test]
    fn test_buffer_insert_char() {
        let program_state = default_program_state();
        let mut buffer = super::InputBuffer::init(&program_state);
        assert_eq!(buffer.len(), 0);

        buffer.insert_char_main_cursor('a');
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
        let program_state = default_program_state();
        let mut buffer = super::InputBuffer::init(&program_state);
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
        let program_state = default_program_state();
        let mut buffer = super::InputBuffer::init(&program_state);
        buffer.insert_str_main_cursor("abcdef");
        buffer.main_cursor.position = 0;
        buffer.secondary_cursor.position = 3;
        buffer.secondary_cursor.active = true;

        buffer.delete_between_cursors();

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_buffer(), &['d', 'e', 'f']);
    }

    #[test]
    fn test_buffer_update() {
        let program_state = default_program_state();
        let mut buffer = super::InputBuffer::init(&program_state);
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
        let program_state = default_program_state();
        let mut buffer = super::InputBuffer::init(&program_state);
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
        let program_state = default_program_state();
        let mut buffer = super::InputBuffer::init(&program_state);
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
        let mut program_state = default_program_state();
        let mv_cmd = command::ConfigCommand {
            exe_name: "mv".to_string(),
            exe_to: "move".to_string(),
            execute_before: None,
            execute_after: None,
            args: vec![
                command::SingleArg {
                    arg_type: enums::ArgType::Path,
                    arg_hint: "".to_string(),
                    arg_pos: 1,
                },
                command::SingleArg {
                    arg_type: enums::ArgType::Path,
                    arg_hint: "".to_string(),
                    arg_pos: 2,
                },
            ],
            flags: vec![],
            arg_flags: vec![],
        };
        program_state.config.commands = vec![mv_cmd];
        let mut buffer = super::InputBuffer::init(&program_state);
        buffer.insert_str_main_cursor("mv sdf asda");
        buffer.update();
        assert_eq!(buffer.get_argument_hints().len(), 2);
    }
}

