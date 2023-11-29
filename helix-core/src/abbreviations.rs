use std::{collections::HashMap, path::PathBuf};

use ropey::Rope;

use crate::{movement, Change, Range, Selection, Tendril, Transaction};
use serde::{Deserialize, Serialize};

/// The type that represents the collection of abbreviations,
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Abbreviations(HashMap<String, String>);

impl Abbreviations {
    pub fn default() -> Self {
        Self(HashMap::new())
    }

    /// Look up the word under the main cursor and trigger abbreviation for all selections if there is a match.
    pub fn expand_or_insert(
        &self,
        doc: &Rope,
        selection: &Selection,
        c: char,
    ) -> Option<Transaction> {
        // Default function to insert the original char when we should not expand an abbreviation
        fn insert(c: char, cursor: usize) -> Change {
            let mut t = Tendril::new();
            t.push(c);
            (cursor, cursor, Some(t))
        }

        let transaction = Transaction::change_by_selection(doc, selection, |range| {
            let cursor = range.cursor(doc.slice(..));

            // Do not look for previous word at start of file
            if cursor == 0 {
                return insert(c, cursor);
            }

            // Move 1 char left to be right on the previous word
            let mut current_word_range = Range {
                anchor: cursor - 1,
                head: cursor - 1,
                old_visual_position: None,
            };
            current_word_range =
                movement::move_prev_word_start(doc.slice(..), current_word_range, 1);

            // Early return. Abbreviation should have at least 2 characters
            if current_word_range.len() < 1 {
                return insert(c, cursor);
            }

            // Get current word and check if we know it as an abbreviation
            let current_word = doc.slice(current_word_range.head..current_word_range.anchor);
            let whole_word = self.0.get(&current_word.to_string());

            // Expand abbreviation if needed, insert the original char otherwise
            match whole_word {
                Some(w) => {
                    let mut t = Tendril::new();
                    t.push_str(w);
                    t.push(c);
                    (current_word_range.cursor(doc.slice(..)), cursor, Some(t))
                }
                None => insert(c, cursor),
            }
        });
        Some(transaction)
    }

    pub fn insert(&mut self, abbr: &str, whole_word: &str) {
        self.0.insert(abbr.to_string(), whole_word.to_string());
    }

    pub fn map(&self) -> &HashMap<String, String> {
        &self.0
    }

    pub fn map_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }

    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
    }
}

impl From<&PathBuf> for Abbreviations {
    fn from(value: &PathBuf) -> Self {
        let mut abbr = Self::default();

        if let Ok(abbr_file_content) = std::fs::read_to_string(value) {
            // Each line should insert an abbr
            for line in abbr_file_content.lines() {
                if let Some(split) = line.split_once(' ') {
                    abbr.insert(split.0, split.1);
                }
            }
        }
        abbr
    }
}
