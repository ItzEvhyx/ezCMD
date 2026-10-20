use ezcmd::context::Context;
use ezcmd::os::Os;
use ezcmd::recommend::{self, Candidate, Decision};
use ezcmd::render::{self, RenderCtx};
use ezcmd::shell::Shell;
use serde::Deserialize;

#[derive(Deserialize)]
struct GoldenSuite {
    case: Vec<GoldenCase>,
}

#[derive(Deserialize)]
struct GoldenCase {
    query: String,
    os: String,
    #[serde(default)]
    shell: Option<String>,
    expect_recipe: String,
    #[serde(default)]
    expect_ambiguous: bool,
    expect_contains: Vec<String>,
    expect_absent: Vec<String>,
}

#[test]
fn golden_recommendations_render_the_expected_answer() {
    let suite: GoldenSuite = toml::from_str(include_str!("golden.toml")).expect("golden TOML");
    let context = Context {
        git_repo: false,
        has_remote_origin: false,
        package_manager: "npm",
        ecosystems: vec![],
    };
    for case in suite.case {
        let os = match case.os.as_str() {
            "linux" => Os::Linux,
            "macos" => Os::Macos,
            "windows" => Os::Windows,
            other => panic!("unknown OS: {}", other),
        };
        let shell = match case.shell.as_deref() {
            None | Some("cmd") => Shell::Cmd,
            Some("powershell") => Shell::PowerShell,
            Some(other) => panic!("unknown shell: {other}"),
        };
        let (frame, decision) = recommend::decide(&case.query, os, shell, &context);
        if case.expect_ambiguous {
            let candidates = match decision {
                Decision::Ambiguous(candidates) => candidates,
                _ => panic!("{} did not produce a choice", case.query),
            };
            assert!(
                candidates
                    .iter()
                    .any(|candidate| candidate.candidate.id() == case.expect_recipe),
                "{} did not include {} as a choice",
                case.query,
                case.expect_recipe
            );
            continue;
        }
        let candidate = match decision {
            Decision::Clear(candidate) => candidate,
            _ => panic!("{} did not produce a clear answer", case.query),
        };
        assert_eq!(
            candidate.candidate.id(),
            case.expect_recipe,
            "{}",
            case.query
        );
        let mut output = Vec::new();
        let ctx = RenderCtx {
            color: false,
            plain: true,
            width: 100,
            shell,
        };
        match candidate.candidate {
            Candidate::Recipe(recipe) => {
                render::render_recipe(&mut output, recipe, &frame, &context, os, &ctx, false)
                    .expect("render recipe")
            }
            Candidate::LegacyCommand {
                tool,
                section,
                command,
            } => render::render_legacy_command(
                &mut output,
                tool,
                section,
                command,
                &frame,
                &context,
                os,
                &ctx,
            )
            .expect("render command"),
            Candidate::Workflow { tool, section } => {
                render::render_workflow(&mut output, tool, section, &frame, &context, os, &ctx)
                    .expect("render workflow")
            }
        }
        let output = String::from_utf8(output).expect("UTF-8 output");
        for expected in case.expect_contains {
            assert!(
                output.contains(&expected),
                "{} missing {:?}\n{}",
                case.query,
                expected,
                output
            );
        }
        for absent in case.expect_absent {
            assert!(
                !output.contains(&absent),
                "{} unexpectedly has {:?}\n{}",
                case.query,
                absent,
                output
            );
        }
    }
}
