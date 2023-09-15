use crate::utils;
use std::path;

pub mod executables;
pub mod filesystem;

#[derive(Debug)]
pub struct Hint<'a> {
    /// Assumes selection is alphabetically sorted!
    selection: Vec<String>,
    inlay: Option<&'a str>,
    set_using: path::PathBuf,
}

impl<'a> Hint<'a> {
    pub fn new(selection: Vec<String>, inlay: Option<&'a str>) -> Self {
        Self {
            selection,
            inlay,
            set_using: path::PathBuf::new(),
        }
    }

    pub fn set_selection(&mut self, selection: Vec<String>) {
        self.selection = selection;
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

    pub fn closest_match(&'a self, s: &str) -> Option<&'a str> {
        if s.is_empty() && self.inlay.is_some() {
            Some(self.inlay.unwrap())
        } else if s.is_empty() {
            self.selection.first().map(|x| x.as_str())
        } else {
            match utils::first_item(&self.selection, s) {
                Some(ind) => Some(&self.selection[ind]),
                None => None,
            }
        }
    }

    pub fn filtered_items(&self, s: &'a str) -> impl Iterator<Item = &str> {
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
        }
    }
}
