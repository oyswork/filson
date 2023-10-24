use crate::{DataNode, FilsonResult};

/// Filson can run comparisons over any data types, as long as they implement [Extractable].
pub trait Extractable {
    /// Lets say, you came to a used car dealership and want to view all the cars that are less than 5 years old and are of certain make.
    /// In order to do that you have to instruct Filson how to retrieve the required fields and convert them into [DataNode]s, which is the data format Filson understands.
    ///
    /// ```rust
    /// use filson::{get_filter, Appliable, DataNode, Extractable, FilsonError};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Car {
    ///     make: &'static str,
    ///     age: u8,
    ///     mileage: u64,
    /// }
    ///
    /// let cars = vec![
    ///     Car {
    ///         make: "Ford",
    ///         age: 7,
    ///         mileage: 100000,
    ///     },
    ///     Car {
    ///         make: "Volvo",
    ///         age: 3,
    ///         mileage: 50000,
    ///     },
    ///     Car {
    ///         make: "Volvo",
    ///         age: 4,
    ///         mileage: 47000,
    ///     },
    /// ];
    ///
    /// // let's implement the Extractable so,
    /// // that Filson knows how to extract age and make data from Car structs
    /// impl Extractable for Car {
    ///     fn extract(&self, path: &str) -> Result<DataNode, FilsonError> {
    ///         match path {
    ///             "make" => Ok(self.make.into()),
    ///             // DataNode only accepts i64 integers so the explicit conversion is required
    ///             "age" => Ok((self.age as i64).into()),
    ///             // We don't need to be able to extract milage field
    ///             _ => Err(FilsonError::ExtractionError),
    ///         }
    ///     }
    /// }
    ///
    /// // Now we ready to build the Filson query and run it over our data
    /// // Remember, that query may be known only at runtime
    /// let query = r#"and(compare("make" == "Volvo"), compare("age" < 5))"#;
    /// let flt = get_filter(query).unwrap();
    ///
    /// let filtered_cars = cars
    ///     .into_iter()
    ///     .filter(|car| flt.apply(car).unwrap_or(false))
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     filtered_cars,
    ///     vec![
    ///         Car {
    ///             make: "Volvo",
    ///             age: 3,
    ///             mileage: 50000
    ///         },
    ///         Car {
    ///             make: "Volvo",
    ///             age: 4,
    ///             mileage: 47000
    ///         }
    ///     ]
    /// );
    /// ```
    fn extract(&self, path: &str) -> FilsonResult<DataNode>;
}
