//! Ordered, id-keyed storage used by the song aggregate.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::id::EntityId;

/// A value taken out of a song together with its id and position, so the removal can be
/// undone exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removed<I, T> {
    /// Id the value had.
    pub id: I,
    /// Position the value had among its siblings.
    pub index: usize,
    /// The removed value.
    pub value: T,
}

/// Entities in insertion order, addressed by ids that are never reused.
#[derive(Debug, Clone)]
pub(crate) struct Collection<I, T> {
    entries: Vec<(I, T)>,
    next_id: u32,
}

/// Equality compares the entries only. The id counter is bookkeeping: it is not saved, and
/// undoing an addition deliberately leaves it advanced so ids are never handed out twice.
impl<I: PartialEq, T: PartialEq> PartialEq for Collection<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<I: Eq, T: Eq> Eq for Collection<I, T> {}

impl<I: EntityId, T> Collection<I, T> {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    /// Builds a collection from stored entries. Fails with the first duplicate id.
    pub(crate) fn from_entries(entries: Vec<(I, T)>) -> Result<Self, I> {
        let mut collection = Self::new();
        for (id, value) in entries {
            collection.restore(Removed {
                id,
                index: usize::MAX,
                value,
            })?;
        }
        Ok(collection)
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (I, &T)> {
        self.entries.iter().map(|(id, value)| (*id, value))
    }

    pub(crate) fn get(&self, id: I) -> Option<&T> {
        self.entries
            .iter()
            .find(|(entry, _)| *entry == id)
            .map(|(_, value)| value)
    }

    pub(crate) fn get_mut(&mut self, id: I) -> Option<&mut T> {
        self.entries
            .iter_mut()
            .find(|(entry, _)| *entry == id)
            .map(|(_, value)| value)
    }

    pub(crate) fn contains(&self, id: I) -> bool {
        self.get(id).is_some()
    }

    /// Appends a value under a fresh id.
    pub(crate) fn push(&mut self, value: T) -> I {
        let id = I::from_raw(self.next_id);
        self.next_id += 1;
        self.entries.push((id, value));
        id
    }

    /// Swaps in a new value and returns the old one, or `None` if the id is unknown.
    pub(crate) fn replace(&mut self, id: I, value: T) -> Option<T> {
        self.get_mut(id).map(|slot| std::mem::replace(slot, value))
    }

    pub(crate) fn remove(&mut self, id: I) -> Option<Removed<I, T>> {
        let index = self.entries.iter().position(|(entry, _)| *entry == id)?;
        let (id, value) = self.entries.remove(index);
        Some(Removed { id, index, value })
    }

    /// Puts a removed value back at its old position (or the end, if the collection shrank).
    /// Fails with the id if it is already taken.
    pub(crate) fn restore(&mut self, removed: Removed<I, T>) -> Result<(), I> {
        if self.contains(removed.id) {
            return Err(removed.id);
        }
        let index = removed.index.min(self.entries.len());
        self.next_id = self.next_id.max(removed.id.raw() + 1);
        self.entries.insert(index, (removed.id, removed.value));
        Ok(())
    }
}

#[derive(Serialize)]
struct EntryRef<'a, I, T> {
    id: I,
    #[serde(flatten)]
    value: &'a T,
}

#[derive(Deserialize)]
struct Entry<I, T> {
    id: I,
    #[serde(flatten)]
    value: T,
}

impl<I: EntityId + Serialize, T: Serialize> Serialize for Collection<I, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter().map(|(id, value)| EntryRef { id, value }))
    }
}

impl<'de, I, T> Deserialize<'de> for Collection<I, T>
where
    I: EntityId + Deserialize<'de>,
    T: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entries = Vec::<Entry<I, T>>::deserialize(deserializer)?;
        Self::from_entries(
            entries
                .into_iter()
                .map(|entry| (entry.id, entry.value))
                .collect(),
        )
        .map_err(|id| serde::de::Error::custom(format!("duplicate {id}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PatternId;

    fn collection() -> Collection<PatternId, &'static str> {
        let mut collection = Collection::new();
        collection.push("a");
        collection.push("b");
        collection.push("c");
        collection
    }

    #[test]
    fn ids_are_never_reused() {
        let mut items = collection();
        let removed = items.remove(PatternId::new(3));
        assert!(removed.is_some());
        assert_eq!(items.push("d"), PatternId::new(4));
    }

    #[test]
    fn restore_puts_value_back_in_place() {
        let mut items = collection();
        let removed = items.remove(PatternId::new(2)).expect("exists");
        assert_eq!(removed.index, 1);
        items.restore(removed).expect("id is free");
        let order: Vec<_> = items.iter().map(|(_, value)| *value).collect();
        assert_eq!(order, ["a", "b", "c"]);
    }

    #[test]
    fn restore_rejects_taken_id() {
        let mut items = collection();
        let duplicate = Removed {
            id: PatternId::new(1),
            index: 0,
            value: "x",
        };
        assert_eq!(items.restore(duplicate), Err(PatternId::new(1)));
    }

    #[test]
    fn from_entries_continues_after_highest_id() {
        let mut items =
            Collection::from_entries(vec![(PatternId::new(7), "a"), (PatternId::new(2), "b")])
                .expect("unique ids");
        assert_eq!(items.push("c"), PatternId::new(8));
    }
}
