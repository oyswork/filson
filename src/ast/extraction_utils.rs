use crate::{DataNode, Extractable, FilsonResult};

#[cfg(feature = "extraction_caching")]
use fxhash::FxHashMap;

#[cfg(feature = "extraction_caching")]
use std::{collections::hash_map::Entry, error::Error};

#[cfg(feature = "extraction_caching")]
pub(super) type CacheType<'a> = *mut FxHashMap<&'a str, DataNode<'a>>;

#[cfg(feature = "extraction_caching")]
pub(super) struct FalliableEntry<'a, K, V> {
    entry: Entry<'a, K, V>,
}

#[cfg(feature = "extraction_caching")]
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

#[cfg(feature = "extraction_caching")]
impl<'a, K, V> From<Entry<'a, K, V>> for FalliableEntry<'a, K, V> {
    fn from(value: Entry<'a, K, V>) -> Self {
        Self { entry: value }
    }
}

#[cfg(feature = "extraction_caching")]
#[inline]
pub(super) fn get_extractable<'a>(
    path: &'a str,
    extractable: &'a impl Extractable,
    cache: Option<CacheType<'a>>,
) -> FilsonResult<&'a mut DataNode<'a>> {
    let cache_ptr = cache.expect("Expected a pointer to cache to be present");
    unsafe {
        FalliableEntry::from((*cache_ptr).entry(path))
            .or_try_insert_with(|| extractable.extract(path))
    }
}

#[cfg(not(feature = "extraction_caching"))]
use std::marker::PhantomData;

#[cfg(not(feature = "extraction_caching"))]
#[derive(Clone, Copy)]
pub(super) struct Nothing<'a>(PhantomData<&'a Self>);

#[cfg(not(feature = "extraction_caching"))]
pub(super) type CacheType<'a> = Nothing<'a>;

#[cfg(not(feature = "extraction_caching"))]
#[inline]
pub(super) fn get_extractable<'a>(
    path: &'a str,
    extractable: &'a impl Extractable,
    _cache: Option<CacheType>,
) -> FilsonResult<DataNode<'a>> {
    extractable.extract(path)
}
