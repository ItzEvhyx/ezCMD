use ezcmd::content::all_tools;
use ezcmd::matcher::{search, SearchResult};
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

#[test]
fn test_query_accuracy() {
    let queries_toml = include_str!("queries.toml");
    let suite: QuerySuite = toml::from_str(queries_toml).expect("failed to parse queries.toml");

    let mut total_runs = 0;
    let mut top1_hits = 0;
    let mut top3_hits = 0;
    let mut failures = Vec::new();

    for case in &suite.q {
        let targets: Vec<Os> = match case.os.as_deref() {
            Some("windows") => vec![Os::Windows],
            Some("linux") => vec![Os::Linux],
            Some("macos") => vec![Os::Macos],
            Some("unix") => vec![Os::Linux, Os::Macos],
            _ => vec![Os::Windows, Os::Linux, Os::Macos],
        };

        for os in targets {
            total_runs += 1;
            let result = search(all_tools(), os, &case.query, None);

            let mut is_top1 = false;
            let mut is_top3 = false;

            match &result {
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
                let actual = match &result {
                    SearchResult::Single(h) => {
                        format!("Single(\"{}\", score={:.2})", h.section.title, h.score)
                    }
                    SearchResult::Ambiguous(hits) => {
                        let titles: Vec<String> = hits
                            .iter()
                            .map(|h| format!("\"{}\" ({:.2})", h.section.title, h.score))
                            .collect();
                        format!("Ambiguous([{}])", titles.join(", "))
                    }
                    SearchResult::NoMatch(hits) => {
                        let titles: Vec<String> = hits
                            .iter()
                            .map(|h| format!("\"{}\" ({:.2})", h.section.title, h.score))
                            .collect();
                        format!("NoMatch([{}])", titles.join(", "))
                    }
                };
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
