use clap::{CommandFactory, Parser};
use portal_services::ProbeKind;

use super::{Command, CommandLine, ProxyAction};

#[test]
fn the_command_line_definition_is_consistent() {
    CommandLine::command().debug_assert();
}

#[test]
fn no_command_leaves_the_choice_to_serve() {
    let line = CommandLine::try_parse_from(["home-portal"]).unwrap();
    assert_eq!(line.command, None);
}

#[test]
fn a_probe_kind_choice_becomes_the_probe_kind_of_the_same_name() {
    let line =
        CommandLine::try_parse_from(["home-portal", "probe", "tcp://nas:22", "--kind", "tcp"])
            .unwrap();
    let Some(Command::Probe { target, kind }) = line.command else {
        panic!("not a probe");
    };
    assert_eq!(target, "tcp://nas:22");
    assert_eq!(kind.map(ProbeKind::from), Some(ProbeKind::Tcp));
}

#[test]
fn proxy_render_is_the_only_proxy_action() {
    let line = CommandLine::try_parse_from(["home-portal", "proxy", "render"]).unwrap();
    assert_eq!(
        line.command,
        Some(Command::Proxy {
            action: ProxyAction::Render
        })
    );
    assert!(CommandLine::try_parse_from(["home-portal", "proxy", "show"]).is_err());
}
