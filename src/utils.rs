use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
};

use crate::{DataNode, Extractable, FilsonError};

struct MaybeEntry<'a, K, V> {
    entry: Entry<'a, K, V>,
}

impl<'a, K, V> MaybeEntry<'a, K, V> {
    fn or_try_insert_with<E: Error, F: FnOnce() -> Result<V, E>>(
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

impl<'a, K, V> From<Entry<'a, K, V>> for MaybeEntry<'a, K, V> {
    fn from(value: Entry<'a, K, V>) -> Self {
        Self { entry: value }
    }
}

pub(crate) fn try_extract_cached<'a>(
    lhs: &'a str,
    v: &'a impl Extractable,
    cache: &'a mut HashMap<&'a str, DataNode<'a>>,
) -> Result<&'a DataNode<'a>, FilsonError> {
    let val = MaybeEntry::from(cache.entry(lhs)).or_try_insert_with(|| v.extract(lhs))?;
    Ok(val)
}
