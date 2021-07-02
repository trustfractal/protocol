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
