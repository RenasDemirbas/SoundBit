//! The editor: a song, its undo history and the rules for changing it.

use std::sync::atomic::{AtomicU64, Ordering};

use soundbit_core::{ProfileViolation, Song};

use crate::edit::{Applied, Created, Edit};
use crate::history::{Entry, History};
use crate::{Command, CommandError};

/// Source of [`Editor::id`] values.
static NEXT_EDITOR_ID: AtomicU64 = AtomicU64::new(1);

/// Who asked for a change. Agent changes are marked in the undo history, so a human can see
/// and revert what an AI did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Origin {
    /// A person using the GUI or CLI.
    #[default]
    User,
    /// An AI agent connected over MCP.
    Agent,
}

impl Origin {
    fn label(self, label: String) -> String {
        match self {
            Self::User => label,
            Self::Agent => format!("AI: {label}"),
        }
    }
}

/// How chip-profile limits are applied to commands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProfilePolicy {
    /// Reject any command that adds a new violation.
    #[default]
    Enforce,
    /// Accept every command and only report violations in the [`Outcome`].
    Report,
}

/// What a successful command did.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// Label recorded in the undo history.
    pub label: String,
    /// Entities the command created, in creation order.
    pub created: Vec<Created>,
    /// All chip-profile violations of the song after the change.
    pub violations: Vec<ProfileViolation>,
    /// Song revision after the change.
    pub revision: u64,
}

/// Owns a song and changes it only through commands, with undo and redo of the most recent
/// steps.
///
/// Every change runs on a draft copy that replaces the song only on success, so the song is
/// never left half-changed.
#[derive(Debug, Clone, PartialEq)]
pub struct Editor {
    id: u64,
    song: Song,
    history: History,
    policy: ProfilePolicy,
    revision: u64,
}

impl Editor {
    /// Starts editing `song` with an empty history.
    pub fn new(song: Song) -> Self {
        Self {
            id: NEXT_EDITOR_ID.fetch_add(1, Ordering::Relaxed),
            song,
            history: History::default(),
            policy: ProfilePolicy::default(),
            revision: 0,
        }
    }

    /// Identifies this editor within the process. Opening or creating a song makes a new
    /// editor, so `(id, revision)` changes whenever the song changes in any way.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// The current song.
    pub fn song(&self) -> &Song {
        &self.song
    }

    /// Counter that increases with every change, undo and redo. Adapters compare it to detect
    /// changes made by someone else.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The chip-profile policy.
    pub fn policy(&self) -> ProfilePolicy {
        self.policy
    }

    /// Changes the chip-profile policy.
    pub fn set_policy(&mut self, policy: ProfilePolicy) {
        self.policy = policy;
    }

    /// Applies a command as one undo step.
    ///
    /// # Errors
    ///
    /// Returns the command's error, or [`CommandError::ProfileViolations`] if the policy is
    /// [`ProfilePolicy::Enforce`] and the change adds violations. The song is unchanged on
    /// error.
    pub fn execute(
        &mut self,
        command: impl Into<Command>,
        origin: Origin,
    ) -> Result<Outcome, CommandError> {
        let command = command.into();
        let label = origin.label(command.label());
        let before = self.song.profile_violations();

        let mut draft = self.song.clone();
        let Applied { inverse, created } = command.apply(&mut draft)?;
        let violations = draft.profile_violations();
        if self.policy == ProfilePolicy::Enforce {
            let added: Vec<_> = violations
                .iter()
                .filter(|v| !before.contains(v))
                .cloned()
                .collect();
            if !added.is_empty() {
                return Err(CommandError::ProfileViolations { violations: added });
            }
        }

        self.commit(draft);
        self.history.record(Entry {
            label: label.clone(),
            edit: inverse,
        });
        Ok(Outcome {
            label,
            created,
            violations,
            revision: self.revision,
        })
    }

    /// Reverts the latest step and returns its label, or `None` if there is nothing to undo.
    ///
    /// # Errors
    ///
    /// Returns an error if the reversing edit no longer applies; the step is kept.
    pub fn undo(&mut self) -> Result<Option<String>, CommandError> {
        let Some(entry) = self.history.pop_undo() else {
            return Ok(None);
        };
        match self.replay(&entry) {
            Ok(redo) => {
                self.history.push_redo(Entry {
                    label: entry.label.clone(),
                    edit: redo,
                });
                Ok(Some(entry.label))
            }
            Err(error) => {
                self.history.push_undo(entry);
                Err(error)
            }
        }
    }

    /// Re-applies the latest undone step and returns its label, or `None` if there is nothing
    /// to redo.
    ///
    /// # Errors
    ///
    /// Returns an error if the edit no longer applies; the step is kept.
    pub fn redo(&mut self) -> Result<Option<String>, CommandError> {
        let Some(entry) = self.history.pop_redo() else {
            return Ok(None);
        };
        match self.replay(&entry) {
            Ok(undo) => {
                self.history.push_undo(Entry {
                    label: entry.label.clone(),
                    edit: undo,
                });
                Ok(Some(entry.label))
            }
            Err(error) => {
                self.history.push_redo(entry);
                Err(error)
            }
        }
    }

    /// Label of the step [`Editor::undo`] would revert.
    pub fn undo_label(&self) -> Option<&str> {
        self.history.undo_label()
    }

    /// Label of the step [`Editor::redo`] would re-apply.
    pub fn redo_label(&self) -> Option<&str> {
        self.history.redo_label()
    }

    /// Applies a history entry to a draft and commits it, returning the reversing edit.
    fn replay(&mut self, entry: &Entry) -> Result<Edit, CommandError> {
        let mut draft = self.song.clone();
        let applied = entry.edit.clone().apply(&mut draft)?;
        self.commit(draft);
        Ok(applied.inverse)
    }

    fn commit(&mut self, song: Song) {
        self.song = song;
        self.revision += 1;
    }
}
