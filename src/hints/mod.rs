use crate::utils;
use std::path;

pub mod executables;
pub mod filesystem;

pub type Disregard = usize;

#[derive(Debug)]
pub struct Hint<'a> {
    /// Assumes selection is alphabetically sorted!
    selection: Vec<String>,
    inlay: Option<&'a str>,
    set_using: path::PathBuf,
    disregard: Disregard,
    last_closest_match: Option<String>,
}

impl<'a> Hint<'a> {
    pub fn new(selection: Vec<String>, inlay: Option<&'a str>) -> Self {
        Self {
            selection,
            inlay,
            set_using: path::PathBuf::new(),
            disregard: 0,
            last_closest_match: None,
        }
    }

    pub fn set_selection(&mut self, selection: Vec<String>) {
        self.last_closest_match = None;
        self.selection = selection;
    }

    pub fn get_selection(&self) -> &[String] {
        &self.selection
    }

    pub fn set_using(&self) -> &path::PathBuf {
        &self.set_using
    }

    pub fn set_set_using(&mut self, set_using: path::PathBuf) {
        self.set_using = set_using;
    }

    pub fn set_inlay(&mut self, inlay: Option<&'a str>) {
        self.inlay = inlay;
    }

    pub fn set_disregard(&mut self, disregard: Disregard) {
        self.disregard = disregard;
    }

    pub fn disregard(&self) -> Disregard {
        self.disregard
    }

    pub fn last_closest_match(&self) -> Option<&String> {
        self.last_closest_match.as_ref()
    }

    // TODO: Cleanup more rubbish
    pub fn closest_match(&mut self, s: &str) {
        let x = if s.is_empty() && self.inlay.is_some() {
            Some(self.inlay.unwrap().to_string())
        } else if s.is_empty() {
            self.selection.first().map(|x| x.clone())
        } else {
            match utils::first_item(&self.selection, s) {
                Some(ind) => Some(self.selection[ind].clone()),
                None => None,
            }
        };
        self.last_closest_match = x;
    }

    pub fn set_closest_match(&mut self, s: String) {
        self.last_closest_match = Some(s);
    }

    pub fn filtered_items(&self, s: &'a str) -> impl Iterator<Item=&str> {
        self.selection.iter().filter_map(move |v| {
            if v.starts_with(s) {
                Some(v.as_str())
            } else {
                None
            }
        })
    }
}

impl<'a> Default for Hint<'a> {
    fn default() -> Self {
        Self {
            selection: vec![],
            inlay: None,
            set_using: path::PathBuf::new(),
            disregard: 0,
            last_closest_match: None,
        }
    }
}
