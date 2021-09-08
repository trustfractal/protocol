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

#[test]
fn construct_numbers() {
    let mut parser = Parser::default();
    parser.add_file_defs(MULTIPLE_NUMBERS).unwrap();

    let def = parser.struct_def("Foo").unwrap();
    let message = def
        .builder()
        .set("foo", 42u64)
        .set("bar", 43u32)
        .try_build()
        .unwrap();

    assert_eq!(message["foo"].as_u64(), Some(42));
    assert_eq!(message["bar"].as_u32(), Some(43));
}

#[test]
fn serialize_object() {
    let mut parser = Parser::default();
    parser.add_file_defs(MULTIPLE_NUMBERS).unwrap();

    let def = parser.struct_def("Foo").unwrap();
    let message = def
        .builder()
        .set("foo", 42u64)
        .set("bar", 43u32)
        .try_build()
        .unwrap();

    let encoded = message.serialize();
    let expected = def
        .id()
        .iter()
        .chain(&[42, 0, 0, 0, 0, 0, 0, 0, 43, 0, 0, 0])
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(encoded, expected);
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

const LIST: &'static str = r#"
struct Foo {
    foo :List<u8>;
}
"#;

#[test]
fn list() {
    let mut parser = Parser::default();
    parser.add_file_defs(LIST).unwrap();

    let id = parser.struct_def("Foo").unwrap().id();
    let encoded_list = [4, 2];
    let encoded = id
        .iter()
        .chain(&[encoded_list.len() as u8])
        .chain(&encoded_list)
        .cloned()
        .collect::<Vec<_>>();

    let message = parser.parse(encoded.as_ref()).unwrap();

    assert_eq!(
        message["foo"]
            .as_list()
            .unwrap()
            .into_iter()
            .map(|v| v.as_u8().unwrap())
            .collect::<Vec<_>>(),
        vec![4, 2]
    );
}

const STRUCT: &'static str = r#"
struct Foo {
    foo :u8;
}

struct Bar {
    bar :Foo;
}
"#;

#[test]
fn struct_() {
    let mut parser = Parser::default();
    parser.add_file_defs(STRUCT).unwrap();

    let def = parser.struct_def("Foo").unwrap();
    let message = def.builder().set("foo", 42u8).try_build().unwrap();

    let bar_def = parser.struct_def("Bar").unwrap();
    let bar_message = bar_def.builder().set("bar", message).try_build().unwrap();

    let obj = bar_message["bar"].as_object().unwrap();

    assert_eq!(obj["foo"].as_u8(), Some(42));
}
