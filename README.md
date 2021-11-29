# Derive debug extras

More customisable `#[derive(Debug)]`

Adds three options:

### `#[debug_single_tuple_inline]`

```rs
use derive_debug_extras::DebugMore;

#[derive(DebugMore)]
#[debug_single_tuple_inline]
struct A(pub u32);
```

Verbose debugging on `A` retains single line under verbose. 

e.g. for `println!("{:#?}", vec![A(123), A(145), A(125),])`
```js
// With #[debug_single_tuple_inline]
[
    A(123),
    A(145),
    A(125),
]
// Without #[debug_single_tuple_inline]
[
    A(
        123
    ),
    A(
        145
    ),
    A(
        125
    ),
]
```

`#[debug_single_tuple_inline]` works on enums as well. Will fail for unnamed tuples with more than one field or named fields.

### `#[debug_ignore]`

```rs
#[derive(DebugMore)]
struct C {
    x: u32,
    #[debug_ignore]
    _y: bool,
}
```

Ignores the `_y` field when debugging

### `#[debug_as_display]`

```rs
#[derive(DebugMore)]
struct D {
    #[debug_as_display]
    x: String,
}
```

Prints the `x` field out as if it was formatted with `Display`

e.g. for `println!("{:#?}", D("Hello World".to_string()))`
```js
// With #[debug_as_display]
D(Hello World)
// Without #[debug_as_display]
D("Hello World")
```