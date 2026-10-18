use crate::model::{Section, Tool};
use crate::os::Os;

pub const W_TITLE: f32 = 1.00;
pub const W_ALIAS: f32 = 0.95;
pub const W_CMD: f32 = 0.70;
pub const W_NOTE: f32 = 0.50;
pub const MATCH_THRESHOLD: f32 = 0.45; // below this: not a match
pub const AMBIGUITY_RATIO: f32 = 0.85; // candidates within 85% of the top score
pub const MAX_CHOICES: usize = 3;

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
    Single(Hit<'a>),
    Ambiguous(Vec<Hit<'a>>), // 2..=MAX_CHOICES
    NoMatch(Vec<Hit<'a>>),   // up to 3 closest, may be empty
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

/// Search sections across tools for the given OS.
pub fn search<'a>(
    tools: impl IntoIterator<Item = &'a Tool>,
    os: Os,
    query: &str,
    only_tool: Option<&str>,
) -> SearchResult<'a> {
    let q_tokens = tokenize_query(query);
    if q_tokens.is_empty() {
        return SearchResult::NoMatch(Vec::new());
    }

    let mut hits = Vec::new();

    for tool in tools {
        if let Some(target) = only_tool {
            if !tool.slug.eq_ignore_ascii_case(target) {
                continue;
            }
        }

        for (section_idx, section) in tool.sections().enumerate() {
            if !section.available_on(os) {
                continue;
            }

            // 1. Title
            let title_tokens = tokenize(section.title);
            let title_score = field_score(&q_tokens, &title_tokens);

            // 2. Each alias separately (take max)
            let mut best_alias_score = 0.0f32;
            for alias in section.aliases {
                let alias_tokens = tokenize(alias);
                let a_score = field_score(&q_tokens, &alias_tokens);
                if a_score > best_alias_score {
                    best_alias_score = a_score;
                }
            }

            // 3. Command text (resolved for OS only)
            let resolved_cmds: Vec<&str> = section
                .commands
                .iter()
                .filter_map(|c| c.resolve(os))
                .collect();
            let joined_cmds = resolved_cmds.join(" ");
            let cmd_tokens = tokenize(&joined_cmds);
            let cmd_score = field_score(&q_tokens, &cmd_tokens);

            // 4. Notes
            let joined_notes = section
                .commands
                .iter()
                .map(|c| c.note)
                .collect::<Vec<_>>()
                .join(" ");
            let note_tokens = tokenize(&joined_notes);
            let note_score = field_score(&q_tokens, &note_tokens);

            let score = (W_TITLE * title_score)
                .max(W_ALIAS * best_alias_score)
                .max(W_CMD * cmd_score)
                .max(W_NOTE * note_score);

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
}
