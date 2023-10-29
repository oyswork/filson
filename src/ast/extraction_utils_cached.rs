use crate::{DataNode, Extractable, FilsonResult};

use fxhash::FxHashMap;

use std::{collections::hash_map::Entry, error::Error, ptr::NonNull};

pub(super) type CacheType<'a> = NonNull<FxHashMap<&'a str, DataNode<'a>>>;

pub(super) struct FalliableEntry<'a, K, V> {
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

#[inline]
pub(super) fn get_extractable<'a>(
    path: &'a str,
    extractable: &'a impl Extractable,
    cache: Option<CacheType<'a>>,
) -> FilsonResult<&'a mut DataNode<'a>> {
    let cache_ptr = cache.expect("Expected a pointer to cache to be present");
    unsafe {
        FalliableEntry::from((*cache_ptr.as_ptr()).entry(path))
            .or_try_insert_with(|| extractable.extract(path))
    }
}
