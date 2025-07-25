use trybuild;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/module/fail/*.rs");
    t.pass("tests/module/pass/*.rs");
}
