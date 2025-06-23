#[test]
fn entry() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/macro/*.rs");
}
