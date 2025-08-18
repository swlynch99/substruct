#[test]
fn trybuild() {
    // This still compiles the crate normally, it just means that we also get
    // clippy warnings as well.
    std::env::set_var("RUSTC_WORKSPACE_WRAPPER", "clippy-driver");

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
    t.pass("tests/ui/pass/*.rs");
}
