use crate::model::{Section, Tool};
use crate::os::Os;

pub const W_TITLE: f32 = 1.00;
pub const W_ALIAS: f32 = 0.95;
pub const W_CMD: f32 = 0.70;
pub const W_NOTE: f32 = 0.50;
pub const MATCH_THRESHOLD: f32 = 0.45; // below this: not a match
pub const AMBIGUITY_RATIO: f32 = 0.85; // candidates within 85% of the top score
pub const MAX_CHOICES: usize = 5;
pub const OTHER_TOOL_FACTOR: f32 = 0.55;
pub const TOOL_BONUS: f32 = 0.10;
pub const HINT_BONUS: f32 = 0.05;

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOpts<'a> {
    pub only_tool: Option<&'a str>,
    /// Interpret exact tool slugs, flag aliases, and shadowed aliases in a query.
    /// Help section selectors intentionally leave this off.
    pub tool_tokens: bool,
}

const STOPWORDS: &[&str] = &[
    "a", "an", "the", "to", "how", "do", "i", "my", "in", "on", "of", "for", "with", "and",
];

#[derive(Debug, Clone)]
pub struct Hit<'a> {
    pub tool: &'a Tool,
    pub section: &'a Section,
    pub score: f32,
    pub section_idx: usize,
}

impl<'a> PartialEq for Hit<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.tool.name == other.tool.name
            && self.section.title == other.section.title
            && (self.score - other.score).abs() < 1e-4
    }
}

#[derive(Debug, PartialEq)]
pub enum SearchResult<'a> {
    ToolOnly(&'a Tool),
    Single(Hit<'a>),
    Ambiguous(Vec<Hit<'a>>), // 2..=MAX_CHOICES
    NoMatch(Vec<Hit<'a>>),   // up to MAX_CHOICES closest, may be empty
}

/// Light stemmer: strip first matching suffix from ing, ed, es, s, e
/// only if at least 3 characters remain.
pub fn stem(token: &str) -> &str {
    const SUFFIXES: &[&str] = &["ing", "ed", "es", "s", "e"];
    for &suffix in SUFFIXES {
        if let Some(prefix) = token.strip_suffix(suffix) {
            if prefix.chars().count() >= 3 {
                return prefix;
            }
        }
    }
    token
}

/// Lowercase, split on any non-alphanumeric character, drop tokens shorter than 2 chars.
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter_map(|raw| {
            let s = raw.trim_start_matches('-').to_lowercase();
            if s.chars().count() >= 2 {
                Some(s)
            } else {
                None
            }
        })
        .collect()
}

/// Tokenize query with stopword removal. If that empties the query, retry without removal.
pub fn tokenize_query(query: &str) -> Vec<String> {
    let all_tokens = tokenize(query);
    let filtered: Vec<String> = all_tokens
        .iter()
        .filter(|t| !STOPWORDS.contains(&t.as_str()))
        .cloned()
        .collect();

    if !filtered.is_empty() {
        filtered
    } else {
        all_tokens
    }
}

/// Token match score between query token `q` and content token `t`.
pub fn token_match_score(q: &str, t: &str) -> f32 {
    let stem_q = stem(q);
    let stem_t = stem(t);

    // 1. Stems equal
    if stem_q == stem_t {
        return 1.00;
    }

    let q_len = q.chars().count();
    let t_len = t.chars().count();

    // 2. `t` starts with `q` (`q.len() >= 3`), e.g. `repo` vs `repository`
    if q_len >= 3 && t.starts_with(q) {
        return 0.85;
    }

    // 3. `q` starts with `t` (`t.len() >= 3`), e.g. `initialize` vs `init`
    if t_len >= 3 && q.starts_with(t) {
        return 0.80;
    }

    // 4. Damerau-Levenshtein distance ≤ 1 (length 4..=7) or ≤ 2 (length ≥ 8)
    let dist = strsim::damerau_levenshtein(q, t).min(strsim::damerau_levenshtein(stem_q, stem_t));
    if ((4..=7).contains(&q_len) && dist <= 1) || (q_len >= 8 && dist <= 2) {
        return 0.70;
    }

    0.00
}

/// Field score = (sum over query tokens of best tm against any field token) / (num query tokens).
fn field_score(query_tokens: &[String], field_tokens: &[String]) -> f32 {
    if query_tokens.is_empty() || field_tokens.is_empty() {
        return 0.0;
    }

    let mut sum = 0.0;
    for q in query_tokens {
        let mut best_tm = 0.0f32;
        for t in field_tokens {
            let score = token_match_score(q, t);
            if score > best_tm {
                best_tm = score;
            }
        }
        sum += best_tm;
    }

    sum / query_tokens.len() as f32
}

fn score_section(section: &Section, os: Os, query_tokens: &[String]) -> f32 {
    let title_tokens = tokenize(section.title);
    let title_score = field_score(query_tokens, &title_tokens);

    let best_alias_score = section
        .aliases
        .iter()
        .map(|alias| field_score(query_tokens, &tokenize(alias)))
        .fold(0.0, f32::max);

    let joined_cmds = section
        .commands
        .iter()
        .filter_map(|command| command.resolve(os))
        .collect::<Vec<_>>()
        .join(" ");
    let cmd_score = field_score(query_tokens, &tokenize(&joined_cmds));

    let joined_notes = section
        .commands
        .iter()
        .map(|command| command.note)
        .collect::<Vec<_>>()
        .join(" ");
    let note_score = field_score(query_tokens, &tokenize(&joined_notes));

    (W_TITLE * title_score)
        .max(W_ALIAS * best_alias_score)
        .max(W_CMD * cmd_score)
        .max(W_NOTE * note_score)
}

fn has_token(values: &[&str], token: &str) -> bool {
    values
        .iter()
        .flat_map(|value| tokenize(value))
        .any(|value| value == token)
}

fn is_named_tool_token(tool: &Tool, token: &str) -> bool {
    tool.slug.eq_ignore_ascii_case(token) || has_token(tool.flag_aliases, token)
}

fn is_hint_tool_token(tool: &Tool, token: &str) -> bool {
    let shadowed_alias = tool
        .aliases
        .iter()
        .filter(|alias| {
            !tool
                .flag_aliases
                .iter()
                .any(|flag| flag.eq_ignore_ascii_case(alias))
        })
        .flat_map(|alias| tokenize(alias))
        .any(|alias| alias == token);
    shadowed_alias || has_token(&[tool.name], token)
}

fn tool_has_scoring_section(tool: &Tool, os: Os, tokens: &[String]) -> bool {
    tool.sections()
        .filter(|section| section.available_on(os))
        .any(|section| score_section(section, os, tokens) >= MATCH_THRESHOLD)
}

/// Search sections across tools for the given OS.
pub fn search<'a>(
    tools: impl IntoIterator<Item = &'a Tool>,
    os: Os,
    query: &str,
    opts: SearchOpts<'_>,
) -> SearchResult<'a> {
    let q_tokens = tokenize_query(query);
    if q_tokens.is_empty() {
        return SearchResult::NoMatch(Vec::new());
    }

    let tools: Vec<&Tool> = tools
        .into_iter()
        .filter(|tool| {
            opts.only_tool
                .is_none_or(|target| tool.slug.eq_ignore_ascii_case(target))
        })
        .collect();

    let mut effective_tokens = q_tokens.clone();
    let mut named_tools: Vec<&Tool> = Vec::new();
    let mut named_tokens = Vec::new();
    let mut named_narrowing = false;
    let mut hint_tools: Vec<&Tool> = Vec::new();
    let mut hint_narrowing = false;

    if opts.tool_tokens {
        for token in &q_tokens {
            let matches: Vec<&Tool> = tools
                .iter()
                .copied()
                .filter(|tool| is_named_tool_token(tool, token))
                .collect();
            if !matches.is_empty() {
                named_tokens.push(token.as_str());
                for tool in matches {
                    if !named_tools
                        .iter()
                        .any(|candidate| candidate.slug == tool.slug)
                    {
                        named_tools.push(tool);
                    }
                }
            }
        }

        // A bare, non-shadowed tool token opens that tool's table of contents.
        if named_tokens.len() == q_tokens.len() {
            if named_tools.len() == 1 {
                return SearchResult::ToolOnly(named_tools[0]);
            }
            let name_matches: Vec<&Tool> = named_tools
                .iter()
                .copied()
                .filter(|tool| {
                    let name_tokens = tokenize(tool.name);
                    q_tokens.iter().all(|token| name_tokens.contains(token))
                })
                .collect();
            if name_matches.len() == 1 {
                return SearchResult::ToolOnly(name_matches[0]);
            }
        }

        let without_named: Vec<String> = q_tokens
            .iter()
            .filter(|token| !named_tokens.contains(&token.as_str()))
            .cloned()
            .collect();
        if !without_named.is_empty()
            && named_tools
                .iter()
                .any(|tool| tool_has_scoring_section(tool, os, &without_named))
        {
            effective_tokens = without_named;
            named_narrowing = true;
        }

        let mut hint_tokens = Vec::new();
        for token in &effective_tokens {
            let matches: Vec<&Tool> = tools
                .iter()
                .copied()
                .filter(|tool| is_hint_tool_token(tool, token))
                .collect();
            if !matches.is_empty() {
                hint_tokens.push(token.as_str());
                for tool in matches {
                    if !hint_tools
                        .iter()
                        .any(|candidate| candidate.slug == tool.slug)
                    {
                        hint_tools.push(tool);
                    }
                }
            }
        }
        let without_hints: Vec<String> = effective_tokens
            .iter()
            .filter(|token| !hint_tokens.contains(&token.as_str()))
            .cloned()
            .collect();
        if !without_hints.is_empty()
            && hint_tools
                .iter()
                .any(|tool| tool_has_scoring_section(tool, os, &without_hints))
        {
            effective_tokens = without_hints;
            hint_narrowing = true;
        }
    }

    let mut hits = Vec::new();

    for tool in tools {
        for (section_idx, section) in tool.sections().enumerate() {
            if !section.available_on(os) {
                continue;
            }

            let mut score = score_section(section, os, &effective_tokens);
            let is_named_tool = named_tools
                .iter()
                .any(|candidate| candidate.slug == tool.slug);
            let is_hint_tool = hint_tools
                .iter()
                .any(|candidate| candidate.slug == tool.slug);
            if named_narrowing {
                if is_named_tool {
                    score += TOOL_BONUS;
                } else {
                    score *= OTHER_TOOL_FACTOR;
                }
            }
            if hint_narrowing && is_hint_tool {
                score += HINT_BONUS;
            }

            hits.push(Hit {
                tool,
                section,
                score,
                section_idx,
            });
        }
    }

    if hits.is_empty() {
        return SearchResult::NoMatch(Vec::new());
    }

    // Sort by score descending; ties broken by tool name then section_idx
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.tool.slug.cmp(b.tool.slug))
            .then_with(|| a.section_idx.cmp(&b.section_idx))
    });

    let top_score = hits[0].score;
    if top_score < MATCH_THRESHOLD {
        hits.truncate(MAX_CHOICES);
        return SearchResult::NoMatch(hits);
    }

    let ambiguity_threshold = top_score * AMBIGUITY_RATIO;
    let mut matching_hits: Vec<Hit<'a>> = hits
        .into_iter()
        .filter(|h| h.score >= ambiguity_threshold)
        .collect();

    if matching_hits.len() == 1 {
        // Provably guaranteed: matching_hits.len() == 1
        SearchResult::Single(matching_hits.remove(0))
    } else {
        matching_hits.truncate(MAX_CHOICES);
        SearchResult::Ambiguous(matching_hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::all_tools;

    #[test]
    fn test_stemmer() {
        assert_eq!(stem("initializing"), "initializ");
        assert_eq!(stem("initialize"), "initializ");
        assert_eq!(stem("branches"), "branch");
        assert_eq!(stem("commits"), "commit");
        assert_eq!(stem("files"), "fil");
        assert_eq!(stem("file"), "fil");
        assert_eq!(stem("the"), "the");
        assert_eq!(stem("red"), "red");
        assert_eq!(stem("yes"), "yes");
    }

    #[test]
    fn test_tokenize_and_stopwords() {
        let tokens = tokenize("git reset --hard");
        assert_eq!(tokens, vec!["git", "reset", "hard"]);

        let q_tokens = tokenize_query("how to initialize a repo");
        assert_eq!(q_tokens, vec!["initialize", "repo"]);

        let fallback_tokens = tokenize_query("how to");
        assert_eq!(fallback_tokens, vec!["how", "to"]);
    }

    #[test]
    fn test_token_match_score() {
        // Equal stems
        assert_eq!(token_match_score("initializing", "initialize"), 1.00);
        // Prefix matching
        assert_eq!(token_match_score("repo", "repository"), 0.85);
        assert_eq!(token_match_score("initialize", "init"), 0.80);
        // Typo tolerance
        assert_eq!(token_match_score("initalize", "initialize"), 0.70);
        // No match
        assert_eq!(token_match_score("git", "docker"), 0.00);
    }

    #[test]
    fn bare_tool_token_returns_tool_only() {
        let result = search(
            all_tools(),
            Os::Linux,
            "git",
            SearchOpts {
                tool_tokens: true,
                ..Default::default()
            },
        );
        assert!(matches!(result, SearchResult::ToolOnly(tool) if tool.slug == "git"));
    }

    #[test]
    fn named_tool_token_narrows_only_when_remaining_words_match() {
        let result = search(
            all_tools(),
            Os::Linux,
            "git initialize repo",
            SearchOpts {
                tool_tokens: true,
                ..Default::default()
            },
        );
        assert!(matches!(result, SearchResult::Single(hit)
            if hit.tool.slug == "git" && hit.section.title == "Initializing a Repo"));
    }

    #[test]
    fn help_search_does_not_interpret_tool_tokens() {
        let result = search(all_tools(), Os::Linux, "git", SearchOpts::default());
        assert!(!matches!(result, SearchResult::ToolOnly(_)));
    }

    #[test]
    fn shadowed_aliases_are_hints_not_bare_tool_tokens() {
        let tool = Tool {
            slug: "example",
            name: "Example Tool",
            category: "Test",
            aliases: &["shared"],
            flag_aliases: &[],
            platforms: &[],
            notes: &[],
            groups: &[],
        };

        assert!(is_hint_tool_token(&tool, "shared"));
        assert!(!is_named_tool_token(&tool, "shared"));
        assert_eq!(MAX_CHOICES, 5);
    }
}
