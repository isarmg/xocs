
    use super::*;
    use std::ffi::OsString;

    fn snapshot(vars: &[(&str, &str)]) -> EnvSnapshot {
        EnvSnapshot::from_pairs(vars)
    }

    fn envs(cmd: &Command) -> Vec<(OsString, Option<OsString>)> {
        cmd.get_envs()
            .map(|(key, value)| (key.to_owned(), value.map(OsStr::to_owned)))
            .collect()
    }

    #[test]
    fn apply_replaces_the_whole_environment() {
        let mut cmd = Command::new("cc");
        cmd.env("CC_TEST_SET_BEFORE", "1")
            .env_remove("CC_TEST_REMOVED_BEFORE");
        snapshot(&[("CC_TEST_IN_SNAPSHOT", "1")]).apply(&mut cmd);
        assert_eq!(
            envs(&cmd),
            [("CC_TEST_IN_SNAPSHOT".into(), Some("1".into()))]
        );
    }

    #[test]
    fn build_env_wins_over_snapshot() {
        let env = BuildEnv::with_inherited(
            snapshot(&[("CC_TEST_ORDER", "inherited")]),
            vec![(
                OsStr::new("CC_TEST_ORDER").into(),
                OsStr::new("explicit").into(),
            )],
        );
        let mut cmd = Command::new("cc");
        env.apply(&mut cmd);
        assert_eq!(
            envs(&cmd),
            [("CC_TEST_ORDER".into(), Some("explicit".into()))]
        );
    }

    #[test]
    fn snapshot_lookup_shares_the_value() {
        let snapshot = snapshot(&[("CC_TEST_SHARED", "a")]);
        let value = snapshot.get(OsStr::new("CC_TEST_SHARED")).unwrap();
        assert!(Arc::ptr_eq(value, &snapshot.vars[0].1));
    }

    #[test]
    fn snapshot_lookup_follows_the_platform_case_rule() {
        let snapshot = snapshot(&[("Path", "a")]);
        assert_eq!(
            snapshot.get(OsStr::new("Path")).map(|value| &**value),
            Some(OsStr::new("a"))
        );
        let other_case = snapshot.get(OsStr::new("PATH")).map(|value| &**value);
        if cfg!(windows) {
            assert_eq!(other_case, Some(OsStr::new("a")));
        } else {
            assert_eq!(other_case, None);
        }
    }

    #[test]
    fn snapshot_debug_hides_values() {
        let debug = format!("{:?}", snapshot(&[("CC_TEST_SECRET", "hunter2")]));
        assert!(!debug.contains("hunter2"), "{debug}");
    }
