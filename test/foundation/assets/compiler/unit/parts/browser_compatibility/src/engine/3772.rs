#[test]
fn test_displayed_value_is_parsable() {
    let engines = vec![
        Engine::Chrome,
        Engine::Deno,
        Engine::Edge,
        Engine::Firefox,
        Engine::Hermes,
        Engine::Ie,
        Engine::Ios,
        Engine::Node,
        Engine::Opera,
        Engine::Rhino,
        Engine::Safari,
        Engine::Samsung,
        Engine::Electron,
        Engine::OperaMobile,
        Engine::Android,
        Engine::Es,
    ];
    for engine in engines {
        // Engine::ES is handled by `ESTarget`
        if engine == Engine::Es {
            continue;
        }
        assert!(engine.to_string().parse::<Engine>().is_ok(), "\"{engine}\" should be parsable");
    }
}
