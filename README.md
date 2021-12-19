# Derive debug extras

![](https://img.shields.io/crates/v/derive-debug-extras)

More customisable `#[derive(Debug)]`

Adds three options:

### `#[debug_single_tuple_inline]`

```rust
use derive_debug_extras::DebugExtras;

#[derive(DebugExtras)]
#[debug_single_tuple_inline]
struct A(pub u32);
```

Verbose debugging on `A` retains single line under verbose. 

e.g. for `println!("{:#?}", vec![A(123), A(145), A(125),])`
```rust
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

For setting this as the default for verbose formatting for structures with one field that use `#[derive(DebugExtras)]`, you can use the `auto-debug-single-tuple-inline` feature to prevent having to write `#[debug_single_tuple_inline]` on every structure

### `#[debug_ignore]`

```rust
#[derive(DebugExtras)]
struct C {
    x: u32,
    #[debug_ignore]
    _y: bool,
}
```

Ignores the `_y` field when debugging

### `#[debug_as_display]`

```rust
#[derive(DebugExtras)]
struct D {
    #[debug_as_display]
    x: String,
}
```

Prints the `x` field out as if it was formatted with `Display`

e.g. for `println!("{:#?}", D("Hello World".to_string()))`
```rust
// With #[debug_as_display]
D(Hello World)
// Without #[debug_as_display]
D("Hello World")
```