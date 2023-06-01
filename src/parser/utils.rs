use core::num::NonZeroU8;
use lexical::parse_with_options;

use crate::{error::FilsonResult, FilsonError};

const INT_FORMAT: u128 = lexical::NumberFormatBuilder::new()
    .digit_separator(NonZeroU8::new(b'_'))
    .internal_digit_separator(true)
    .build();

const FLOAT_FORMAT: u128 = lexical::NumberFormatBuilder::new()
    .digit_separator(NonZeroU8::new(b'_'))
    .required_digits(true)
    .no_exponent_without_fraction(true)
    .internal_digit_separator(true)
    .case_sensitive_exponent(false)
    .build();

pub(crate) fn parse_int(inp_str: &str) -> FilsonResult<i64> {
    let options = lexical::ParseIntegerOptions::new();
    parse_with_options::<i64, _, INT_FORMAT>(inp_str.as_bytes(), &options)
        .map_err(FilsonError::from)
}

pub(crate) fn parse_float(inp_str: &str) -> FilsonResult<f64> {
    let options = lexical::ParseFloatOptions::new();
    parse_with_options::<f64, _, FLOAT_FORMAT>(inp_str.as_bytes(), &options)
        .map_err(FilsonError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int("1").unwrap(), 1);
        assert_eq!(parse_int("+1").unwrap(), 1);
        assert_eq!(parse_int("-1").unwrap(), -1);
        assert_eq!(parse_int("1_2").unwrap(), 12);
        assert!(parse_int("_1").is_err());
        assert!(parse_int("1_").is_err());
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_float("1.2").unwrap(), 1.2);
        assert_eq!(parse_float("+1.2").unwrap(), 1.2);
        assert_eq!(parse_float("-1.2").unwrap(), -1.2);
        assert!(parse_float(".1").is_err());
        assert!(parse_float("1.").is_err());
        assert_eq!(parse_float("1_2.34").unwrap(), 12.34);
        assert_eq!(parse_float("12.3_4").unwrap(), 12.34);
        assert!(parse_float("_1.0").is_err());
        assert!(parse_float("1_.0").is_err());
        assert!(parse_float("1._0").is_err());
        assert!(parse_float("1.0_").is_err());
        assert!(parse_float("1.0e1_").is_err());
        assert!(parse_float("1.0e_1").is_err());
        assert_eq!(parse_float("1.0e1").unwrap(), 10.0);
        assert_eq!(parse_float("1.0E1").unwrap(), 10.0);
        assert!(parse_float("1.0e").is_err());
        assert!(parse_float("1.e").is_err());
        assert_eq!(parse_float("1.0e+1").unwrap(), 10.0);
        assert_eq!(parse_float("1.0e-1").unwrap(), 0.1);
        assert_eq!(parse_float("1.0e1_0").unwrap(), 10000000000.0);
    }
}
