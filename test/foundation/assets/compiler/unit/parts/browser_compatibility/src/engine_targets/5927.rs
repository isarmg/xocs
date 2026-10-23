#[test]
fn test_supports_es_feature() {
    use crate::browser_compatibility::ESFeature::{
        ES2015ArrowFunctions, ES2015RegExpConstructorCanAlterFlags, ES2020OptionalChaining,
    };

    assert!(!EngineTargets::default().supports_es_feature(ES2015ArrowFunctions));

    assert!(
        EngineTargets::from_target("es2015").unwrap().supports_es_feature(ES2015ArrowFunctions)
    );
    assert!(
        EngineTargets::from_target("esnext").unwrap().supports_es_feature(ES2020OptionalChaining)
    );
    assert!(
        !EngineTargets::from_target("es2019").unwrap().supports_es_feature(ES2020OptionalChaining)
    );
    assert!(
        EngineTargets::from_target("es2020").unwrap().supports_es_feature(ES2020OptionalChaining)
    );

    // Browser targets also contain an implicit `esnext`. All configured engines must support the
    // feature, regardless of compatibility-table iteration order.
    assert!(
        !EngineTargets::from_target("chrome90")
            .unwrap()
            .supports_es_feature(ES2020OptionalChaining)
    );
    assert!(
        EngineTargets::from_target("chrome91").unwrap().supports_es_feature(ES2020OptionalChaining)
    );

    assert!(
        !EngineTargets::from_target("chrome48")
            .unwrap()
            .supports_es_feature(ES2015RegExpConstructorCanAlterFlags)
    );
    assert!(
        EngineTargets::from_target("chrome49")
            .unwrap()
            .supports_es_feature(ES2015RegExpConstructorCanAlterFlags)
    );

    // Missing compatibility data is not proof of support.
    let rhino = EngineTargets::new(FxHashMap::from_iter([(Engine::Rhino, Version(1, 7, 15))]));
    assert!(!rhino.supports_es_feature(ES2020OptionalChaining));
}
