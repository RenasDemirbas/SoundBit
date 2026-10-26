//! Undo and redo stacks.

use std::collections::VecDeque;

use crate::edit::Edit;

/// Undo steps kept before the oldest are forgotten.
pub(crate) const HISTORY_LIMIT: usize = 500;

/// One undoable step: applying `edit` reverses it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Entry {
    pub(crate) label: String,
    pub(crate) edit: Edit,
}

/// Undo and redo stacks of reversing edits.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct History {
    undo: VecDeque<Entry>,
    redo: Vec<Entry>,
}

impl History {
    /// Records a new step. Clears the redo stack, as the old future no longer applies.
    pub(crate) fn record(&mut self, entry: Entry) {
        self.redo.clear();
        self.push_undo(entry);
    }

    /// Pushes onto the undo stack without touching redo (used by redo itself).
    pub(crate) fn push_undo(&mut self, entry: Entry) {
        if self.undo.len() == HISTORY_LIMIT {
            self.undo.pop_front();
        }
        self.undo.push_back(entry);
    }

    pub(crate) fn push_redo(&mut self, entry: Entry) {
        self.redo.push(entry);
    }

    pub(crate) fn pop_undo(&mut self) -> Option<Entry> {
        self.undo.pop_back()
    }

    pub(crate) fn pop_redo(&mut self) -> Option<Entry> {
        self.redo.pop()
    }

    pub(crate) fn undo_label(&self) -> Option<&str> {
        self.undo.back().map(|entry| entry.label.as_str())
    }

    pub(crate) fn redo_label(&self) -> Option<&str> {
        self.redo.last().map(|entry| entry.label.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(label: &str) -> Entry {
        Entry {
            label: label.to_owned(),
            edit: Edit::Batch(Vec::new()),
        }
    }

    #[test]
    fn recording_clears_redo() {
        let mut history = History::default();
        history.record(entry("a"));
        history.push_redo(entry("b"));
        history.record(entry("c"));
        assert_eq!(history.redo_label(), None);
        assert_eq!(history.undo_label(), Some("c"));
    }

    #[test]
    fn forgets_oldest_beyond_limit() {
        let mut history = History::default();
        for i in 0..=HISTORY_LIMIT {
            history.record(entry(&i.to_string()));
        }
        let mut count = 0;
        let mut last = None;
        while let Some(entry) = history.pop_undo() {
            count += 1;
            last = Some(entry.label);
        }
        assert_eq!(count, HISTORY_LIMIT);
        assert_eq!(last.as_deref(), Some("1"));
    }
}
