use crate::{DataNode, Extractable, FilsonResult};
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub(super) struct Nothing<'a>(PhantomData<&'a Self>);

pub(super) type CacheType<'a> = Nothing<'a>;

#[inline]
pub(super) fn get_extractable<'a>(
    path: &'a str,
    extractable: &'a impl Extractable,
    _cache: Option<CacheType>,
) -> FilsonResult<DataNode<'a>> {
    extractable.extract(path)
}
