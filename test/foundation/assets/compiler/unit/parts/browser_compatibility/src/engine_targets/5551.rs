#[test]
fn test_displayed_value_is_parsable() {
    let target = EngineTargets::new(FxHashMap::from_iter([
        (Engine::Chrome, Version(139, 0, 0)),
        (Engine::Deno, Version(2, 5, 1)),
        (Engine::Es, Version(2024, 0, 0)),
    ]));
    let s = target.to_string();
    let parsed = EngineTargets::from_target(&s).unwrap();
    assert_eq!(target.0, parsed.0);
}
