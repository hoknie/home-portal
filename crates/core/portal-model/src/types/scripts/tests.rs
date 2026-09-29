use super::{ScriptPath, ScriptPathProblem};

#[test]
fn a_script_path_must_stay_inside_the_directory() {
    assert!(ScriptPath::parse("restart.sh").is_ok());
    assert!(ScriptPath::parse("media/restart.sh").is_ok());
    assert!(ScriptPath::parse("./restart.sh").is_ok());
    for refused in [
        "",
        "  ",
        "/bin/sh",
        "../home-portal.toml",
        "media/../../x",
        ".",
    ] {
        assert!(ScriptPath::parse(refused).is_err(), "{refused:?}");
    }
}

#[test]
fn each_refused_path_names_its_own_problem() {
    assert_eq!(ScriptPath::parse(""), Err(ScriptPathProblem::Empty));
    assert_eq!(
        ScriptPath::parse("/bin/sh"),
        Err(ScriptPathProblem::Absolute)
    );
    assert_eq!(ScriptPath::parse("../x.sh"), Err(ScriptPathProblem::Climbs));
    assert_eq!(
        ScriptPath::parse(".cache/x.sh"),
        Err(ScriptPathProblem::Hidden)
    );
    assert_eq!(
        ScriptPath::parse("a/b/c.sh"),
        Err(ScriptPathProblem::TooDeep)
    );
}

#[test]
fn a_path_knows_its_folder_and_its_name() {
    let path = ScriptPath::parse("./media/restart.sh").unwrap();
    assert_eq!(path.text(), "media/restart.sh");
    assert_eq!(path.folder(), Some("media"));
    assert_eq!(path.name(), "restart.sh");
    let top = ScriptPath::parse("backup.sh").unwrap();
    assert_eq!(top.folder(), None);
}

mod headers {
    use super::super::{ArgumentKind, ScriptHeader};

    #[test]
    fn a_shell_header_declares_its_description_and_arguments() {
        let header = ScriptHeader::parse(
            "#!/bin/sh\n# @description Restart a service's container\n# @arg service <text> Service id\n# @arg --retries <number=3> Tries before giving up\n# @arg --force Skip the health check\nset -eu\n",
        );
        assert_eq!(
            header.description.as_deref(),
            Some("Restart a service's container")
        );
        assert!(header.problems.is_empty(), "{:?}", header.problems);
        let names: Vec<&str> = header
            .arguments
            .iter()
            .map(|argument| argument.name.as_str())
            .collect();
        assert_eq!(names, vec!["service", "--retries", "--force"]);
        assert!(header.arguments[0].required);
        assert_eq!(header.arguments[0].description, "Service id");
        assert_eq!(header.arguments[1].kind, ArgumentKind::Number);
        assert_eq!(header.arguments[1].default.as_deref(), Some("3"));
        assert!(!header.arguments[1].required);
        assert_eq!(header.arguments[2].kind, ArgumentKind::Flag);
    }

    #[test]
    fn a_choice_with_a_default_is_an_optional_positional() {
        let header = ScriptHeader::parse("# @arg mode <fast|full=fast> How deep to check\n");
        let mode = &header.arguments[0];
        assert_eq!(
            mode.kind,
            ArgumentKind::Choice(vec!["fast".into(), "full".into()])
        );
        assert_eq!(mode.default.as_deref(), Some("fast"));
        assert!(!mode.required);
    }

    #[test]
    fn the_header_ends_at_the_first_line_that_is_not_a_comment() {
        let header = ScriptHeader::parse("# @arg name <text>\nset -e\n# @arg late <text>\n");
        assert_eq!(header.arguments.len(), 1);
    }

    #[test]
    fn a_flag_with_a_default_is_a_problem_naming_its_line() {
        let header = ScriptHeader::parse("#!/bin/sh\n# @arg --force <flag=true> Always\n");
        assert!(header.arguments.is_empty());
        assert_eq!(header.problems[0].line, 2);
        assert!(
            header.problems[0]
                .message
                .contains("a flag takes no default")
        );
    }

    #[test]
    fn an_unknown_type_drops_only_its_line() {
        let header = ScriptHeader::parse("# @arg --keep <weird> Days\n# @arg name\n");
        assert_eq!(header.arguments.len(), 1);
        assert!(
            header.problems[0]
                .message
                .contains("weird is not a known type")
        );
    }

    #[test]
    fn a_second_name_a_required_after_an_optional_and_a_second_description_are_problems() {
        let header = ScriptHeader::parse(
            "// @description One\n// @description Two\n// @arg host?\n// @arg host\n// @arg port <number>\n",
        );
        assert_eq!(header.description.as_deref(), Some("One"));
        let lines: Vec<usize> = header.problems.iter().map(|problem| problem.line).collect();
        assert_eq!(lines, vec![2, 4, 5]);
        assert_eq!(header.arguments.len(), 1);
    }

    #[test]
    fn at_most_thirty_two_arguments_are_declared() {
        let text: String = (0..40)
            .map(|index| format!("# @arg --o{index}\n"))
            .collect();
        let header = ScriptHeader::parse(&text);
        assert_eq!(header.arguments.len(), ScriptHeader::MOST_ARGUMENTS);
        assert_eq!(header.problems.len(), 8);
    }

    #[test]
    fn other_tags_and_plain_comments_are_ignored() {
        let header = ScriptHeader::parse("# Copyright nobody\n# @author me\n#\n# @arg x\n");
        assert!(header.problems.is_empty());
        assert_eq!(header.arguments.len(), 1);
    }
}
