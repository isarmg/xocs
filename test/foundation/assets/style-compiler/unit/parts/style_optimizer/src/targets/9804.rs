#[test]
fn supports_scope_correctly() {
  let mut targets = TargetsWithSupportsScope::new(Targets::default());
  assert!(!targets.current.exclude.contains(Features::OklabColors));
  assert!(!targets.current.exclude.contains(Features::LabColors));
  assert!(!targets.current.exclude.contains(Features::P3Colors));

  targets.enter_supports(Features::OklabColors | Features::LabColors);
  assert!(targets.current.exclude.contains(Features::OklabColors));
  assert!(targets.current.exclude.contains(Features::LabColors));

  targets.enter_supports(Features::P3Colors | Features::LabColors);
  assert!(targets.current.exclude.contains(Features::OklabColors));
  assert!(targets.current.exclude.contains(Features::LabColors));
  assert!(targets.current.exclude.contains(Features::P3Colors));

  targets.exit_supports();
  assert!(targets.current.exclude.contains(Features::OklabColors));
  assert!(targets.current.exclude.contains(Features::LabColors));
  assert!(!targets.current.exclude.contains(Features::P3Colors));

  targets.exit_supports();
  assert!(!targets.current.exclude.contains(Features::OklabColors));
  assert!(!targets.current.exclude.contains(Features::LabColors));
}
