use std::{collections::hash_map::Entry, error::Error};

pub(crate) struct FalliableEntry<'a, K, V> {
    entry: Entry<'a, K, V>,
}

impl<'a, K, V> FalliableEntry<'a, K, V> {
    pub fn or_try_insert_with<E: Error, F: FnOnce() -> Result<V, E>>(
        self,
        default: F,
    ) -> Result<&'a mut V, E> {
        match self.entry {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let val = default()?;
                Ok(entry.insert(val))
            }
        }
    }
}

impl<'a, K, V> From<Entry<'a, K, V>> for FalliableEntry<'a, K, V> {
    fn from(value: Entry<'a, K, V>) -> Self {
        Self { entry: value }
    }
}
