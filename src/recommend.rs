use crate::content::{all_recipes, all_tools};
use crate::context::Context;
use crate::history::History;
use crate::matcher::{field_score, tokenize, tokenize_query};
use crate::model::{Command, Recipe, RecipeKind, Risk, Section, Tool};
use crate::os::Os;
use crate::shell::Shell;
use crate::tuning;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Frame {
    pub tokens: Vec<String>,
    pub tool: Option<String>,
    pub tool_explicit: bool,
    pub action: Option<&'static str>,
    pub object: Option<&'static str>,
    pub options: Vec<String>,
    pub slots: Vec<(String, String)>,
    pub denied_path: bool,
    /// A natural-language request aimed at a protected or whole-machine
    /// target (for example, "delete root" or "wipe my disk"). This is
    /// separate from `denied_path`: a literal denied path keeps the safe
    /// template placeholder, while this must never rank destructive catalog
    /// commands at all.
    pub protected_destructive_target: bool,
    pub leftover: Vec<String>,
}

#[derive(Clone, Copy)]
pub enum Candidate {
    Recipe(&'static Recipe),
    /// A single command selected from the established catalog. This prevents
    /// normal searches from falling back to a whole legacy section.
    LegacyCommand {
        tool: &'static Tool,
        section: &'static Section,
        command: &'static Command,
    },
    Workflow {
        tool: &'static Tool,
        section: &'static Section,
    },
}

impl Candidate {
    pub fn title(self) -> String {
        match self {
            Self::Recipe(recipe) => recipe.title.to_string(),
            Self::LegacyCommand { tool, section, .. } => {
                format!("{}: {}", tool.name, section.title)
            }
            Self::Workflow { tool, section } => format!("{}: {}", tool.name, section.title),
        }
    }
    pub fn id(self) -> String {
        match self {
            Self::Recipe(recipe) => recipe.id.to_string(),
            Self::LegacyCommand {
                tool,
                section,
                command,
            } => format!(
                "command.{}.{}.{}",
                tool.slug,
                identifier(section.title),
                identifier(
                    command
                        .cmd
                        .or(command.unix)
                        .or(command.windows)
                        .unwrap_or(command.note)
                )
            ),
            Self::Workflow { tool, section } => {
                format!("workflow.{}.{}", tool.slug, identifier(section.title))
            }
        }
    }
    pub fn danger_primary(self) -> bool {
        matches!(self, Self::Recipe(recipe) if recipe.primary.is_some_and(|command| command.risk == Risk::Danger))
    }
}

fn identifier(value: &str) -> String {
    let mut output = String::new();
    let mut previous_dash = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash && !output.is_empty() {
            output.push('-');
            previous_dash = true;
        }
    }
    output.trim_end_matches('-').to_string()
}

#[derive(Clone)]
pub struct ScoredCandidate {
    pub candidate: Candidate,
    pub score: f32,
    pub reasons: Vec<String>,
}

pub enum Decision {
    Clear(ScoredCandidate),
    Ambiguous(Vec<ScoredCandidate>),
    NoMatch(Vec<ScoredCandidate>),
    /// No command is returned for a protected destructive target. This avoids
    /// presenting an unrelated destructive command as a selectable answer.
    Blocked(&'static str),
}

const ACTIONS: &[(&str, &[&str])] = &[
    (
        "create",
        &[
            "make",
            "create",
            "new",
            "scaffold",
            "generate",
            "start",
            "init",
            "initialize",
            "setup",
        ],
    ),
    (
        "delete",
        &["remove", "delete", "rm", "erase", "drop", "wipe", "destroy"],
    ),
    ("commit", &["commit"]),
    ("undo", &["undo", "revert", "rollback", "uncommit"]),
    ("install", &["install", "add"]),
    ("restore", &["restore", "unstage", "pop", "apply"]),
    ("stash", &["stash", "shelve"]),
    ("run", &["run", "start", "test"]),
    ("copy", &["copy", "duplicate"]),
    ("move", &["move", "rename"]),
    ("push", &["push", "publish", "upload"]),
    ("list", &["list", "show", "view"]),
    ("build", &["build", "compile"]),
];

pub fn parse(query: &str) -> Frame {
    let tokens = tokenize_query(query);
    // `start a project` means scaffold, while `start a container` means run.
    // Treat project/repository targets as creation and every other `start`
    // request as execution; focused trigger matches can still refine this.
    let action = if tokens.iter().any(|token| token == "start") {
        if tokens
            .iter()
            .any(|token| matches!(token.as_str(), "project" | "repo" | "repository"))
        {
            Some("create")
        } else {
            Some("run")
        }
    } else {
        ACTIONS.iter().find_map(|(canonical, words)| {
            tokens
                .iter()
                .any(|token| words.contains(&token.as_str()))
                .then_some(*canonical)
        })
    };
    // A tool's own name is more reliable than an alias.  For example,
    // “docker image” must select Docker even if another tool has a broad
    // alias such as “image” or “build”.
    // Prefer an exact slug before display-name words. Several display names
    // share ordinary English words (for example, Bash is described as a
    // "shell"), so looking at names first can route `shell cd` to Bash just
    // because it appears earlier in the catalog.
    let tool = multiword_alias_tool(&tokens)
        .or_else(|| {
            tokens
                .iter()
                .find_map(|token| all_tools().find(|tool| token == tool.slug))
        })
        .or_else(|| {
            tokens.first().and_then(|token| {
                all_tools().find(|tool| {
                    tool.aliases.iter().any(|alias| {
                        let alias_tokens = tokenize(alias);
                        alias_tokens.len() == 1 && alias_tokens[0] == *token
                    })
                })
            })
        })
        .or_else(|| {
            tokens.iter().find_map(|token| {
                all_tools().find(|tool| tokenize(tool.name).iter().any(|name| name == token))
            })
        })
        .map(|tool| tool.slug.to_string());
    // A slug or declared alias is an explicit tool request.  Tool-name words
    // alone are weaker (for example, “install package” can match a package
    // manager's display name) and must not suppress unrelated candidates.
    let tool_explicit = tool.as_deref().is_some_and(|slug| {
        all_tools()
            .find(|tool| tool.slug == slug)
            .is_some_and(|tool| {
                tokens.iter().any(|token| {
                    token == tool.slug
                        || tool
                            .aliases
                            .iter()
                            .flat_map(|alias| tokenize(alias))
                            .any(|alias| alias == *token)
                })
            })
    });
    let object = if tokens
        .iter()
        .any(|token| matches!(token.as_str(), "project" | "app"))
    {
        Some("project")
    } else if tokens.iter().any(|token| token == "commit") {
        Some("commit")
    } else if tokens
        .iter()
        .any(|token| matches!(token.as_str(), "repo" | "repository"))
    {
        Some("repo")
    } else if tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "package" | "packages" | "dependency" | "dependencies"
        )
    }) {
        Some("package")
    } else if tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "folder" | "folders" | "directory" | "directories" | "file" | "files"
        )
    }) {
        Some("file")
    } else if tokens.iter().any(|token| token == "branch") {
        Some("branch")
    } else if tokens.iter().any(|token| token == "stash") {
        Some("stash")
    } else {
        None
    };
    let mut ordered_options = Vec::new();
    for recipe in all_recipes() {
        for option in recipe.options {
            if let Some(position) = option
                .matches
                .iter()
                .filter_map(|term| query_position(query, term))
                .min()
            {
                if !ordered_options
                    .iter()
                    .any(|(_, id): &(usize, String)| id == option.id)
                {
                    ordered_options.push((position, option.id.to_string()));
                }
            }
        }
    }
    ordered_options.sort_by_key(|(position, _)| *position);
    let options: Vec<String> = ordered_options
        .into_iter()
        .map(|(_, option)| option)
        .collect();
    let (slots, denied_path) = extract_slots(query);
    let protected_destructive_target = has_protected_destructive_target(action, query, &tokens);
    let mut explained = Vec::new();
    if let Some(tool) = &tool {
        explained.push(tool.clone());
    }
    if let Some(action) = action {
        explained.push(action.to_string());
    }
    if let Some(object) = object {
        explained.push(object.to_string());
    }
    explained.extend(options.clone());
    let leftover = tokens
        .iter()
        .filter(|token| !explained.iter().any(|known| known == *token))
        .cloned()
        .collect();
    Frame {
        tokens,
        tool,
        tool_explicit,
        action,
        object,
        options,
        slots,
        denied_path,
        protected_destructive_target,
        leftover,
    }
}

/// Keep natural-language requests for protected locations or an entire user's
/// data set out of fuzzy matching. Such prose is not a literal path slot, so
/// the path denylist alone cannot prevent `delete root file` from selecting an
/// unrelated command such as `kubectl delete` or `rsync --delete`.
fn has_protected_destructive_target(action: Option<&str>, query: &str, tokens: &[String]) -> bool {
    if action != Some("delete") {
        return false;
    }
    if tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "root" | "home" | "disk" | "drive" | "filesystem" | "system" | "boot"
        )
    }) || tokens.iter().any(|token| token == "everything")
    {
        return true;
    }

    // Query tokenization removes pronouns as stop words, but “all my files”
    // is materially different from a scoped request such as “all build
    // files.” Retain that distinction only for this safety guard.
    let original = tokenize(query);
    original.iter().any(|token| token == "all")
        && original
            .iter()
            .any(|token| matches!(token.as_str(), "my" | "our"))
        && original.iter().any(|token| {
            matches!(
                token.as_str(),
                "file" | "files" | "folder" | "folders" | "data"
            )
        })
}

/// Prefer an explicitly declared multi-word tool alias over a component tool
/// slug. `docker compose up` belongs to the Compose catalog even though the
/// first word is also Docker's slug. The longest alias wins; ties preserve the
/// leftmost phrase in the user query.
fn multiword_alias_tool(tokens: &[String]) -> Option<&'static Tool> {
    let mut best: Option<(usize, usize, &'static Tool)> = None;
    for tool in all_tools() {
        for alias in tool.aliases {
            let alias_tokens = tokenize(alias);
            if alias_tokens.len() < 2 || alias_tokens.len() > tokens.len() {
                continue;
            }
            let Some(position) = tokens
                .windows(alias_tokens.len())
                .position(|window| window == alias_tokens.as_slice())
            else {
                continue;
            };
            let replace = best.is_none_or(|(best_position, best_len, _)| {
                alias_tokens.len() > best_len
                    || (alias_tokens.len() == best_len && position < best_position)
            });
            if replace {
                best = Some((position, alias_tokens.len(), tool));
            }
        }
    }
    best.map(|(_, _, tool)| tool)
}

fn query_position(query: &str, term: &str) -> Option<usize> {
    let lowered = query.to_ascii_lowercase();
    let term = term.to_ascii_lowercase();
    lowered.find(&term).filter(|position| {
        let before = lowered[..*position].chars().last();
        let after = lowered[*position + term.len()..].chars().next();
        before.is_none_or(|character| !character.is_alphanumeric())
            && after.is_none_or(|character| !character.is_alphanumeric())
    })
}

fn extract_slots(query: &str) -> (Vec<(String, String)>, bool) {
    let words: Vec<&str> = query.split_whitespace().collect();
    let mut slots = Vec::new();
    let mut denied_path = false;
    for word in &words {
        let value = word.trim_matches(|c: char| matches!(c, '\'' | '"' | ','));
        if is_denied_path(value) {
            denied_path = true;
        }
        if valid_slot("url", value) {
            slots.push(("url".to_string(), value.to_string()));
            break;
        }
    }
    if let Some(value) = quoted_value(query) {
        if valid_slot("name", value) {
            slots.push(("name".to_string(), value.to_string()));
        }
    }
    for (index, word) in words.iter().enumerate() {
        if matches!(
            word.to_ascii_lowercase().as_str(),
            "called" | "named" | "name" | "folder" | "file" | "branch" | "message"
        ) {
            if let Some(value) = slot_value_after(&words, index) {
                // “folder named logs” has two slot cue words.  Only the
                // second one owns the value; treating `named` as the folder
                // name would be both surprising and unsafe.
                if matches!(
                    value.to_ascii_lowercase().as_str(),
                    "called"
                        | "named"
                        | "name"
                        | "recursive"
                        | "recursively"
                        | "with"
                        | "from"
                        | "into"
                ) {
                    continue;
                }
                let kind = match word.to_ascii_lowercase().as_str() {
                    "folder" | "file" => "path",
                    "branch" => "branch",
                    "message" => "message",
                    _ => "name",
                };
                if valid_slot(kind, &value) {
                    slots.push((kind.to_string(), value));
                } else if kind == "path" || is_denied_path(&value) {
                    denied_path |= is_denied_path(&value);
                }
            }
        }
    }
    (slots, denied_path)
}

/// Return the word after a slot cue, preserving a complete quoted phrase.
/// Shell-style parsing would be inappropriate here because this is natural
/// language, not a command line; we only need one conservative quoted value.
fn slot_value_after(words: &[&str], cue_index: usize) -> Option<String> {
    let first = *words.get(cue_index + 1)?;
    let Some(quote) = first
        .chars()
        .next()
        .filter(|quote| matches!(quote, '\'' | '"'))
    else {
        return Some(
            first
                .trim_matches(|character| matches!(character, '\'' | '"'))
                .to_string(),
        );
    };
    let mut value = first[quote.len_utf8()..].to_string();
    if let Some(end) = value.find(quote) {
        value.truncate(end);
        return Some(value);
    }
    for word in &words[cue_index + 2..] {
        value.push(' ');
        if let Some(end) = word.find(quote) {
            value.push_str(&word[..end]);
            return Some(value);
        }
        value.push_str(word);
    }
    // An unmatched quote is not a trustworthy value to echo into a command.
    None
}

fn quoted_value(query: &str) -> Option<&str> {
    for quote in ['\'', '"'] {
        let Some(start) = query.find(quote) else {
            continue;
        };
        let from = start + quote.len_utf8();
        let Some(end) = query[from..].find(quote).map(|end| end + from) else {
            continue;
        };
        let value = &query[from..end];
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

pub fn valid_slot(kind: &str, value: &str) -> bool {
    if value.is_empty()
        || value.len() > 100
        || value.starts_with('-')
        || value.contains([';', '&', '|', '`', '$', '\n', '\r'])
    {
        return false;
    }
    match kind {
        "message" => {
            !value.contains(['"', '`', '$', '\\', '\n', '\r'])
                && value.chars().all(|character| !character.is_control())
        }
        "branch" => !value.contains(".."),
        "url" => {
            (value.starts_with("https://")
                || value.starts_with("http://")
                || value.starts_with("git@"))
                && value.chars().all(|character| {
                    character.is_ascii_alphanumeric()
                        || matches!(character, ':' | '/' | '.' | '_' | '@' | '-' | '+')
                })
        }
        "path" => {
            !is_denied_path(value)
                && value.chars().all(|character| {
                    character.is_ascii_alphanumeric()
                        || matches!(character, '.' | '_' | '@' | '-' | '/' | '\\')
                })
        }
        _ => value.chars().all(|char| {
            char.is_ascii_alphanumeric() || matches!(char, '.' | '_' | '@' | '-' | '/')
        }),
    }
}

pub fn is_denied_path(value: &str) -> bool {
    crate::content::safety_policy()
        .path_denylist
        .iter()
        .any(|denied| value.eq_ignore_ascii_case(denied))
}

pub fn decide(query: &str, os: Os, shell: Shell, context: &Context) -> (Frame, Decision) {
    decide_with_history(query, os, shell, context, &History::default())
}

pub fn decide_with_history(
    query: &str,
    os: Os,
    shell: Shell,
    context: &Context,
    history: &History,
) -> (Frame, Decision) {
    let frame = parse(query);
    if frame.protected_destructive_target {
        return (
            frame,
            Decision::Blocked(
                "it targets a protected system, home, or whole-device location; no delete command was returned.",
            ),
        );
    }
    // Natural-language ranking removes conversational filler, but literal
    // command recognition must use untouched query tokens. Otherwise a valid
    // command such as `firebase use <project-id>` can lose its subcommand
    // before the exact-command path sees it.
    let raw_query_tokens = tokenize(query);
    let raw_command = is_raw_command_query(query, &raw_query_tokens, os, shell);
    if raw_command {
        if let Some(recipe) = exact_recipe_command(query, os, shell) {
            return (
                frame,
                Decision::Clear(ScoredCandidate {
                    candidate: Candidate::Recipe(recipe),
                    score: 100.0,
                    reasons: vec!["exact command text".into()],
                }),
            );
        }
        if let Some(candidate) = exact_catalog_command(query, os, shell) {
            return (
                frame,
                Decision::Clear(ScoredCandidate {
                    candidate,
                    score: 100.0,
                    reasons: vec!["exact command text".into()],
                }),
            );
        }
    }
    let raw_command_tokens =
        raw_command.then(|| &raw_query_tokens[..raw_query_tokens.len().min(2)]);
    let mut candidates = Vec::new();
    let mut has_exact_focused_recipe = false;
    for recipe in all_recipes() {
        if frame.tool_explicit
            && frame
                .tool
                .as_deref()
                .is_some_and(|tool| tool != recipe.tool)
        {
            continue;
        }
        if raw_command
            && frame
                .tool
                .as_deref()
                .is_some_and(|tool| tool != recipe.tool)
        {
            continue;
        }
        if raw_command_tokens
            .is_some_and(|tokens| score_recipe_commands(recipe, os, shell, tokens) < 0.95)
        {
            continue;
        }
        if !recipe_available(recipe, os, shell) {
            continue;
        }
        let (mut score, mut reasons) =
            score_recipe(recipe, &frame, query, raw_command_tokens, os, shell);
        if context.matches_tool(recipe.tool) && score > 0.0 {
            score += 1.0;
            reasons.push("package-manager context".into());
        }
        if score > 0.0 {
            has_exact_focused_recipe |= recipe
                .triggers
                .iter()
                .any(|trigger| trigger_matches(&frame.tokens, trigger));
            candidates.push(ScoredCandidate {
                candidate: Candidate::Recipe(recipe),
                score,
                reasons,
            });
        }
    }
    // A focused recipe that clears the confidence threshold is already more
    // specific than a catalog fallback. Otherwise select one command from
    // each matching legacy section. Whole sections remain reserved for
    // explicit workflows.
    let has_focused_recipe = candidates.iter().any(|candidate| {
        matches!(candidate.candidate, Candidate::Recipe(recipe)
            if recipe_has_specific_signal(recipe, &frame, query))
            && candidate.score >= tuning::T_MIN
    });
    if !has_exact_focused_recipe && (!has_focused_recipe || broad_action_query(&frame)) {
        let has_workflow_intent = all_tools()
            .flat_map(|tool| tool.sections())
            .any(|section| is_workflow_intent(section, &frame));
        for tool in all_tools() {
            // Once a tool is explicit, unrelated legacy workflows cannot improve
            // the recommendation.  This also avoids scoring the full catalogue
            // for the common `ezcmd git ...` / `ezcmd docker ...` path.
            if frame.tool.as_deref().is_some_and(|slug| slug != tool.slug) {
                continue;
            }
            let filtered_tokens: Vec<String> = frame
                .tokens
                .iter()
                .filter(|token| !is_tool_token(tool, token))
                .cloned()
                .collect();
            let command_tokens = if filtered_tokens.is_empty() {
                &frame.tokens
            } else {
                &filtered_tokens
            };
            for section in tool
                .sections()
                .filter(|section| section.available_for(os, shell))
            {
                if has_workflow_intent && !is_workflow_intent(section, &frame) {
                    continue;
                }
                if is_workflow_intent(section, &frame) {
                    let mut score = score_workflow(tool, section, command_tokens) * 5.0;
                    // Repository initialization is the documented multi-step
                    // workflow. Action and object evidence must outweigh
                    // conversational filler such as "what command do I use".
                    if section.title.eq_ignore_ascii_case("Initializing a Repo")
                        && frame.action == Some("create")
                        && frame.object == Some("repo")
                    {
                        score += tuning::W_ACTION + tuning::W_OBJECT;
                    }
                    if score > 0.0 {
                        candidates.push(ScoredCandidate {
                            candidate: Candidate::Workflow { tool, section },
                            score,
                            reasons: vec!["explicit workflow intent".into()],
                        });
                    }
                    continue;
                }

                // Keep every meaningful word for catalog commands even when
                // the query begins with a literal `<tool> <subcommand>`. The
                // raw-command path above narrows recipe retrieval, but
                // discarding the remaining words here turned
                // `flutter build android apk` into merely `flutter build`.
                let tokens = command_tokens;
                let Some((command, mut score, exact_alias_match)) =
                    best_legacy_command(tool, section, os, shell, tokens, frame.action)
                else {
                    continue;
                };
                let mut reasons = vec!["catalog command match".into()];
                if exact_alias_match {
                    reasons.push("exact section alias".into());
                }
                if frame.tool_explicit && frame.action.is_some() {
                    score += tuning::W_ACTION;
                    reasons.push("action".into());
                }
                if tool.slug == context.package_manager {
                    score += 1.0;
                    reasons.push("package-manager context".into());
                }
                candidates.push(ScoredCandidate {
                    candidate: Candidate::LegacyCommand {
                        tool,
                        section,
                        command,
                    },
                    score,
                    reasons,
                });
            }
        }
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.candidate.title().cmp(&right.candidate.title()))
    });
    if let Some(top) = candidates.first().map(|candidate| candidate.score) {
        for candidate in &mut candidates {
            if top - candidate.score <= tuning::T_MARGIN {
                let boost = history.boost(&candidate.candidate.id());
                if boost > 0.0 {
                    candidate.score += boost;
                    candidate.reasons.push(format!("history +{boost:.2}"));
                }
            }
        }
        candidates.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.candidate.title().cmp(&right.candidate.title()))
        });
    }
    if candidates
        .first()
        .is_none_or(|candidate| candidate.score < tuning::T_MIN)
    {
        candidates.truncate(3);
        return (frame, Decision::NoMatch(candidates));
    }
    // A lone action word has deliberately underspecified intent.  When it
    // names both a focused command and a workflow, let the user choose rather
    // than silently selecting one interpretation (for example `undo`).
    if broad_action_query(&frame) {
        let broad: Vec<ScoredCandidate> = candidates
            .iter()
            .filter(|candidate| candidate.score >= tuning::T_MIN)
            .take(tuning::MAX_CHOICES)
            .cloned()
            .collect();
        if broad.len() > 1 {
            return (frame, Decision::Ambiguous(broad));
        }
    }
    let top = candidates[0].score;
    let ambiguous: Vec<ScoredCandidate> = candidates
        .iter()
        .filter(|candidate| {
            candidate.score >= tuning::T_MIN && top - candidate.score < tuning::T_MARGIN
        })
        .take(tuning::MAX_CHOICES)
        .cloned()
        .collect();
    if ambiguous.len() > 1 {
        (frame, Decision::Ambiguous(ambiguous))
    } else {
        (frame, Decision::Clear(candidates.remove(0)))
    }
}

fn is_tool_token(tool: &Tool, token: &str) -> bool {
    // A display name can itself mention multiple ecosystems (for example,
    // `Django, Flask & FastAPI`). It and broad aliases help identify a tool,
    // but only the unambiguous slug is removed from the task terms.
    token == tool.slug
}

fn broad_action_query(frame: &Frame) -> bool {
    frame.action.is_some()
        && frame.tool.is_none()
        && frame.object.is_none()
        && frame.options.is_empty()
        && frame.tokens.len() == 1
}

fn is_raw_command_query(query: &str, tokens: &[String], os: Os, shell: Shell) -> bool {
    if query.contains("--") {
        return true;
    }
    // Phrases such as "bash command help" are natural-language section
    // aliases, not literal invocations. Keep `<command>` as a signal for the
    // documented placeholder form (for example `pnpm -r <command>`).
    if matches!(
        tokens.get(1).map(String::as_str),
        Some("command" | "commands")
    ) && !query.contains("<command>")
    {
        return false;
    }
    let Some(first) = tokens.first() else {
        return false;
    };
    tokens.len() >= 2
        && all_tools().any(|tool| {
            let named_tool = first == tool.slug
                || tokenize(tool.name).iter().any(|name| name == first)
                || tool
                    .aliases
                    .iter()
                    .flat_map(|alias| tokenize(alias))
                    .any(|alias| alias == *first);
            named_tool
                && tool
                    .sections()
                    .flat_map(|section| section.commands.iter())
                    .any(|command| {
                        command
                            .resolve_for(os, shell)
                            .map(tokenize)
                            .is_some_and(|command_tokens| command_tokens.get(1) == tokens.get(1))
                    })
        })
}

/// A literal catalog command is unambiguous even when nearby sections share
/// its subcommands. This is the fast path for the documented raw-command
/// query form (`<tool> <subcommand> ...`); natural-language retrieval still
/// handles every other request.
fn exact_catalog_command(query: &str, os: Os, shell: Shell) -> Option<Candidate> {
    let query = query.trim();
    all_tools().find_map(|tool| {
        tool.sections().find_map(|section| {
            section.visible_commands_for(os, shell).find_map(|command| {
                command
                    .resolve_for(os, shell)
                    .is_some_and(|text| text.eq_ignore_ascii_case(query))
                    .then_some(Candidate::LegacyCommand {
                        tool,
                        section,
                        command,
                    })
            })
        })
    })
}

/// Recipes carry the richer answer: variants, safety metadata, and follow-up
/// hints. When a raw command appears in both recipe and legacy data, retain
/// that richer form rather than downgrading it to a one-line catalog answer.
fn exact_recipe_command(query: &str, os: Os, shell: Shell) -> Option<&'static Recipe> {
    let query = query.trim();
    all_recipes().iter().find(|recipe| {
        recipe
            .base
            .into_iter()
            .chain(recipe.primary)
            .chain(recipe.variants.iter().copied())
            .filter_map(|command| command.resolve_for(os, shell))
            .any(|command| command.eq_ignore_ascii_case(query))
    })
}

/// Recommendation workflows are retrieved from their concise title/alias
/// metadata.  The legacy help matcher continues to search command and note
/// text when browsing sections, but rebuilding those large strings for every
/// normal query made the focused path needlessly expensive.
fn score_workflow(tool: &Tool, section: &Section, tokens: &[String]) -> f32 {
    let title = field_score_text(tokens, section.title);
    let aliases = section
        .aliases
        .iter()
        .map(|alias| {
            let alias_tokens: Vec<String> = tokenize(alias)
                .into_iter()
                .filter(|token| !is_tool_token(tool, token))
                .collect();
            field_score(tokens, &alias_tokens)
        })
        .fold(0.0, f32::max);
    let exact = section.aliases.iter().any(|alias| {
        let alias_tokens: Vec<String> = tokenize(alias)
            .into_iter()
            .filter(|token| !is_tool_token(tool, token))
            .collect();
        exact_phrase_match(tokens, &alias_tokens)
    }) || exact_phrase_match(tokens, &tokenize(section.title));
    title.max(aliases) + if exact { 1.0 } else { 0.0 }
}

/// Select the best single command in a legacy section. Commands and their
/// notes are already data-authored for every supported package, so this gives
/// the old catalog focused-answer behaviour without guessing from an entire
/// section at render time. Ties retain data order, making the first (safest
/// default) command win until a query supplies a more specific word.
fn best_legacy_command(
    tool: &'static Tool,
    section: &'static Section,
    os: Os,
    shell: Shell,
    tokens: &[String],
    action: Option<&str>,
) -> Option<(&'static Command, f32, bool)> {
    let section_score = score_workflow(tool, section, tokens);
    // A section alias is the data-owned vocabulary for the task. A complete
    // alias match deserves to outweigh incidental command/note word overlap:
    // `flutter build android apk` must beat a fuzzy `build web` match.
    let exact_alias_match = section.aliases.iter().any(|alias| {
        let alias_tokens: Vec<String> = tokenize(alias)
            .into_iter()
            .filter(|token| !is_tool_token(tool, token))
            .collect();
        exact_phrase_match(tokens, &alias_tokens)
    });
    let section_has_action_match = action.is_some_and(|action| {
        section.visible_commands_for(os, shell).any(|command| {
            command
                .resolve_for(os, shell)
                .is_some_and(|text| action_matches_command(action, text, command.note))
        })
    });
    let mut best = None;
    for command in section.visible_commands_for(os, shell) {
        let Some(text) = command.resolve_for(os, shell) else {
            continue;
        };
        if action.is_some_and(|action| {
            (!exact_alias_match || section_has_action_match)
                && !action_matches_command(action, text, command.note)
        }) {
            continue;
        }
        let command_score = field_score_text(tokens, text);
        let note_score = field_score_text(tokens, command.note);
        let score = command_score * 4.0
            + note_score * 4.0
            + section_score * 1.5
            + if exact_alias_match {
                tuning::W_EXACT_ALIAS
            } else {
                0.0
            };
        // A complete, data-authored alias identifies the section, but it
        // must not blindly select the section's first command. For example,
        // "change directory" belongs to navigation but must select
        // `cd <path>`, not the preceding `pwd` command. The exact-alias
        // bonus stays in every candidate score while command and note text
        // select the most relevant entry. Strictly greater retains the
        // declared first command only when the query supplies no further
        // command-level signal.
        if score > 0.0 && best.is_none_or(|(_, best_score, _)| score > best_score) {
            best = Some((command, score, exact_alias_match));
        }
    }
    best
}

fn action_matches_command(action: &str, command: &str, note: &str) -> bool {
    let Some((_, terms)) = ACTIONS.iter().find(|(canonical, _)| *canonical == action) else {
        return true;
    };
    let command_tokens = tokenize(command);
    let note_tokens = tokenize(note);
    // Do not let a descriptive note such as "starts from an example" make a
    // `create` command qualify as a `run` command. Explicit verbs in the
    // command itself take precedence over note text.
    if ACTIONS.iter().any(|(other_action, other_terms)| {
        *other_action != action
            && command_tokens.iter().any(|token| {
                other_terms
                    .iter()
                    .any(|term| action_word_matches(token, term))
            })
    }) {
        return false;
    }
    terms.iter().any(|term| {
        command_tokens
            .iter()
            .chain(note_tokens.iter())
            .any(|token| action_word_matches(token, term))
    })
}

/// Action recognition intentionally uses exact words plus ordinary present
/// tense forms. Stem matching would make the noun/adjective `installed`
/// qualify as the imperative `install`, so `apt install` could select
/// `apt list --installed`.
fn action_word_matches(token: &str, term: &str) -> bool {
    token == term
        || token.strip_suffix('s').is_some_and(|root| root == term)
        || token.strip_suffix("ing").is_some_and(|root| root == term)
}

/// A full, two-way match prevents a short query from being considered an
/// exact hit for a longer alias merely because the query's words are a subset
/// of it. Morphological forms still use the existing token matcher.
fn exact_phrase_match(tokens: &[String], phrase_tokens: &[String]) -> bool {
    !tokens.is_empty()
        && !phrase_tokens.is_empty()
        && field_score(tokens, phrase_tokens) >= 0.95
        && field_score(phrase_tokens, tokens) >= 0.95
}

fn is_workflow_intent(section: &Section, frame: &Frame) -> bool {
    if frame
        .tokens
        .iter()
        .any(|token| matches!(token.as_str(), "workflow" | "steps" | "sequence"))
    {
        return true;
    }
    // Repository initialization is the documented multi-step workflow: a
    // usable result needs initialization, commit, remote, and push together.
    section.title.eq_ignore_ascii_case("Initializing a Repo")
        && frame.object == Some("repo")
        && frame
            .tokens
            .iter()
            .any(|token| crate::matcher::token_match_score(token, "initialize") >= 0.70)
}

fn trigger_matches(tokens: &[String], trigger: &str) -> bool {
    tokenize(trigger).iter().all(|term| {
        tokens
            .iter()
            .any(|token| crate::matcher::token_match_score(token, term) >= 0.70)
    })
}

/// A high score based only on a named tool is not enough to hide the general
/// catalog. This keeps a query such as `docker logs` from being captured by a
/// loosely-related `docker run container` recipe, while still allowing a
/// complete trigger, a matching structured frame, or a specifically promoted
/// variant to take precedence.
fn recipe_has_specific_signal(recipe: &Recipe, frame: &Frame, query: &str) -> bool {
    if recipe.triggers.iter().any(|trigger| {
        query_position(query, trigger).is_some() || trigger_matches(&frame.tokens, trigger)
    }) {
        return true;
    }
    if structured_recipe_match(recipe, frame) {
        return true;
    }
    recipe.variants.iter().any(|variant| {
        variant
            .matches
            .iter()
            .any(|term| query_position(query, term).is_some())
    })
}

fn structured_recipe_match(recipe: &Recipe, frame: &Frame) -> bool {
    recipe
        .action
        .is_some_and(|action| frame.action == Some(action))
        && recipe
            .object
            .is_some_and(|object| frame.object == Some(object))
}

/// Like `matcher::field_score`, but consumes static title/alias text directly.
/// The recommender traverses many workflow aliases for broad requests, and
/// allocating a `Vec<String>` for every field dominated its release p95.
fn field_score_text(query_tokens: &[String], text: &str) -> f32 {
    if query_tokens.is_empty() || text.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    for query in query_tokens {
        let mut best = 0.0f32;
        for raw in text.split(|character: char| !character.is_alphanumeric()) {
            if raw.len() < 2 {
                continue;
            }
            let score = if raw.bytes().all(|byte| !byte.is_ascii_uppercase()) {
                crate::matcher::token_match_score(query, raw)
            } else {
                let lowered = raw.to_ascii_lowercase();
                crate::matcher::token_match_score(query, &lowered)
            };
            best = best.max(score);
        }
        sum += best;
    }
    sum / query_tokens.len() as f32
}

fn recipe_available(recipe: &Recipe, os: Os, shell: Shell) -> bool {
    let command = recipe.base.or(recipe.primary);
    command.is_some_and(|command| command.resolve_for(os, shell).is_some())
}

fn score_recipe(
    recipe: &Recipe,
    frame: &Frame,
    query: &str,
    raw_command_tokens: Option<&[String]>,
    os: Os,
    shell: Shell,
) -> (f32, Vec<String>) {
    let mut score = 0.0;
    let mut reasons = Vec::new();
    let mut matched = 0usize;
    let exact_trigger = recipe
        .triggers
        .iter()
        .any(|trigger| query_position(query, trigger).is_some());
    let trigger = recipe
        .triggers
        .iter()
        .map(|trigger| field_score(&frame.tokens, &tokenize(trigger)))
        .fold(0.0, f32::max);
    // A one-word fuzzy overlap must not turn a generic word such as
    // "uninstall" into a package-specific recommendation.  Tool-less focused
    // recipes need an exact phrase (or a near-complete trigger) instead.
    let complete_trigger = recipe.triggers.iter().any(|trigger| {
        let trigger_tokens = tokenize(trigger);
        trigger_tokens.len() <= frame.tokens.len()
            && field_score(&frame.tokens, &trigger_tokens) >= 0.95
    });
    if frame.tool.as_deref() == Some(recipe.tool)
        && !recipe_has_specific_signal(recipe, frame, query)
        && raw_command_tokens
            .is_none_or(|tokens| score_recipe_commands(recipe, os, shell, tokens) < 0.95)
    {
        return (0.0, reasons);
    }
    // Do not let a tool hit plus a single fuzzy word become a focused answer.
    // The generic catalog has the actual command vocabulary for these cases.
    if frame.tool.as_deref() == Some(recipe.tool)
        && frame.action.is_none()
        && frame.object.is_none()
        && frame.options.is_empty()
        && !exact_trigger
        && !complete_trigger
        && trigger < 0.85
        && raw_command_tokens
            .is_none_or(|tokens| score_recipe_commands(recipe, os, shell, tokens) < 0.95)
    {
        return (0.0, reasons);
    }
    // Naming a tool is not enough to select an unrelated recipe for that
    // tool. `nextjs start dev server` must reach the catalog's dev command,
    // not the project-creation recipe.
    if frame.tool.as_deref() == Some(recipe.tool)
        && frame.action.is_some()
        && frame.action != recipe.action
        && !exact_trigger
        && trigger < 0.70
    {
        return (0.0, reasons);
    }
    // A one-word action query is intentionally retained as a candidate so
    // the decision layer can present the focused and workflow interpretations
    // as an ambiguity.  Other tool-less partial overlaps remain excluded.
    let command_score =
        raw_command_tokens.map(|tokens| score_recipe_commands(recipe, os, shell, tokens));
    if frame.tool.is_none()
        && !exact_trigger
        && !complete_trigger
        && !broad_action_query(frame)
        && command_score.unwrap_or_default() == 0.0
    {
        return (0.0, reasons);
    }
    if frame.tool.as_deref() == Some(recipe.tool) {
        score += tuning::W_TOOL;
        reasons.push("tool".into());
        matched += 1;
    }
    if recipe
        .action
        .is_some_and(|action| frame.action == Some(action))
    {
        score += tuning::W_ACTION;
        reasons.push("action".into());
        matched += 1;
    }
    if frame.object == recipe.object && recipe.object.is_some() {
        score += tuning::W_OBJECT;
        reasons.push("object".into());
        matched += 1;
    }
    if trigger > 0.0 {
        score += tuning::W_TRIGGER * trigger;
        reasons.push(format!("trigger {:.2}", trigger));
        matched += (trigger * frame.tokens.len() as f32).round() as usize;
    }
    if exact_trigger {
        // Longer exact triggers are more specific than a short phrase that
        // merely happens to be contained in the query. This separates
        // `npm install packages` from `npm install <package>` without
        // hard-coding either command in the matcher.
        let most_specific_exact_trigger = recipe
            .triggers
            .iter()
            .filter(|trigger| query_position(query, trigger).is_some())
            .map(|trigger| tokenize(trigger).len())
            .max()
            .unwrap_or(1) as f32;
        score += 0.75 + 0.75 * most_specific_exact_trigger;
        reasons.push("exact trigger".into());
    }
    if let Some(command_score) = command_score.filter(|score| *score > 0.0) {
        score += 5.0 * command_score;
        reasons.push(format!("command {:.2}", command_score));
        matched += (command_score * frame.tokens.len() as f32).round() as usize;
    }
    let options = recipe
        .options
        .iter()
        .filter(|option| {
            frame
                .options
                .iter()
                .any(|matched_option| matched_option == option.id)
        })
        .count();
    if options > 0 {
        score += tuning::W_OPTION * options.min(tuning::OPT_CAP) as f32;
        reasons.push(format!("{options} option(s)"));
        matched += options;
    }
    let total = frame.tokens.len().max(1);
    let coverage = (matched.min(total) as f32) / total as f32;
    score += tuning::W_COVERAGE * coverage;
    score -= tuning::W_LEFTOVER * (1.0 - coverage);
    if recipe.kind == RecipeKind::Compose
        && !query.is_empty()
        && frame.tool.as_deref() == Some(recipe.tool)
    {
        score += 0.25;
    }
    (score, reasons)
}

fn score_recipe_commands(recipe: &Recipe, os: Os, shell: Shell, tokens: &[String]) -> f32 {
    recipe
        .base
        .into_iter()
        .chain(recipe.primary)
        .chain(recipe.variants.iter().copied())
        .filter_map(|command| command.resolve_for(os, shell))
        .map(|command| field_score_text(tokens, command))
        .fold(0.0, f32::max)
}

pub fn explain(frame: &Frame, decision: &Decision) -> String {
    let mut text = format!("frame: tool={:?}, action={:?}, object={:?}, options={:?}, slots={:?}, denied_path={}, protected_destructive_target={}, leftover={:?}\nthresholds: min={}, margin={}\n", frame.tool, frame.action, frame.object, frame.options, frame.slots, frame.denied_path, frame.protected_destructive_target, frame.leftover, tuning::T_MIN, tuning::T_MARGIN);
    if let Decision::Blocked(reason) = decision {
        text.push_str(&format!("blocked: {reason}\n"));
        return text;
    }
    let candidates = match decision {
        Decision::Clear(candidate) => std::slice::from_ref(candidate),
        Decision::Ambiguous(candidates) | Decision::NoMatch(candidates) => candidates,
        Decision::Blocked(_) => unreachable!("blocked decisions return above"),
    };
    for candidate in candidates.iter().take(5) {
        text.push_str(&format!(
            "{}: {:.2} ({})\n",
            candidate.candidate.id(),
            candidate.score,
            candidate.reasons.join(", ")
        ));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context {
            git_repo: false,
            has_remote_origin: false,
            package_manager: "npm",
            ecosystems: vec![],
        }
    }

    #[test]
    fn specific_next_query_selects_the_compose_recipe() {
        let (_, decision) = decide(
            "how do i make a nextjs project with typescript and tailwindcss",
            Os::Linux,
            Shell::Cmd,
            &context(),
        );
        assert!(
            matches!(decision, Decision::Clear(ScoredCandidate { candidate: Candidate::Recipe(recipe), .. }) if recipe.id == "next.create")
        );
    }

    #[test]
    fn init_repo_keeps_the_full_workflow() {
        let (_, decision) = decide("initialize a repo", Os::Linux, Shell::Cmd, &context());
        assert!(
            matches!(decision, Decision::Clear(ScoredCandidate { candidate: Candidate::Workflow { section, .. }, .. }) if section.title == "Initializing a Repo")
        );
    }

    #[test]
    fn nonsense_is_never_guessed() {
        let (_, decision) = decide("make me a sandwich", Os::Linux, Shell::Cmd, &context());
        assert!(matches!(decision, Decision::NoMatch(_)));
    }

    #[test]
    fn unsafe_slot_text_is_dropped() {
        let frame = parse("make next project named a; rm -rf ~");
        assert!(frame.slots.is_empty());
        assert!(!valid_slot("name", "a;rm-rf"));
        assert!(!valid_slot("path", "/"));
        assert!(!valid_slot("path", "C:\\"));
    }

    #[test]
    fn quoted_message_slot_keeps_the_complete_phrase() {
        let frame = parse(r#"initialize repo message "initial release""#);
        assert!(frame
            .slots
            .iter()
            .any(|(kind, value)| kind == "message" && value == "initial release"));
        assert!(!valid_slot("message", r"unsafe\value"));
    }

    #[test]
    fn workflow_history_identifiers_are_safe_and_stable() {
        assert_eq!(identifier("Undo & Rollback"), "undo-rollback");
        assert_eq!(identifier("Tags (release)"), "tags-release");
    }

    #[test]
    fn raw_command_text_selects_its_focused_recipe() {
        let (_, decision) = decide("reset --hard", Os::Linux, Shell::Cmd, &context());
        assert!(
            matches!(decision, Decision::Clear(ScoredCandidate { candidate: Candidate::Recipe(recipe), .. }) if recipe.id == "git.undo-last-commit")
        );
    }
}
