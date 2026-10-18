use ezcmd::content::all_tools;
use ezcmd::matcher::{search, SearchOpts, SearchResult};
use ezcmd::os::Os;
use serde::Deserialize;

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
    os: Option<String>,
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

    for case in &suite.q {
        for os in targets(case.os.as_deref()) {
            total += 1;
            let result = search_query(&case.query, os);
            let expected_tool = case.tool.as_deref();
            let expected_section = case.expect.as_deref();
            let pass = match (case.kind.as_str(), &result) {
                ("tool", SearchResult::ToolOnly(tool)) => expected_tool == Some(tool.slug),
                ("task" | "guard", SearchResult::Single(hit)) => {
                    expected_tool.is_none_or(|tool| hit.tool.slug == tool)
                        && expected_section == Some(hit.section.title)
                }
                ("task", SearchResult::Ambiguous(hits)) => hits.iter().any(|hit| {
                    expected_tool.is_none_or(|tool| hit.tool.slug == tool)
                        && expected_section == Some(hit.section.title)
                }),
                ("none", SearchResult::NoMatch(_)) => true,
                _ => false,
            };
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

    assert_eq!(suite.q.len(), 30, "V6 seed corpus must contain 30 queries");
    assert!(
        failures.is_empty(),
        "V2 seed failures:\n{}",
        failures.join("\n")
    );
    println!("V2 seed accuracy: {total} evaluations passed");
}
