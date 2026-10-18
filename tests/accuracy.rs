use ezcmd::content::all_tools;
use ezcmd::matcher::{search, SearchOpts, SearchResult};
use ezcmd::os::Os;
use ezcmd::shell::Shell;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Deserialize)]
struct QuerySuite {
    q: Vec<QueryCase>,
}

#[derive(Deserialize)]
struct QueryCase {
    query: String,
    expect: String,
    os: Option<String>,
}

#[derive(Deserialize)]
struct V2QuerySuite {
    q: Vec<V2QueryCase>,
}

#[derive(Deserialize)]
struct V2QueryCase {
    kind: String,
    query: String,
    tool: Option<String>,
    expect: Option<String>,
    choices: Option<Vec<String>>,
    os: Option<String>,
    shell: Option<String>,
}

fn targets(os: Option<&str>) -> Vec<Os> {
    match os {
        Some("windows") => vec![Os::Windows],
        Some("linux") => vec![Os::Linux],
        Some("macos") => vec![Os::Macos],
        Some("unix") => vec![Os::Linux, Os::Macos],
        _ => vec![Os::Windows, Os::Linux, Os::Macos],
    }
}

fn search_query(query: &str, os: Os) -> SearchResult<'static> {
    search(
        all_tools(),
        os,
        query,
        SearchOpts {
            tool_tokens: true,
            ..Default::default()
        },
    )
}

fn shell_for_query(shell: Option<&str>) -> Shell {
    match shell {
        None | Some("cmd") => Shell::Cmd,
        Some("powershell") => Shell::PowerShell,
        Some(value) => panic!("unknown query shell {value:?}"),
    }
}

fn search_v2_query(query: &str, os: Os, shell: Option<&str>) -> SearchResult<'static> {
    search(
        all_tools(),
        os,
        query,
        SearchOpts {
            tool_tokens: true,
            shell: shell_for_query(shell),
            ..Default::default()
        },
    )
}

fn result_description(result: &SearchResult<'_>) -> String {
    match result {
        SearchResult::ToolOnly(tool) => format!("ToolOnly(\"{}\")", tool.slug),
        SearchResult::Single(hit) => {
            format!(
                "Single(\"{}: {}\", score={:.2})",
                hit.tool.slug, hit.section.title, hit.score
            )
        }
        SearchResult::Ambiguous(hits) => {
            let titles: Vec<String> = hits
                .iter()
                .map(|hit| {
                    format!(
                        "\"{}: {}\" ({:.2})",
                        hit.tool.slug, hit.section.title, hit.score
                    )
                })
                .collect();
            format!("Ambiguous([{}])", titles.join(", "))
        }
        SearchResult::NoMatch(hits) => {
            let titles: Vec<String> = hits
                .iter()
                .map(|hit| {
                    format!(
                        "\"{}: {}\" ({:.2})",
                        hit.tool.slug, hit.section.title, hit.score
                    )
                })
                .collect();
            format!("NoMatch([{}])", titles.join(", "))
        }
    }
}

#[test]
fn test_query_accuracy() {
    let queries_toml = include_str!("queries.toml");
    let suite: QuerySuite = toml::from_str(queries_toml).expect("failed to parse queries.toml");

    let mut total_runs = 0;
    let mut top1_hits = 0;
    let mut top3_hits = 0;
    let mut failures = Vec::new();

    for case in &suite.q {
        for os in targets(case.os.as_deref()) {
            total_runs += 1;
            let result = search_query(&case.query, os);

            let mut is_top1 = false;
            let mut is_top3 = false;

            match &result {
                SearchResult::ToolOnly(_) => {}
                SearchResult::Single(hit) => {
                    if hit.section.title == case.expect {
                        is_top1 = true;
                        is_top3 = true;
                    }
                }
                SearchResult::Ambiguous(hits) => {
                    if !hits.is_empty() && hits[0].section.title == case.expect {
                        is_top1 = true;
                    }
                    if hits.iter().any(|h| h.section.title == case.expect) {
                        is_top3 = true;
                    }
                }
                SearchResult::NoMatch(hits) => {
                    if hits.iter().any(|h| h.section.title == case.expect) {
                        // In NoMatch, hits are below threshold so not considered top-3 match
                    }
                }
            }

            if is_top1 {
                top1_hits += 1;
            }
            if is_top3 {
                top3_hits += 1;
            } else {
                let actual = result_description(&result);
                failures.push(format!(
                    "Query: \"{}\" on {:?} | Expected: \"{}\" | Got: {}",
                    case.query, os, case.expect, actual
                ));
            }
        }
    }

    let top1_rate = (top1_hits as f32 / total_runs as f32) * 100.0;
    let top3_rate = (top3_hits as f32 / total_runs as f32) * 100.0;

    println!("\n================== ACCURACY REPORT ==================");
    println!("Total query evaluations: {}", total_runs);
    println!(
        "Top-1 Hits: {} / {} ({:.2}%)",
        top1_hits, total_runs, top1_rate
    );
    println!(
        "Top-3 Hits: {} / {} ({:.2}%)",
        top3_hits, total_runs, top3_rate
    );
    println!("=====================================================\n");

    if !failures.is_empty() {
        println!("Failures:");
        for f in &failures {
            println!("  - {}", f);
        }
    }

    // Enforce full PRD accuracy gates (Section 8.5)
    assert!(
        top1_rate >= 90.0,
        "Top-1 rate {:.2}% below expected 90.0% target",
        top1_rate
    );
    assert!(
        top3_rate >= 98.0,
        "Top-3 rate {:.2}% below expected 98.0% target",
        top3_rate
    );
}

#[test]
fn test_v2_seed_query_accuracy() {
    let suite: V2QuerySuite =
        toml::from_str(include_str!("queries_v2.toml")).expect("failed to parse queries_v2.toml");
    let mut failures = Vec::new();
    let mut total = 0;
    let mut totals_by_kind: BTreeMap<&str, usize> = BTreeMap::new();
    let mut passes_by_kind: BTreeMap<&str, usize> = BTreeMap::new();
    let mut queries_by_tool: HashMap<&str, usize> = HashMap::new();

    for case in &suite.q {
        if let Some(tool) = case.tool.as_deref() {
            *queries_by_tool.entry(tool).or_default() += 1;
        }
    }
    for tool in all_tools() {
        assert!(
            queries_by_tool.get(tool.slug).copied().unwrap_or_default() >= 5,
            "{} needs at least five v2 accuracy queries",
            tool.slug
        );
    }

    for case in &suite.q {
        for os in targets(case.os.as_deref()) {
            total += 1;
            let result = search_v2_query(&case.query, os, case.shell.as_deref());
            let expected_tool = case.tool.as_deref();
            let expected_section = case.expect.as_deref();
            let section_matches = |hit: &ezcmd::matcher::Hit<'_>| {
                expected_tool.is_none_or(|tool| hit.tool.slug == tool)
                    && expected_section == Some(hit.section.title)
            };
            let pass = match (case.kind.as_str(), &result) {
                ("tool", SearchResult::ToolOnly(tool)) => {
                    expected_section.is_none() && expected_tool == Some(tool.slug)
                }
                ("tool" | "guard", SearchResult::Single(hit)) => section_matches(hit),
                ("tool" | "guard", SearchResult::Ambiguous(hits)) => {
                    hits.first().is_some_and(section_matches)
                }
                ("task", SearchResult::Single(hit)) => section_matches(hit),
                ("task", SearchResult::Ambiguous(hits)) => hits.iter().take(3).any(section_matches),
                ("ambiguous", SearchResult::Ambiguous(hits)) => {
                    case.choices.as_ref().is_some_and(|choices| {
                        choices
                            .iter()
                            .all(|tool| hits.iter().any(|hit| hit.tool.slug == tool))
                    })
                }
                ("none", SearchResult::NoMatch(_)) => true,
                _ => false,
            };
            *totals_by_kind.entry(case.kind.as_str()).or_default() += 1;
            if pass {
                *passes_by_kind.entry(case.kind.as_str()).or_default() += 1;
            }
            if !pass {
                failures.push(format!(
                    "{} query \"{}\" on {:?}: {}",
                    case.kind,
                    case.query,
                    os,
                    result_description(&result)
                ));
            }
        }
    }

    assert!(
        suite.q.len() >= 30,
        "V2 corpus must not shrink below its V6 seed"
    );
    assert!(
        failures.is_empty(),
        "V2 accuracy failures:\n{}",
        failures.join("\n")
    );
    let rate = |kind: &str| {
        let total = totals_by_kind.get(kind).copied().unwrap_or_default();
        let passed = passes_by_kind.get(kind).copied().unwrap_or_default();
        assert!(total > 0, "V2 corpus has no {kind} queries");
        (passed as f32 / total as f32) * 100.0
    };
    let tool_rate = rate("tool");
    let task_rate = rate("task");
    let none_rate = rate("none");
    let guard_rate = rate("guard");
    assert!(
        tool_rate >= 95.0,
        "tool top-1 rate {tool_rate:.2}% is below 95%"
    );
    assert!(
        task_rate >= 90.0,
        "task top-3 rate {task_rate:.2}% is below 90%"
    );
    assert_eq!(none_rate, 100.0, "none rate must be 100%");
    assert_eq!(guard_rate, 100.0, "guard top-1 rate must be 100%");

    if totals_by_kind.contains_key("ambiguous") {
        let ambiguous_rate = rate("ambiguous");
        assert!(
            ambiguous_rate >= 90.0,
            "ambiguous chooser rate {ambiguous_rate:.2}% is below 90%"
        );
    }

    println!("V2 accuracy: {total} evaluations passed");
}
