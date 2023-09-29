//
// fn out_of_range_or_different(&self, i: usize, target: enums::ArgType) -> bool {
//     if i >= self.argument_hints.len() {
//         true
//     } else {
//         self.argument_hints[i].0 != target
//     }
// }
//
// fn push_or_replace(&mut self, i: usize, val: (enums::ArgType, hints::Hint<'a>)) {
//     if i < self.argument_hints.len() {
//         self.argument_hints[i] = val;
//     } else {
//         self.argument_hints.push(val);
//     }
// }
//
// // TODO: Clean all of this rubbish up
// fn arg_to_path(&self, s: &str) -> Option<(path::PathBuf, Disregard, String)> {
//     let fp = path::PathBuf::from(s);
//
//     let last = if !s.is_empty() {
//         fp.iter().last().unwrap().len()
//     } else {
//         0
//     };
//     let disregard = s.len() - last;
//
//     let fp = match fp.is_relative() {
//         true => self.program_state.current_working_directory.join(fp),
//         false => fp,
//     };
//
//     let mut cleaned_path = path::PathBuf::new();
//     for dir in fp.iter() {
//         if dir == ".." {
//             let _ = cleaned_path.pop();
//         } else {
//             cleaned_path.push(dir);
//         }
//     }
//
//     if cleaned_path.is_dir() {
//         return Some((cleaned_path, disregard, s[disregard..].to_string()));
//     }
//     if let Some(p) = cleaned_path.parent() {
//         if p.is_dir() {
//             return Some((cleaned_path.parent().unwrap().to_path_buf(), disregard, s[disregard..].to_string()));
//         }
//     }
//     None
// }
//
// fn process_arg_flags(
//     &mut self,
//     i: usize,
//     arg: &str,
//     cmd: &'a command::ConfigCommand,
//     arg_flag_skips: &mut Vec<usize>,
// ) -> Skip {
//     let mut skip = Skip::None;
//     for (k, arg_flag) in cmd.arg_flags.iter().enumerate() {
//         if arg_flag_skips.contains(&k) {
//             continue;
//         }
//         if arg_flag.flag_name == arg {
//             if self.out_of_range_or_different(i, enums::ArgType::Text) {
//                 let hint = hints::Hint::default();
//                 self.push_or_replace(i, (enums::ArgType::Text, hint));
//             }
//             if self.out_of_range_or_different(i + 1, arg_flag.arg_type.clone()) {
//                 let hint = match arg_flag.arg_type {
//                     enums::ArgType::Executable => {
//                         hints::executables::make_executables_hint(arg)
//                     }
//                     enums::ArgType::Path => hints::filesystem::make_directory_hints(
//                         self.arg_to_path(&arg),
//                         Some(&arg_flag.arg_hint),
//                     ),
//                     enums::ArgType::Text => hints::Hint::default(),
//                 };
//                 self.push_or_replace(i + 1, (arg_flag.arg_type.clone(), hint));
//             } else if arg_flag.arg_type == enums::ArgType::Path {
//                 hints::filesystem::update_directory_hints(
//                     &self.arg_to_path(&arg),
//                     &mut self.argument_hints[i + 1].1,
//                 );
//             } else if arg_flag.arg_type == enums::ArgType::Executable {
//                 hints::executables::update_executables_hint(
//                     arg,
//                     &mut self.argument_hints[i + 1].1,
//                 );
//             }
//             skip = Skip::Twice;
//             arg_flag_skips.push(k);
//             break;
//         }
//     }
//     skip
// }
//
// fn process_flags(
//     &mut self,
//     i: usize,
//     arg: &str,
//     cmd: &'a command::ConfigCommand,
//     flag_skips: &mut Vec<usize>,
// ) -> Skip {
//     for (k, flag) in cmd.flags.iter().enumerate() {
//         if flag_skips.contains(&k) {
//             continue;
//         }
//         if flag.flag_name == arg {
//             if self.out_of_range_or_different(i, enums::ArgType::Text) {
//                 self.push_or_replace(i, (enums::ArgType::Text, hints::Hint::default()));
//             }
//             flag_skips.push(k);
//             return Skip::Once;
//         }
//     }
//     Skip::None
// }
//
// fn process_args(
//     &mut self,
//     i: usize,
//     arg: &str,
//     cmd: &'a command::ConfigCommand,
//     arg_c: &mut usize,
//     arg_skips: &mut Vec<usize>,
// ) -> Skip {
//     for (k, single_arg) in cmd.args.iter().enumerate() {
//         if arg_skips.contains(&k) {
//             continue;
//         }
//         if single_arg.arg_pos == *arg_c {
//             if self.out_of_range_or_different(i, single_arg.arg_type.clone()) {
//                 let hint = match single_arg.arg_type {
//                     enums::ArgType::Executable => {
//                         hints::executables::make_executables_hint(arg)
//                     }
//                     enums::ArgType::Path => hints::filesystem::make_directory_hints(
//                         self.arg_to_path(&arg),
//                         Some(&single_arg.arg_hint),
//                     ),
//                     enums::ArgType::Text => hints::Hint::default(),
//                 };
//                 self.push_or_replace(i, (single_arg.arg_type.clone(), hint));
//             } else {
//                 if single_arg.arg_type == enums::ArgType::Path {
//                     hints::filesystem::update_directory_hints(
//                         &self.arg_to_path(&arg),
//                         &mut self.argument_hints[i].1,
//                     );
//                 } else if single_arg.arg_type == enums::ArgType::Executable {
//                     hints::executables::update_executables_hint(
//                         arg,
//                         &mut self.argument_hints[i].1,
//                     );
//                 }
//             }
//             *arg_c += 1;
//             arg_skips.push(k);
//             return Skip::Once;
//         }
//     }
//     Skip::None
// }
//
// fn update_arguments(&mut self) {
//     let first_arg = {
//         if self.num_args() == 0 {
//             self.current_command = None;
//             return;
//         }
//         self.get_buffer_str(self.arg_locs(0))
//     };
//
//     if self.out_of_range_or_different(0, enums::ArgType::Executable) {
//         let hint = hints::executables::make_executables_hint(&first_arg);
//         self.push_or_replace(0, (enums::ArgType::Executable, hint));
//     } else {
//         hints::executables::update_executables_hint(
//             &first_arg,
//             &mut self.argument_hints[0].1,
//         );
//     }
//
//     if !first_arg.is_empty() {
//         if self.current_command.is_some() {
//             if first_arg != self.current_command.unwrap().exe_name {
//                 self.current_command = None;
//             }
//         }
//         if self.current_command.is_none() {
//             for cmd in &self.program_state.config.commands {
//                 if cmd.exe_name == first_arg {
//                     self.current_command = Some(&cmd);
//                 }
//             }
//         }
//     }
//
//     if self.current_command.is_none() {
//         for arg_i in 1..self.num_args() {
//             let arg = self.get_buffer_str(self.arg_locs(arg_i));
//             let path = self.arg_to_path(&arg);
//             if self.out_of_range_or_different(arg_i, enums::ArgType::Path) {
//                 let hint = hints::filesystem::make_directory_hints(path, None);
//                 self.push_or_replace(arg_i, (enums::ArgType::Path, hint));
//                 continue;
//             }
//             hints::filesystem::update_directory_hints(&path, &mut self.argument_hints[arg_i].1);
//         }
//         return;
//     }
//
//     let cmd = self.current_command.unwrap();
//
//     let mut flag_skips = Vec::with_capacity(cmd.flags.len());
//     let mut arg_skips = Vec::with_capacity(cmd.args.len());
//     let mut arg_flag_skips = Vec::with_capacity(cmd.arg_flags.len());
//
//     let mut skip = Skip::None;
//     let mut arg_c = 1;
//
//     let iter = self
//         .arg_locs_iterator()
//         .map(|range| self.get_buffer_str(range))
//         .collect::<Vec<_>>();
//
//
//     'outer: for (i, arg) in iter.iter().enumerate().skip(1)
//     {
//         if skip == Skip::Once {
//             skip = Skip::None;
//             continue;
//         }
//         // TODO: Use binary searches instead
//         skip = Self::process_arg_flags(self, i, &arg, cmd, &mut arg_flag_skips);
//
//         if skip == Skip::Once {
//             skip = Skip::None;
//             continue 'outer;
//         } else if skip == Skip::Twice {
//             skip = Skip::Once;
//             continue 'outer;
//         }
//
//         skip = Self::process_flags(self, i, &arg, cmd, &mut flag_skips);
//
//         if skip == Skip::Once {
//             skip = Skip::None;
//             continue 'outer;
//         } else if skip == Skip::Twice {
//             skip = Skip::Once;
//             continue 'outer;
//         }
//
//         skip = Self::process_args(self, i, &arg, cmd, &mut arg_c, &mut arg_skips);
//
//         if skip == Skip::Once {
//             skip = Skip::None;
//             continue 'outer;
//         } else if skip == Skip::Twice {
//             skip = Skip::Once;
//             continue 'outer;
//         }
//
//         if self.out_of_range_or_different(i, enums::ArgType::Text) {
//             self.push_or_replace(i, (enums::ArgType::Text, hints::Hint::default()));
//         }
//     }
// }
