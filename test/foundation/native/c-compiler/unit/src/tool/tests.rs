
    use super::*;

    #[test]
    fn tool_env_wins_over_inherited_env() {
        let mut tool = Tool::with_family(
            "cc".into(),
            ToolFamily::Gnu,
            EnvSnapshot::from_pairs(&[("CC_TEST_ORDER", "inherited")]),
        );
        tool.env.push(("CC_TEST_ORDER".into(), "tool".into()));
        let cmd = tool.to_command();
        let envs: Vec<_> = cmd.get_envs().collect();
        assert_eq!(
            envs,
            [(OsStr::new("CC_TEST_ORDER"), Some(OsStr::new("tool")))]
        );
    }
