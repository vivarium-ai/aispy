use aispy::observe;

#[observe(kind = "test")]
fn observed_function() -> u32 {
    42
}

#[test]
fn observe_macro_preserves_function_behavior() {
    assert_eq!(observed_function(), 42);
}
