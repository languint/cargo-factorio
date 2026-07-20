#![allow(clippy::expect_used, clippy::unwrap_used)]

#[test]
fn bundled_prototype_api_generates_allowlist() {
    let source = factorio_api_gen::generate_prototypes_from_bundled().expect("generate");
    for name in [
        "pub struct Item",
        "pub struct Recipe",
        "pub struct Technology",
        "pub struct Fluid",
        "pub struct AssemblingMachine",
    ] {
        assert!(
            source.contains(name),
            "missing `{name}` in generated prototypes"
        );
    }
    assert!(
        source.contains("prototype API"),
        "expected version header comment"
    );
}
