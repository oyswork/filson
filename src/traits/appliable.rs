use crate::{Extractable, FilsonError};

pub trait Appliable: Send {
    /// Most likely you will never have to implement it yourself.
    /// But for completeness sake, let's build on the `Quickstart` example.
    ///
    /// ```rust
    /// use serde_json::json;
    /// use filson::{Appliable, Extractable, DataNode, get_filter, FilsonError};
    ///
    /// struct TogglableFilter<T: Appliable> {
    ///     is_on: bool,
    ///     flt: T,
    /// }
    ///
    /// impl<T: Appliable> TogglableFilter<T> {
    ///     fn toggle(&mut self) {
    ///         self.is_on = !self.is_on;
    ///     }
    /// }
    ///
    /// // Technically, you don't have to implement Appliable for your type.
    /// // But you might want to do it if you need to re-export the Appliable trait.
    /// impl<T: Appliable> Appliable for TogglableFilter<T> {
    ///     fn apply<Y: Extractable>(&self, v: &Y) -> Result<bool, FilsonError> {
    ///         // you can do whatever you like in here
    ///         // for example query some data from db
    ///         // and then use it
    ///         // or perform arbitraty input data transformations
    ///         Ok(self.is_on && self.flt.apply(v)?)
    ///     }
    /// }
    ///
    /// let array_to_filter = [
    ///     json!({"num": 1}),
    ///     json!({"num": 2}),
    /// ];
    ///
    /// // note the json pointer syntax
    /// // Remember, that query may be known only at runtime
    /// let cond = r#"compare("/num" == 1)"#;
    /// let mut flt_wrapper = TogglableFilter{is_on: true, flt: get_filter(cond).unwrap()};
    ///
    /// let res = array_to_filter
    ///             .clone()
    ///             .into_iter()
    ///             .filter(|data_point| flt_wrapper.apply(data_point).unwrap_or(false))
    ///             .collect::<Vec<_>>();
    ///
    /// assert_eq!(res, vec![ json!({"num": 1})]);
    ///
    /// flt_wrapper.toggle();
    /// let res = array_to_filter
    ///             .into_iter()
    ///             .filter(|data_point| flt_wrapper.apply(data_point).unwrap_or(false))
    ///             .collect::<Vec<_>>();
    ///
    /// assert_eq!(res, Vec::<serde_json::Value>::new());
    /// ```
    fn apply<T: Extractable>(&self, v: &T) -> Result<bool, FilsonError>;
}
