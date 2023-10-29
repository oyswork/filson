use crate::parser::Rule;
use lexical::Error as LexicalError;
use pest::error::Error as PestError;
use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError, PartialEq, Eq)]
pub enum FilsonError {
    #[error(transparent)]
    NumberParseError(#[from] LexicalError),

    #[error(transparent)]
    ParseError(#[from] Box<PestError<Rule>>),

    #[error("Data by specified path doesn't exist.")]
    ExtractionError,

    #[error("Values are of different types.")]
    TypeError,

    #[error("Ordering operations between collection types are not allowed. Maybe you want to enable \"collection_ordering\" crate feature.")]
    OrderingProhibitedError,

    #[error("Can't check for intersection, since extracted data isn't array/set/map/string.")]
    IntersectsError,

    #[error("Can't check if subset, since extracted data isn't array/set/map/string.")]
    IsSubsetError,

    #[error("Can't check if superset, since extracted data isn't array/set/map/string.")]
    IsSupersetError,

    #[cfg(feature = "extraction_caching")]
    #[error("Ptr to cache was null")]
    CacheCreationError,
}

pub type FilsonResult<T> = Result<T, FilsonError>;
