//! # Filson
//! A simple DSL that can be embedded into rust applications to perform data filtration when:
//! - Filtration conditions are known only at runtime.
//! - Data shape may be inconsistent.
//!
//! The name comes from **fil**tering j**son**s, which was the original intended use.
//!
//! # Disclamer
//! **This project is in it's infancy, there will be breaking changes! Use with caution.**
//!
//! Filson has begun as a leaning project to explore rust, so it is made to be extremely simple to both use and reason about it's code.
//!
//! While Filson is not made to be fast(yet), but depending on the case it may be fast enough™.
//!
//! # Quickstart
//!
//! ```rust
//! use serde_json::json;
//!
//! use filson::{Appliable, get_filter};
//!
//! let array_to_filter = [json!({"num": 1}), json!({"num": 2})];
//!
//! // note the json pointer syntax
//! // Remember that this condition may be known only at runtime
//! let cond = r#"compare("/num" == 1)"#;
//! let flt = get_filter(cond).unwrap();
//!
//! let res = array_to_filter
//!             .into_iter()
//!             .filter(|data_point| flt.apply(data_point).unwrap_or(false))
//!             .collect::<Vec<_>>();
//! assert_eq!(res, vec![json!({"num": 1})]);
//! ```
//!
//! # The concepts
//!
//! [get_filter]
//! This function is the single entry point.
//! It accepts an ```&str``` as input with the filtration condition written in Filson syntax(see syntax reference).
//! This condition may be known only at runtime, say, recieved over the network or supplied as a command line argument.
//! It builds the opaque [Appliable] object which knows how to apply the filtration over your data.
//! Please note, that your data type has to implement [Extractable].
//!
//!
//! [DataNode] is the internal representation of data used by Filson.
//! In order to run comparisons over your data it has to be converted during extraction. See [Extractable] for examples.
//!
//!
//! [Appliable] is a trait that defines the behaviour of filtering conditions and how they are applied over the data. See [examples](Appliable).
//!
//!
//! [Extractable] is a triat that defines how to **extract** the data from any type that implements and convert it to [DataNode]. See [examples](Extractable).

mod actors;
mod ast;
mod error;
mod integrations;
mod parser;
mod traits;
mod types;

use crate::parser::get_ast;

pub use error::{FilsonError, FilsonResult};
pub use traits::{Appliable, Extractable};
pub use types::DataNode;

pub fn get_filter(inp: &str) -> FilsonResult<impl Appliable + '_> {
    get_ast(inp)
}
