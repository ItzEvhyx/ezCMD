//! Acceptance gates for the focused recommendation engine.  The checked-in
//! golden examples cover user-facing rendering; this corpus supplements them
//! with two stable trigger phrases per data-defined recipe across all OSes.

use ezcmd::content::{all_recipes, all_tools};
use ezcmd::context::Context;
use ezcmd::matcher::tokenize;
use ezcmd::model::RecipeKind;
use ezcmd::os::Os;
use ezcmd::recommend::{self, Decision};
use ezcmd::render::{self, RenderCtx};
use ezcmd::shell::Shell;
use serde::Deserialize;
use std::time::{Duration, Instant};

const MIN_EVALUATIONS: usize = 150;
const TOP_1_TARGET: f32 = 90.0;
const TOP_3_TARGET: f32 = 98.0;
const P95_BUDGET: Duration = Duration::from_millis(15);

#[derive(Deserialize)]
struct GoldenSuite {
    case: Vec<GoldenCase>,
}

#[derive(Deserialize)]
struct GoldenCase {
    query: String,
    os: String,
    expect_recipe: String,
}

#[derive(Clone)]
struct AccuracyCase {
    query: String,
    os: Os,
    expected: String,
}

fn context() -> Context {
    Context {
        git_repo: false,
        has_remote_origin: false,
        package_manager: "npm",
        ecosystems: vec![],
    }
}

fn os(value: &str) -> Os {
    match value {
        "linux" => Os::Linux,
        "macos" => Os::Macos,
        "windows" => Os::Windows,
        other => panic!("unknown OS {other:?}"),
    }
}

fn corpus() -> Vec<AccuracyCase> {
    let golden: GoldenSuite =
        toml::from_str(include_str!("golden.toml")).expect("valid focused golden TOML");
    let mut cases: Vec<AccuracyCase> = golden
        .case
        .into_iter()
        .map(|case| AccuracyCase {
            query: case.query,
            os: os(&case.os),
            expected: case.expect_recipe,
        })
        .collect();

    // The recipe's first two triggers are intentionally the most stable,
    // data-authored ways to ask for that task.  Running them on each supported
    // OS catches ranking regressions and confirms platform filtering.
    for recipe in all_recipes() {
        let command = recipe.base.or(recipe.primary);
        for trigger in recipe.triggers.iter().take(2) {
            for os in Os::ALL {
                if command.is_some_and(|command| command.resolve_for(os, Shell::Cmd).is_some()) {
                    cases.push(AccuracyCase {
                        query: (*trigger).to_string(),
                        os,
                        expected: recipe.id.to_string(),
                    });
                }
            }
        }
    }
    cases
}

fn candidates(decision: Decision) -> Vec<String> {
    match decision {
        Decision::Clear(candidate) => vec![candidate.candidate.id()],
        Decision::Ambiguous(candidates) | Decision::NoMatch(candidates) => candidates
            .into_iter()
            .map(|candidate| candidate.candidate.id())
            .collect(),
        Decision::Blocked(_) => vec![],
    }
}

fn clear_recipe(query: &str) -> (&'static ezcmd::model::Recipe, ezcmd::recommend::Frame) {
    let (frame, decision) = recommend::decide(query, Os::Linux, Shell::Cmd, &context());
    match decision {
        Decision::Clear(candidate) => match candidate.candidate {
            ezcmd::recommend::Candidate::Recipe(recipe) => (recipe, frame),
            ezcmd::recommend::Candidate::LegacyCommand { .. } => {
                panic!("{query:?} selected a catalog command instead of a focused recipe")
            }
            _ => panic!("{query:?} selected a workflow instead of a focused recipe"),
        },
        _ => panic!("{query:?} did not produce a clear focused recommendation"),
    }
}

fn plain_ctx() -> RenderCtx {
    RenderCtx {
        color: false,
        plain: true,
        width: 100,
        shell: Shell::Cmd,
    }
}

#[test]
fn recommendation_corpus_meets_accuracy_targets() {
    let cases = corpus();
    assert!(
        cases.len() >= MIN_EVALUATIONS,
        "recommendation corpus has {} evaluations; need at least {MIN_EVALUATIONS}",
        cases.len()
    );

    let mut top_1 = 0usize;
    let mut top_3 = 0usize;
    let mut failures = Vec::new();
    for case in &cases {
        let (_, decision) = recommend::decide(&case.query, case.os, Shell::Cmd, &context());
        let actual = candidates(decision);
        let first_matches = actual.first().is_some_and(|id| id == &case.expected);
        let three_match = actual.iter().take(3).any(|id| id == &case.expected);
        top_1 += usize::from(first_matches);
        top_3 += usize::from(three_match);
        if !three_match {
            failures.push(format!(
                "{:?} query {:?}: expected {}, got {:?}",
                case.os, case.query, case.expected, actual
            ));
        }
    }

    let total = cases.len() as f32;
    let top_1_rate = top_1 as f32 / total * 100.0;
    let top_3_rate = top_3 as f32 / total * 100.0;
    println!(
        "focused recommendation accuracy: {} evaluations; top-1 {top_1}/{}, ({top_1_rate:.2}%); top-3 {top_3}/{}, ({top_3_rate:.2}%)",
        cases.len(), cases.len(), cases.len()
    );
    assert!(
        top_1_rate >= TOP_1_TARGET,
        "top-1 rate {top_1_rate:.2}% is below {TOP_1_TARGET}%\n{}",
        failures.join("\n")
    );
    assert!(
        top_3_rate >= TOP_3_TARGET,
        "top-3 rate {top_3_rate:.2}% is below {TOP_3_TARGET}%\n{}",
        failures.join("\n")
    );
}

#[test]
fn every_catalog_section_resolves_to_a_focused_answer() {
    let mut total = 0usize;
    let mut failures = Vec::new();
    for tool in all_tools() {
        for os in Os::ALL {
            if !tool.available_for(os, Shell::Cmd) {
                continue;
            }
            for section in tool
                .sections()
                .filter(|section| section.available_for(os, Shell::Cmd))
            {
                total += 1;
                let alias = section.aliases[0];
                // Most aliases already include their tool (`docker logs`,
                // `npm audit`). Prefix only aliases that genuinely omit it,
                // so this audit exercises realistic user queries rather than
                // invented `npm npm audit` forms that look like raw commands.
                let query = if tokenize(alias).iter().any(|token| token == tool.slug) {
                    alias.to_string()
                } else {
                    format!("{} {alias}", tool.slug)
                };
                let (_, decision) = recommend::decide(&query, os, Shell::Cmd, &context());
                let focused = matches!(
                    decision,
                    Decision::Clear(candidate)
                        if matches!(candidate.candidate,
                            ezcmd::recommend::Candidate::Recipe(_)
                                | ezcmd::recommend::Candidate::LegacyCommand { .. }
                                | ezcmd::recommend::Candidate::Workflow { .. }
                        )
                );
                if !focused {
                    failures.push(format!(
                        "{os:?} {query:?} from {}: {}",
                        tool.slug, section.title
                    ));
                }
            }
        }
    }
    println!(
        "catalog focused-answer coverage: {}/{} sections",
        total - failures.len(),
        total
    );
    assert!(
        failures.is_empty(),
        "catalog aliases did not resolve to one focused command:\n{}",
        failures.join("\n")
    );
}

#[test]
fn every_raw_catalog_command_resolves_to_a_focused_answer() {
    let mut total = 0usize;
    let mut failures = Vec::new();
    for tool in all_tools() {
        for os in Os::ALL {
            if !tool.available_for(os, Shell::Cmd) {
                continue;
            }
            for section in tool
                .sections()
                .filter(|section| section.available_for(os, Shell::Cmd))
            {
                for command in section.visible_commands_for(os, Shell::Cmd) {
                    let query = command
                        .resolve_for(os, Shell::Cmd)
                        .expect("visible command resolves for the audited target");
                    let tokens = tokenize(query);
                    let starts_with_tool = tokens.first().is_some_and(|first| {
                        first == tool.slug
                            || tool
                                .aliases
                                .iter()
                                .flat_map(|alias| tokenize(alias))
                                .any(|alias| alias == *first)
                            || tokenize(tool.name).iter().any(|name| name == first)
                    });
                    // Single-key shortcuts and commands whose executable is
                    // not the catalog tool (for example, `node -v` in the
                    // npm section) are not raw `<tool> <subcommand>` forms.
                    if tokens.len() < 2 || !starts_with_tool {
                        continue;
                    }
                    total += 1;
                    let (_, decision) = recommend::decide(query, os, Shell::Cmd, &context());
                    let focused = matches!(
                        decision,
                        Decision::Clear(candidate)
                            if matches!(candidate.candidate,
                                ezcmd::recommend::Candidate::Recipe(_)
                                    | ezcmd::recommend::Candidate::LegacyCommand { .. }
                                    | ezcmd::recommend::Candidate::Workflow { .. }
                            )
                    );
                    if !focused {
                        failures.push(format!(
                            "{os:?} {query:?} from {}: {}",
                            tool.slug, section.title
                        ));
                    }
                }
            }
        }
    }
    println!(
        "catalog raw-command focused-answer coverage: {}/{} commands",
        total - failures.len(),
        total
    );
    assert!(
        failures.is_empty(),
        "catalog raw command text did not resolve to one focused answer:\n{}",
        failures.join("\n")
    );
}

#[test]
fn recommendation_decision_p95_is_under_budget_in_release() {
    if cfg!(debug_assertions) {
        eprintln!("skipping release-only recommendation latency gate in a debug test build");
        return;
    }
    let cases = corpus();
    let mut durations = Vec::with_capacity(cases.len());
    for case in &cases {
        let started = Instant::now();
        let _ = recommend::decide(&case.query, case.os, Shell::Cmd, &context());
        durations.push((started.elapsed(), &case.query));
    }
    durations.sort_unstable_by_key(|(duration, _)| *duration);
    let p95_index = (durations.len() * 95).div_ceil(100).saturating_sub(1);
    let p95 = durations[p95_index].0;
    println!(
        "focused recommendation p95: {:.3} ms (budget: {:.3} ms)",
        p95.as_secs_f64() * 1_000.0,
        P95_BUDGET.as_secs_f64() * 1_000.0
    );
    assert!(
        p95 < P95_BUDGET,
        "focused recommendation p95 {p95:?} exceeds {P95_BUDGET:?}; slowest samples: {:?}",
        durations
            .iter()
            .rev()
            .take(5)
            .map(|(duration, query)| format!("{query:?}={duration:?}"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn every_single_recipe_and_variant_has_a_focused_acceptance_case() {
    for recipe in all_recipes()
        .iter()
        .filter(|recipe| recipe.kind == RecipeKind::Single)
    {
        let query = recipe.triggers[0];
        let (selected, frame) = clear_recipe(query);
        assert_eq!(selected.id, recipe.id, "primary query {query:?}");

        let primary = recipe
            .primary
            .and_then(|command| command.resolve_for(Os::Linux, Shell::Cmd))
            .expect("single recipe has a Linux primary command");
        let mut output = Vec::new();
        render::render_recipe(
            &mut output,
            recipe,
            &frame,
            &context(),
            Os::Linux,
            &plain_ctx(),
            true,
        )
        .expect("render primary");
        let output = String::from_utf8(output).expect("UTF-8 output");
        assert!(
            output.contains(primary),
            "{} primary command missing from:\n{}",
            recipe.id,
            output
        );

        assert!(
            recipe.variants.len() <= 3,
            "{} exceeds the three-variant output cap",
            recipe.id
        );
        for variant in recipe
            .variants
            .iter()
            .filter(|variant| !variant.matches.is_empty())
        {
            let query = format!("{} {}", recipe.triggers[0], variant.matches[0]);
            let (selected, frame) = clear_recipe(&query);
            assert_eq!(selected.id, recipe.id, "promoted query {query:?}");
            let command = variant
                .resolve_for(Os::Linux, Shell::Cmd)
                .expect("variant has a Linux command");
            let mut output = Vec::new();
            render::render_recipe(
                &mut output,
                recipe,
                &frame,
                &context(),
                Os::Linux,
                &plain_ctx(),
                true,
            )
            .expect("render promoted variant");
            let output = String::from_utf8(output).expect("UTF-8 output");
            assert!(
                output.contains(command),
                "{} promoted command missing for {query:?}:\n{}",
                recipe.id,
                output
            );
        }
    }
}

#[test]
fn ambiguity_and_negative_queries_never_guess() {
    let (_, ambiguous) = recommend::decide("undo", Os::Linux, Shell::Cmd, &context());
    assert!(matches!(ambiguous, Decision::Ambiguous(_)));

    for query in ["make me a sandwich", "quantum banana telescope"] {
        let (_, decision) = recommend::decide(query, Os::Linux, Shell::Cmd, &context());
        assert!(matches!(decision, Decision::NoMatch(_)), "{query:?}");
    }
}
