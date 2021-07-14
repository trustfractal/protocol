use sier_codec::Parser;

const VOID_DEF: &'static str = r#"
struct Foo {}
"#;

#[test]
fn simple_void_type() {
    let mut parser = Parser::default();
    parser.add_file_defs(VOID_DEF).unwrap();

    let id = parser.struct_def("Foo").unwrap().id();
    let message = parser.parse(&id).unwrap();

    assert_eq!(message.schema().type_name(), "Foo");
}

const MULTIPLE_NUMBERS: &'static str = r#"
struct Foo {
    foo :u64;
    bar :u32;
}
"#;

#[test]
fn some_numbers() {
    let mut parser = Parser::default();
    parser.add_file_defs(MULTIPLE_NUMBERS).unwrap();

    let id = parser.struct_def("Foo").unwrap().id();
    let encoded = id
        .iter()
        .chain(&[42, 0, 0, 0, 0, 0, 0, 0, 43, 0, 0, 0])
        .cloned()
        .collect::<Vec<_>>();

    let message = parser.parse(encoded.as_ref()).unwrap();

    assert_eq!(message["foo"].as_u64(), Some(42));
    assert_eq!(message["bar"].as_u32(), Some(43));
}

const STRING: &'static str = r#"
struct Foo {
    foo :string;
}
"#;

#[test]
fn string() {
    let mut parser = Parser::default();
    parser.add_file_defs(STRING).unwrap();

    let id = parser.struct_def("Foo").unwrap().id();
    let encoded_string = "abc";
    let encoded = id
        .iter()
        .chain(&[encoded_string.len() as u8])
        .chain(encoded_string.as_bytes())
        .cloned()
        .collect::<Vec<_>>();

    let message = parser.parse(encoded.as_ref()).unwrap();

    assert_eq!(message["foo"].as_string(), Some("abc"));
}
