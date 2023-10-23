use lexical::Error as LexicalError;
use thiserror::Error as ThisError;

type PestError = pest::error::Error<crate::parser::Rule>;

#[derive(Debug, Clone, ThisError, PartialEq, Eq)]
pub enum FilsonError {
    #[error(transparent)]
    NumberParseError(#[from] LexicalError),

    #[error(transparent)]
    ParseError(#[from] Box<PestError>),

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
}

pub(crate) type FilsonResult<T> = Result<T, FilsonError>;
