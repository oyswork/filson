# Filson

A simple DSL with dynamic strong typing that can be embedded into rust applications to perform data filtration when:

- Filtration conditions are known only at runtime.
- Data shape may be inconsistent.

The name stems from **Fil**tering j**son**s.

## Known issues (in the order of priority)

- Tree walking interpreter is slow, will move to VM in future
- No extraction caching, meaning that the same value from the same data entry will be extracted as many times as it is mentioned in the filtering condition.
- No support for datetime type, will be added in the future.
- All of the number types are strictly 64 bit signed. Support for larger types will be added in the future.
- All of the number types are strictly decimal, parser can't handle hex, octal or binary notation. Support will be added in the future.
- Actors code (and DataNode) desperately needs refactoring. Right now all of the types are represented by one single enum, as the result typechecking has to be done manually. Work should be offloaded onto rust's typesystem instead.

## Primitive types

- `integer` - signed 64 bit.

  Optional sign in front and optional `_` between digits are accepted.
  Examples:
  - 1
  - +1
  - 1
  - 1_2

- `float` - signed 64 bit.

  Optional sign in front and optional `_` between digits are accepted.
  Mantissa has to be present, but the integer part may be absent.
  Optional exponent part delimeted by `e` or `E` with optinal sign in front of it is also accepted.
  Examples:
  - 1.2,
  - +1.2,
  - -1.2,
  - .1,
  - 1_2.34,
  - 12.3_4,
  - 1.0e1,
  - 1.0E1
  - 1.0e+1
  - 1.0e-1
  - 1.0e1_0

- `boolean`
  - true
  - false

- `null`
  - null

- `string`

  Delimeted by `"` `"`
  > `"this is
            a string with $pec1al char's, punctuation and escapes \\ \" \t and
            maybe crabs 🦀"`

## Container types

- `array`

  Delimeted by `[` `]`. Can contain any other primitive and container types, including several different types at once.
  Represented by `Vec` under the hood, which means that is is actually ordered.
  > `[1, 2.0, "text", ["another", {"set"}, <"map_key": ["map value"]>], []]`

- `set`

  Delimeted by `{` `}`. Can contain any other primitive and container types, including several different types at once.
  Duplicate values may be present, but will be ignored during evaluation.
  Represented by `BTreeSet` under the hood, which means that is is actually ordered.

- `map`

   Delimeted by `<` `>`. Containt `key: value` pairs, where keys can be any primitive types and values can be any primitive or container types.
   Duplicate pairs may be present, but will be ignored during evaluation.
   Represented by `BTreeMap` under the hood, which means that is is actually ordered.

  - `<"key": 1, 2: {[10, 20]}, "karl": [], true: null>`

## Actors

### Foreword

All of the actors expect a `string` as a first argument.  That string should describe the path to the data that is used during comparisons and will be used in the implementation of `Extractable` trait to fetch the required piece of data.

Example - consider json `{"a": {"b": 1}}`.  if we want to extract the nested `{"b": 1}` json by using the key `"a"`, we then should use `"/a"` ([json pointer](https://www.rfc-editor.org/rfc/rfc6901) format) as an argument, or if we wish to extract `1`, then the argument would be `"/a/b"`.  

- `compare(lhs op rhs)`
  - lhs - any valid `string`
  - op - comparison operator, any of the `!=, ==, >, >=, <, <=`
  - rhs - any valid `primitive` or `container` type.  
  Example - consider json `{"a": {"b": 1}}`
       > Value that lies in `/a/b` shoud be greater than 0  
       > `compare("/a/b" > 0)`  

  **Important!** `compare` is strict in regards of the data types.
  Which means that type of value by the path in `lhs` should be the same as the type in `rhs` otherwise it will yield an error.

  **Important!** `compare` can perform ordering (`>`, `<`, `>=`, `<=`) operations on `container` types,
  but only if **`collection_ordering`** crate feature is enabled (**disabled** by default).
  `container` types are ordered **lexicographically**.

- `intersects(lhs rhs)`
  - lhs - any valid `string`, but value by that path should be `string/array/set/map`
  - rhs - `string/array/set/map`

  > Checks if `container` or a `string` in `lhs` and `rhs` have at least 1 common element.

  - Example - consider json `{"a": [1, 2, 3]}`.
    > Check if a `contaner` by the path `"/a"` and `[1, 5, 6]` have a common element.
    > `intersects("/a" [1, 5, 6])`

  **Important!**  `intersects` is strict in regards of the data types.
  Which means that type of value by the path in `lhs` should be the same as the type in `rhs` otherwise it will yield an error.

- `is_contained(lhs rhs)`
  - lhs - any valid `string`
  - rhs - `array/set/map`

   > Checks that whatever lies in `lhs` is contained within `rhs`.

  - Example - consider json `{"a": {"b": 1}}`.
    > Check that value by the path `"/a/b"` is contained within `[1, 2, 3, 4]`
    > `is_contained("/a/b" [1, 2, 3, 4])`

- `exists(path)`
  - path - any valid `string`.
  > Checks that **some** value exists in `path`, but ignores the actual value.
  - Example - consider json `{"a": [1, 2, 3]}`.
    > Some value should exist in `"/a"`
    > `exists("/a")`

- `is_subset(lhs rhs)`
  - lhs - - any valid `string`, but value by that path should be `string/array/set/map`.
  - rhs - `string/array/set/map`

   > Checks that whatever is in `lhs` is fully contained within `rhs`.

  - Example - consider json `{"a": [1, 2, 3]}`.
    > Value in `"/a"` is a subset of `[1, 2, 3, 4]`
    > `is_subset("/a" [1, 2, 3, 4])`

  **Important!**  `intersects` is strict in regards of the data types.
  Which means that type of value by the path in `lhs` should be the same as the type in `rhs` otherwise it will yield an error.

- `is_superset(lhs rhs)` - inversion of `is_subset`.

## Binary conditions

- `and(lhs, rhs)`
  - `lhs` and `rhs` - any valid actor or binary condition.

  > Checks that both `lhs` and `rhs` are true.

  - Example - consider json `{"a": {"b": 1}}`.
  > Some value exists in `"/a"` **and** value in `"/a/b"` is greater than 0.
  > `and(exists("/a"), compare("/a/b" > 0))`

- `or(lhs, rhs)`

- `xor(lhs, rhs)`

Binary conditions can be nested:

    and(
     xor(
      or(compare, and(exists, compare)),
      intersects
      ),
     exists
    )

## Negation operator

- `!` - prepends any `binary condition` or `actor`

  - Example - consider json `{"a" : {"b": 1}}`
  > Value in `"/c"` does **not** exist.
  > `!exists("/c")`
