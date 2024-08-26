#[test]
fn test_macro_compilation() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/instruction/*.rs");
}
