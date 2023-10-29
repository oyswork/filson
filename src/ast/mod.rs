mod ast;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "extraction_caching")] {
        mod extraction_utils_cached;
    } else {
        mod extraction_utils_uncached;
    }
}

pub(crate) use ast::Ast;
