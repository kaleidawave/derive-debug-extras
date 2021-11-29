use derive_debug_extras::DebugMore;

#[derive(DebugMore)]
#[debug_single_tuple_inline]
struct A(pub u32);

#[test]
fn struct_tuple_inline() {
    assert_eq!(format!("{:#?}", A(154)), "A(154)");
}

#[derive(DebugMore)]
#[debug_single_tuple_inline]
enum B {
    X(String),
    Y(f32),
}

#[test]
fn enum_tuple_inline() {
    assert_eq!(format!("{:#?}", B::X("Hi".to_owned())), "B::X(\"Hi\")");
    assert_eq!(format!("{:#?}", B::Y(123.1)), "B::Y(123.1)");
}

#[derive(DebugMore)]
struct C {
    x: u32,
    #[debug_ignore]
    _y: bool,
}

#[test]
fn debug_ignore_field() {
    assert_eq!(format!("{:?}", C { x: 12, _y: false }), "C { x: 12 }");
}

#[derive(DebugMore)]
struct D {
    #[debug_as_display]
    x: String,
}

#[test]
fn debug_as_display() {
    assert_eq!(
        format!(
            "{:?}",
            D {
                x: "Hello World".to_owned()
            }
        ),
        "D { x: Hello World }"
    );
}
