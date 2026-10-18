# ezCMD Version 2: Implementation Guide (for AI coding agents)

This document tells an AI coding agent **how to build ezCMD version 2** on top of the shipped v1 code. It is written to be handed to the agent together with `PRD.md`, `README.md`, `IMPLEMENTATION.md` (the v1 guide), and `VERSION2_COMMANDS.md` (the v2 command library).

Version 2 has **one behavioral change** and **one content expansion**:

1. **Table-of-contents help.** `ezcmd --help` stops dumping every command. It lists categories and tools, and you drill down to a tool's sections and then to a section's commands.
2. **Command library expansion.** 61 new tools plus 9 new Git sections, 286 sections in total and 1,651 command lines (1,639 entries once OS pairs are merged), all taken from `VERSION2_COMMANDS.md`.

Everything else about ezCMD stays as it is in v1.

---

## 0. How to use this document

1. **Read all five files before writing code:** `PRD.md`, `README.md`, `IMPLEMENTATION.md`, `VERSION2_COMMANDS.md`, and this file.
2. **Inspect the real repository first (milestone V0).** `IMPLEMENTATION.md` describes how v1 was *meant* to be built. The shipped code (v0.2.1) has additions that guide doesn't cover, such as `--install`, `--uninstall`, `--welcome`, the install scripts, and a `-shell` tool. Where this document and the actual code disagree on names or structure, **follow the code and tell me about the difference**. Don't rewrite modules just to match the guide.
3. **Precedence:** this file wins over `IMPLEMENTATION.md` for anything in v2's scope (help output, data format, matching, rendering). `IMPLEMENTATION.md` still governs everything else, including all of its hard constraints. Section 14 lists where v2 deliberately changes v1 decisions.
4. **The command library is already verified.** `VERSION2_COMMANDS.md` was checked against current documentation in several passes, the last on October 18, 2026, with all corrections applied (see its Verification status section). Import every command and note as written. Two items there are still unconfirmed (`tsc --init` on TypeScript 7, and nano's `Alt` shortcuts on current macOS). Import them as written and list them in your report. Change any other command only if you can show it is wrong, and list each change in your report.
5. Work **one milestone at a time** (Section 12). Don't start the next one until the current "Done when" list passes and I approve.
6. After each milestone run the checks in Section 13 and report what you built, what you tested, and any deviation.
7. **Do not add features outside this document.** If something is unclear, ask.

---

## 1. Scope

### In scope
| Area | Change |
|---|---|
| Help output | `--help` becomes a three-level table of contents (Section 3) |
| Section lookup | `--help -<section>` searches every tool (Section 3.4) |
| Data format | Categories, groups, tool metadata, tool aliases, platforms, draft flag (Section 5) |
| Matching | The tool name in a query narrows results; the chooser shows tool names (Section 7) |
| Rendering | Section frames show `Tool: Section` as the title; TOC output (Section 6) |
| Library | Import all of `VERSION2_COMMANDS.md` with aliases and tests (Sections 8 to 10) |
| Docs | Update `README.md` usage and examples, and bump the version (Section 12, V9) |

### Out of scope
- No new commands or flags beyond those in Section 3.
- No config files, user-defined sheets, shell keybind, community packs, or AI fallback.
- No change to `--install`, `--uninstall`, `--welcome`, `--plain`, or `--version`.
- No change to the install scripts or the release workflow, other than what the version bump needs.
- No runtime network, file reads, background work, or new runtime dependencies.

### Hard constraints (all v1 constraints still apply)
Offline only, no threads or async, no `clap`, no `regex`, no `serde` or `toml` at runtime, no `unsafe`, no `unwrap` or `panic` in non-test code, ASCII-only output, never execute displayed commands, broken pipes handled quietly. See `IMPLEMENTATION.md` Section 1.

### Budgets
| Metric | v1 target | v2 target |
|---|---|---|
| Cold start to output | < 20 ms | **< 20 ms** (aim for under 15 ms, the README badge says 15) |
| Release binary | < 3 MB | **< 3 MB** (aim for under 1.5 MB) |
| Peak memory | < 10 MB | **< 10 MB** |

The library grows from dozens of sections to about 290 and roughly 1,600 commands. That's a small amount of text, but **measure before and after** (milestone V0 and V10) so a regression is caught early.

---

## 2. Tools & environment

Same toolchain as `IMPLEMENTATION.md` Section 2 (Rust stable, `cargo-insta`, `hyperfine`, `cargo-bloat`, the three-OS CI matrix). Two additions:

| Tool | Purpose |
|---|---|
| **Python 3** (standard library only) | For the one-off converter in V8. It is a development tool under `tools/`. It is **not** part of the build and not shipped |
| Any text diff tool | To compare the converter's output with `VERSION2_COMMANDS.md` |

Adding a dependency still needs a written justification (what it does, why `std` isn't enough, size impact). Expect to add **none**.

---

## 3. The new help behavior

### 3.1 The three levels

```
ezcmd --help                       Level 1: categories, and the tools in each
ezcmd --help -<tool>               Level 2: that tool's groups and section titles
ezcmd --help -<tool> -<section>    Level 3: that section's commands
```

Plus two shortcuts: a section search across all tools (3.4) and the OS view (3.5).

### 3.2 Level 1: `ezcmd --help`

Prints the usage guide, then every category with its tools. Only tools that exist on the detected OS are listed.

```
ezcmd 2.0.0
Find the right command, in order.

HOW TO USE
  ezcmd <query>                        Search for a task or a command
  ezcmd --help -<tool>                 List a tool's sections       ezcmd --help -docker
  ezcmd --help -<tool> -<section>      Show a section's commands    ezcmd --help -docker -logs
  ezcmd --help -<section>              Search every tool            ezcmd --help -initialize
  ezcmd --help -<os>                   Browse as another OS         ezcmd --help -windows

Version Control
  -git          Git                                21 sections

JavaScript & Web
  -npm          Node.js & npm                      15 sections
  -pnpm         pnpm                                5 sections
  ...

Containers & DevOps
  -docker       Docker                              9 sections
  -compose      Docker Compose                      5 sections
  -gh           GitHub CLI (gh)                     7 sections
  ...
```

Rules:
- The section counts in the example above are illustrative. Print the real counts.
- Categories follow the order in `data/categories.toml` (Section 5.4). Tools inside a category follow their `order` value, then alphabetical by name.
- The `-slug` column is exactly what the user types after `--help`.
- Counts are **sections visible on the current OS**, and a tool with zero visible sections is hidden.
- With `--plain`, drop colors but keep the layout.

### 3.3 Level 2: `ezcmd --help -<tool>`

```
Docker  (-docker)
Also matches: container, containers, dockerfile

Containers
  -Check Docker & Log In
  -Run a Container
  -Manage Containers
  -Inspect & Debug

Images & Registries
  -Build Images
  -Images & Tags

Storage, Networks & Cleanup
  -Volumes
  -Networks
  -Clean Up Space

View a section:  ezcmd --help -docker -run
The section name can be shortened or misspelled. It is fuzzy-matched.
```

Rules:
- Print tool-level notes (the `Note:` lines in `VERSION2_COMMANDS.md`) under the title.
- Print a group heading **only if the tool has more than one group**. Tools with a single group (the "Everything X" groups in the library file) show their sections without a heading.
- Section lines are written as `-Title`, matching the format in the feature request. Keep the leading `-` so users see what to type.
- Aliases come from the tool's `aliases` field (the "Also matches" names in the library file).

### 3.4 Level 3 and section search

| Invocation | Behavior |
|---|---|
| `ezcmd --help -docker -run` | Fuzzy-match "run" **inside Docker only**. One match prints the section. Several close matches use the `[1/2/3]` chooser. No match prints the three closest titles in that tool and exits 1 |
| `ezcmd --help -initialize` | If `initialize` is not an OS or tool name, treat it as a **section search across all tools**. One match prints the section. Several use the chooser with `Tool: Section` labels |
| `ezcmd --help -- <words>` | Forces a section search across all tools, even if the first word is also a tool name (for example `ezcmd --help -- find files`) |

The section words use the same rule as v1: the first non-selector token starts the section text, a single leading `-` is stripped, and following tokens that don't start with `-` are appended.

### 3.5 OS view

`-windows`, `-linux`, `-macos` still switch the OS used for visibility and command resolution. They combine with everything above, in any order:

```
ezcmd --help -windows                       Level 1, as Windows
ezcmd --help -windows -docker               Level 2, as Windows
ezcmd --help -windows -docker -run          Level 3, as Windows
```

If a tool is requested but has no visible sections on the chosen OS (for example `--help -linux -pods`), print `CocoaPods is not available on Linux. It works on: macOS.` to stderr and exit `1`. The list of OS names comes from the tool's `platforms` field.

### 3.6 Parser grammar (replaces the help rules in `IMPLEMENTATION.md` Section 6.2)

After `--help`, process tokens left to right:

1. `--plain` is accepted anywhere.
2. `--` stops selector processing. All remaining tokens are the section text. This also forces a section search, even when the first word is a tool name.
3. A token starting with a single `-` is a **selector**, unless section text has already started (rule 5). Strip the dash, lowercase it, and look it up in this order:
   1. An OS name (`windows`, `linux`, `macos`) sets `os`. A repeated OS selector replaces the earlier one.
   2. A tool **slug or flag alias** (`Tool::matches_flag`) sets `tool` to that tool's slug. If a tool is already set, this is an error (`CliError::TwoTools`). Shadowed aliases (Section 5.8) are not flag aliases, so they skip this step.
   3. Anything else **begins the section text**.
4. A token that does not start with `-` belongs to the section text if the text has started. If it has not, it is an error (`CliError::StrayWord`), so that typos like `--help docker` get a clear message.
5. Once the section text has started, a token that starts with a single `-` is an error (`CliError::SelectorAfterText`).
6. The mode follows from what was set. Neither tool nor text: Level 1. Only a tool: Level 2. A tool and text: Level 3. Text without a tool: section search.

The section words use the same rule as v1: a single leading `-` on the first word is stripped, and following tokens that do not start with `-` are appended.

| Args | Parsed result |
|---|---|
| `--help` | Level 1 |
| `--help -docker` | Level 2, tool = docker |
| `--help -k8s` | Level 2, tool = kubectl (alias) |
| `--help -docker -run` | Level 3, tool = docker, section text = `run` |
| `--help -docker -run a container` | Same, section text = `run a container` |
| `--help -initialize` | Section search, all tools, text = `initialize` |
| `--help -windows -docker` | Level 2, os = windows |
| `--help -docker -windows -logs` | Level 3, os = windows, tool = docker |
| `--help -- find files` | Section search, text = `find files` |
| `--help -docker -- logs` | Level 3, tool = docker, text = `logs`, forced |
| `--help -history` | Section search, text = `history` (a shadowed alias, see 5.8) |
| `--help -npm -cache` | Level 3, tool = npm, text = `cache` |
| `--help --plain -docker` | Level 2, plain |
| `--help -docker -npm` | Error: two tools |
| `--help docker` | Error: stray word, with a hint to use `-docker` |
| `--help -docker -run -windows` | Error: selector after the section text |

### 3.7 Types

`cli.rs` replaces the v1 `Action::Help { os, tool, section }` variant. Every other variant (`Usage`, `Version`, `Query`, and the shipped install, uninstall and welcome actions) is unchanged.

```rust
pub enum Action { /* unchanged variants */, Help(HelpRequest) }

pub struct HelpRequest {
    pub os:      Option<Os>,
    pub tool:    Option<&'static str>,  // the resolved slug, never the alias the user typed
    pub section: Option<String>,        // section text, already joined with spaces
    pub forced:  bool,                  // true when the section text came after `--`
}
pub enum HelpLevel { Overview, ToolToc, ToolSection, SectionSearch }
impl HelpRequest { pub fn level(&self) -> HelpLevel { /* rule 6 in 3.6 */ } }

pub enum CliError {
    UnknownFlag(String),                   // any other `--word`
    TwoTools { first: String, second: String },
    StrayWord(String),
    SelectorAfterText(String),
    EmptyQuery,                            // a query with no usable words, as in v1
}
```

`UnknownSelector` from v1 no longer exists. `CliError` implements `Display`. `main.rs` prints it to stderr, then a one-line hint, and exits `2`:

| Error | Hint |
|---|---|
| `StrayWord` | `Try: ezcmd --help -<word>` |
| `SelectorAfterText` | `Put -<os> and -<tool> before the section text` |
| `TwoTools` | `Use one tool at a time` |
| others | `Try: ezcmd --help` (as in v1) |

### 3.8 `main.rs` flow

The v1 flow (`IMPLEMENTATION.md` Section 6.7) stays, with these changes:

```
Query(q)  -> matcher::search(os = Os::current(), opts = { only_tool: None, tool_tokens: true })
               ToolOnly(tool) -> toc_tool, exit 0
               Single         -> render section
               Ambiguous      -> interactive ? prompt::choose -> render chosen : render best hit
               NoMatch        -> message + closest 3 as `Tool: Section`, exit 1

Help(req) by req.level():
  Overview       -> usage_guide, then toc_overview. Exit 0
  ToolToc        -> tool not visible on the OS: platforms message (3.5), exit 1
                    otherwise toc_tool. Exit 0
  ToolSection    -> matcher::search(opts = { only_tool: Some(slug), tool_tokens: false })
  SectionSearch  -> matcher::search(opts = { only_tool: None,       tool_tokens: false })
                    Single / Ambiguous / NoMatch are handled exactly as for Query
Write errors: BrokenPipe -> exit 0 quietly; other io errors -> stderr, exit 1.
```

Help lookups set `tool_tokens: false` because the user has already said what to search for. The matcher scores sections only (Section 7.1, step 5).

### 3.9 Exit codes

| Code | Meaning |
|---|---|
| `0` | Success, including every help level, `--version`, and no-argument usage |
| `1` | No match (query or section search), a tool that is not available on the chosen OS, or the user aborted the chooser |
| `2` | Invalid arguments: unknown flag, `TwoTools`, `StrayWord`, `SelectorAfterText`, empty query |

---

## 4. Data model changes

Extend `model.rs` and the generated tables (names below are suggestions, so adapt to the real code):

```rust
pub struct Category { pub name: &'static str, pub tools: &'static [Tool] }

pub struct Tool {
    pub slug: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub aliases: &'static [&'static str],      // tool-level, from "Also matches"; all are used by the matcher
    pub flag_aliases: &'static [&'static str], // the subset that works as `-flag` (Section 5.8)
    pub platforms: &'static [Os],              // empty = all
    pub notes: &'static [&'static str],
    pub groups: &'static [Group],
}
pub struct Group   { pub title: &'static str, pub sections: &'static [Section] }
pub struct Section { /* as in v1: title, aliases, commands */ }
```

Helpers:
- `Tool::available_on(os)`: true if `platforms` is empty or contains `os`, **and** at least one section is visible on `os`.
- `Tool::matches_flag(name)`: case-insensitive match against `slug` or any entry of `flag_aliases`.
- `Section::visible_commands(os)`: commands that resolve for `os`, in order.
- `Section::available_on(os)`: `visible_commands(os)` is non-empty.
- `TOOLS` becomes `CATEGORIES` (a `&[Category]`) plus a flat iterator, `all_tools()`, so the matcher can keep working over tools.

**Visibility rule (changes v1):** a command that doesn't resolve for the OS is **omitted**. The v1 `(no equivalent)` placeholder line is removed everywhere. A section with no visible commands is hidden. A tool with no visible sections is hidden. This keeps Linux users from seeing Windows-only lines, and vice versa, in sections like "Cache & Clean Reinstall".

---

## 5. Data format (TOML)

### 5.1 One file per tool, flat `data/` folder

```
data/
  categories.toml        # category order (5.4)
  git.toml               # v1 file, migrated (5.5)
  shell.toml             # v1 file, migrated (5.5)
  npm.toml
  docker.toml
  flutter.toml
  ...
```

The file name matches the tool slug.

### 5.2 Tool file layout

```toml
tool       = "docker"
name       = "Docker"
category   = "Containers & DevOps"
order      = 1                              # position inside the category (optional)
aliases    = ["container", "containers", "dockerfile"]
platforms  = ["windows", "linux", "macos"]  # optional, default: all
notes      = ["Optional tool-wide note"]    # optional
draft      = false                          # optional, see 5.6

[[group]]
title = "Containers"

  [[group.section]]
  title   = "Run a Container"
  aliases = ["start container", "docker run", "launch container"]

    [[group.section.command]]
    cmd  = "docker run <image>"
    note = "Runs a container from an image"

    [[group.section.command]]
    note    = "Removes a folder and everything inside it"
    windows = "rmdir /s /q <folder>"
    unix    = "rm -rf <folder>"
```

OS keys and precedence are unchanged from v1: exact OS (`windows`, `linux`, `macos`), then `unix`, then `cmd`.

### 5.3 Backward compatibility

v1 files use top-level `[[section]]` with no groups. `build.rs` must still accept them. Treat top-level sections as one implicit group with the same title as the tool's name. New and migrated files use `[[group]]`.

### 5.4 `data/categories.toml`

```toml
order = [
  "Version Control",
  "JavaScript & Web",
  "Mobile Development",
  "Containers & DevOps",
  "Cloud & Backend Services",
  "Languages & Package Tools",
  "Databases",
  "Linux & System Tools",
  "Package Managers",
]
```

Categories not listed come last, alphabetically, with a build warning. This is the only place the order lives, so it is easy to change.

### 5.5 Migrating the v1 files

- `git.toml`: add `name = "Git"` and `category = "Version Control"`. Keep every existing section and command exactly as they are. Add the new Git sections from `VERSION2_COMMANDS.md` ("Git (additions to v1)") as extra groups in the same file.
- `shell.toml`: add `name = "Shell Basics (CMD & Terminal)"` and `category = "Linux & System Tools"`, keeping slug `shell`.
- Add tool-level aliases where useful (`git`: `rebase`, `submodule`, `worktree`, `reflog`, `git-advanced`).

### 5.6 `draft` tools

A tool with `draft = true` is **skipped by the build** with a printed warning, so the library can grow one tool at a time without failing the build over missing aliases. A test (Section 11) fails if any tool is still draft at release time.

### 5.7 `build.rs` validation (extends v1 Section 5.3)

Fail the build, naming the file and section, when:
- A non-draft tool is missing `tool`, `name`, or `category`.
- Tool slugs and tool aliases share one namespace. Fail when a slug or alias is used by two tools, when an alias equals another tool's slug, or when either equals an OS name (`windows`, `linux`, `macos`). The shipped v1 tools take part in this check. (That is why Bash does not list `shell` as an alias: the v1 `shell` tool owns that slug. For the same reason Vim and nano do not list `editor`, which both would otherwise claim.)
- A category is empty or not a string.
- A group has no sections.
- Any v1 rule is broken (section title unique per tool, at least 3 section aliases, command note up to 60 characters with no trailing period, and so on. Tool-level `notes` are exempt from the 60-character limit). The one exception is the v1 rule "`<` and `>` are balanced", which is replaced by the placeholder rule below.
- A section has fewer than 2 or more than 12 command **entries**. An entry is one `[[command]]` block, so an OS pair merged into one entry counts once.
- A tool-level `platforms` value is not one of `windows`, `linux`, `macos`.
- A non-draft tool has a section that is invisible on **every** OS.
- Any title, group title, tool name, category, alias, command, or note contains a non-ASCII character. All output is ASCII.

Warn, but do not fail, when:
- Two sections of one tool share an alias.
- A section has exactly one visible command on an OS where its tool is available.

**Placeholder rule.** Scan each command string left to right:
1. A `<` followed by a letter or `_` starts a placeholder. It must close with `>` before any whitespace.
2. Any other `<`, and any `>` outside a placeholder, must be a shell redirect: ` < `, ` > `, ` >> `, `2>`, or `>&`.
3. Anything else fails the build.

The rule must accept `psql -U <user> <database> < <backup.sql>`, `adb logcat -d > <log.txt>`, and `<command> > <file> 2>&1`. All 1,651 command lines in the library pass it. `render.rs` highlights placeholders with the same scan (v1 Section 9.3), so a redirect is never colored as a placeholder.

`build.rs` generates `CATEGORIES`, sorted by `categories.toml`, then tool `order`, then name.

### 5.8 Flag aliases and shadowing

Every tool alias helps the matcher, but not every alias should work as `ezcmd --help -<alias>`. Some aliases are ordinary words that also appear in section titles of other tools, and `ezcmd --help -cache` should search for cache sections, not open Redis.

`build.rs` marks an alias as **shadowed** when it is a single word that also appears, as a whole word, in a section title of a *different* tool. Shadowed aliases stay in `aliases` (the matcher still sees them) but are left out of `flag_aliases`. Slugs are never shadowed. Print the shadowed list as a build note.

With the current library the shadowed aliases are exactly: `lint` and `format` (eslint), `android` (adb), `pip` (python), `cache` (redis), `users` and `groups` (perms), and `history` (bash). A unit test pins this list, so a content change that shadows or un-shadows an alias is noticed and reviewed.

Users reach those words through section search: `ezcmd --help -cache`, or `ezcmd --help -- cache` to force it.

---

## 6. Rendering changes

### 6.1 Section frame title

Show the tool name in the frame title so the user knows which tool a section came from:

```
 ___[ Flutter: Run the App ]____________________________________________________________
|
|  flutter devices                          --- Lists connected devices and emulators
|  flutter run                              --- Runs the app in debug mode
|  flutter run --profile                    --- Runs in profile mode for performance testing
|  flutter run --release                    --- Runs in release mode
|_______________________________________________________________________________________
```

In `--plain` mode the header line becomes `# ---Flutter: Run the App---`. The v1 layout rules (wide and narrow layouts, frame width, padding before coloring) are unchanged.

### 6.2 TOC rendering

Add pure functions to `render.rs`, writing to `impl Write` like the existing ones:

```rust
pub fn toc_overview(out: &mut impl Write, cats: &[Category], os: Os, ctx: &RenderCtx) -> io::Result<()>;
pub fn toc_tool(out: &mut impl Write, tool: &Tool, os: Os, ctx: &RenderCtx) -> io::Result<()>;
pub fn usage_guide(out: &mut impl Write, ctx: &RenderCtx) -> io::Result<()>;
```

Styles (extends `IMPLEMENTATION.md` Section 9.3):
| Element | Style |
|---|---|
| Category heading | bold cyan |
| `-slug` in the overview | bold |
| Tool name | default |
| Section counts | dim gray |
| Group heading (Level 2) | bold |
| `-Section Title` | the dash in dim gray, the title in default |
| "How to use" headings | bold cyan, like atuin's `Usage:` |
| Example commands in the guide | dim gray |

Layout rules:
- Single column. In the overview, pad the slug column to the longest slug and the name column to the longest name.
- Never wrap or truncate section titles. Titles are short by rule.
- On terminals narrower than 80 columns, drop the example column from the usage guide and keep the rest.
- All output is ASCII.

### 6.3 Chooser labels

`prompt::choose` and the "no match" suggestions show `Tool: Section`:

```
Multiple matches for "create app":
  1. Next.js: Create an App
  2. React Native & Expo: Create an Expo App
  3. Flutter: Create a Project
Choose [1/2/3]:
```

---

## 7. Matching changes for a bigger library

The v1 algorithm (`IMPLEMENTATION.md` Section 8) stays. Add the following, because with about 290 sections many titles repeat across tools ("Clean & Reset", "Run the App", "Install Dependencies").

### 7.1 Tool tokens

Tool names in a query narrow the results. They are handled in two groups, and a guard stops common words from hijacking a query.

1. Tokenize the query as before. Tool tokens need an **exact** match (`tm == 1.0`). Prefix and typo matches never create tool tokens, so "contain" does not turn into "container".
2. Classify each query token:
   - **Named:** equals a tool slug or a flag alias (`docker`, `npm`, `k8s`, `go`, `find`).
   - **Hint:** equals a shadowed alias or a word in a tool's name (`cache`, `history`, `kubernetes`, `react`).
3. **Guard.** Narrowing is tried in two steps, and each step is kept or dropped on its own.
   1. *Named tokens.* Remove them from the content tokens. If no content token remains, go to step 4. Otherwise score the sections on the remaining tokens. Keep the narrowing only if a section of a named tool scores at least `MATCH_THRESHOLD`. When kept, multiply the score of sections from **other tools** by `OTHER_TOOL_FACTOR` and add `TOOL_BONUS` to sections of the named tools. When dropped, put the tokens back as ordinary content, with no factor and no bonus.
   2. *Hint tokens.* Apply the same test to the tokens that are left. When kept, remove them from the content tokens and add `HINT_BONUS` to the owning tool's sections. Hints never demote other tools.
4. **Only tool names.** If removing the named tokens leaves nothing and exactly one tool is named, return `SearchResult::ToolOnly(tool)`. This makes `ezcmd docker` and `ezcmd k8s` behave like `ezcmd --help -docker`. If several tools are named (`docker compose`), prefer the one whose name contains every token (`Docker Compose`). If none does, skip this shortcut and search normally. A shadowed alias alone (`ezcmd cache`) is only a hint, so it never produces `ToolOnly`.
5. When `SearchOpts.tool_tokens` is `false` (every `--help` lookup), skip this whole section and score sections only.

Why the guard matters: `go`, `find`, and `next` are real words as well as tool names. These cases must keep working:

| Query | Expected | What the guard does |
|---|---|---|
| `go back` | Git: Undo & Rollback | `go` is a slug, but no Go section matches `back`, so the narrowing is dropped |
| `next window` | tmux: Windows & Panes (Keys) | `next` is a flag alias of Next.js, which has nothing for `window`, so it is dropped |
| `clear npm cache` | npm: Cache & Clean Reinstall | `npm` narrows; `cache` is a hint for Redis, which has no `clear` section, so it is dropped |
| `find files` | find: Find Files | `find` is a slug and its own section matches `files`, so the narrowing is kept |

### 7.2 Constants and results

```rust
const OTHER_TOOL_FACTOR: f32 = 0.55;
const TOOL_BONUS:        f32 = 0.10;
const HINT_BONUS:        f32 = 0.05;
const MAX_CHOICES:       usize = 5;      // was 3 in v1
const MATCH_THRESHOLD:   f32 = 0.45;     // retune against the v2 test set
const AMBIGUITY_RATIO:   f32 = 0.85;     // retune against the v2 test set

pub struct SearchOpts<'a> { pub only_tool: Option<&'a str>, pub tool_tokens: bool }
pub enum SearchResult<'a> {
    Single(Hit<'a>),
    Ambiguous(Vec<Hit<'a>>),     // 2..=MAX_CHOICES
    NoMatch(Vec<Hit<'a>>),       // up to 3 closest, may be empty
    ToolOnly(&'a Tool),          // new: the query named only a tool
}
pub fn search<'a>(tools: &[&'a Tool], os: Os, query: &str, opts: SearchOpts) -> SearchResult<'a>;
```

`tools` is the flat list from `all_tools()`. Keep all constants together at the top of `matcher.rs`. Tune them against the test set below. Don't weaken the accuracy targets to make tests pass.

### 7.3 Performance

Do not add indexing until you have a measurement. With about 290 sections and 1,600 commands, runtime tokenizing should be well under 1 ms. Measure with `hyperfine` in V0 and V10. If the v2 run is more than 5 ms slower than the baseline, precompute stems for titles and aliases in `build.rs`, and leave commands and notes to be tokenized at runtime.

### 7.4 What does not change

The query mode (`ezcmd <query>`), `--` handling, the `[1/N]` prompt rules, and non-terminal behavior (print the best hit, or the TOC for `ToolOnly`) stay as in v1. Exit codes are in Section 3.9.

---

## 8. Importing `VERSION2_COMMANDS.md`

### 8.1 Source of truth

`VERSION2_COMMANDS.md` is the **source until the import is complete**. After V9, the TOML files become the source of truth. At that point, add a line at the top of `VERSION2_COMMANDS.md` saying it is an archived plan and the `data/` folder is authoritative.

### 8.2 Write a converter (do not hand-copy)

Create `tools/md_to_toml.py` (Python 3, standard library only, not shipped). It reads `VERSION2_COMMANDS.md` and writes one TOML file per tool into `data/`. It must be re-runnable and deterministic. **Parse only the lines between the `## 4. Command library` heading and the `## 5. Open items` heading.** Everything before and after (the conventions, summary, table of contents, open items, and verification notes) is documentation. Its `##` and `###` headings are not categories or tools.

Parsing rules, based on the library file's structure:

| Markdown | Becomes |
|---|---|
| `## <Category>` heading (in the library part of the file) | `category` of the tools below it |
| `### <Tool name>` | `name` |
| `**Flag:** \`-slug\`  \|  **Also matches:** a, b  \|  **Platforms:** ...` | `tool`, `aliases`, `platforms` |
| `> **Note:** text` | one entry in `notes` |
| `#### <Group>` | `[[group]]` |
| A fenced block with `# ---Title---` headers | one `[[group.section]]` per header |
| `command --- note` line | one `[[group.section.command]]` |
| `[win]` / `[linux]` / `[mac]` / `[unix]` prefix | `windows` / `linux` / `macos` / `unix` key |
| No prefix | `cmd` key |

Details to get right:
- Split each line at the **first** `---` that has at least two spaces before it (commands can contain `--`, such as `--release`).
- **Merge** consecutive tagged lines that have the **same note** into one command entry (for example a `[unix]` and `[win]` pair). Lines with different notes stay separate entries.
- Map platform text: `Windows, Linux, macOS` means all (omit the field), `Linux, macOS` means `["linux","macos"]`, `Linux` means `["linux"]`, `macOS` means `["macos"]`, `Windows` means `["windows"]`.
- Assign `order` from the tool's position within its category.
- Write `aliases = []` for sections and set `draft = true`. The agent fills the aliases in V8b and removes `draft` as each tool is completed.
- Escape TOML strings correctly (use literal strings or escape backslashes, since commands contain `\`, `"`, and `$`).
- **Git additions:** do not write them to `data/`. Write the groups of the "Git (additions to v1)" tool to `tools/generated/git_additions.toml` (groups only, no tool header). V8i appends that file to `data/git.toml` (5.5).
- **Fail loudly:** stop with the line number if a line inside a fenced block has no `---` separator with two or more spaces before it, or if any line contains a non-ASCII character.

### 8.3 Converter test

Add a check (script or test) that converts the markdown, regenerates the commands from the TOML, and compares them with the original: **every section title, command, note, and OS tag must round-trip exactly.** The totals must match the summary table at the top of the library file: **62 tools** (61 tool files plus the Git additions), **286 sections**, and **1,651 command lines**, which become **1,639 entries** after OS pairs are merged (12 pairs). The converter's printed numbers are authoritative if they differ. Print both numbers.

---

## 9. Writing aliases

Every section needs **at least 3 aliases**. Aliases are the phrases people type, and they decide whether the tool feels smart.

### 9.1 Rules
- Lowercase, no punctuation, 1 to 5 words.
- Write them the way a beginner would phrase the task, not the way the docs do.
- Cover: a verb phrase ("start container"), the common noun ("container run"), and a synonym or shorthand ("docker run").
- Don't include the tool's own name unless it's part of the command people know (`docker run` is fine for the Docker run section, `docker` alone is not, because the matcher handles tool names separately).
- Don't repeat the title word for word.
- Don't duplicate an alias between two sections of the same tool.
- Prefer phrases that people would actually search over single common words like "list" or "show".

### 9.2 Examples

| Tool: Section | Good aliases |
|---|---|
| Docker: Run a Container | `start container`, `run image`, `launch container`, `docker run` |
| Flutter: Run the App | `start app`, `launch app`, `run on device`, `run emulator` |
| Node.js & npm: Install Dependencies | `install packages`, `npm install`, `add dependencies`, `fresh install` |
| Git: Undo & Rollback | `undo commit`, `revert changes`, `discard changes`, `go back` |

### 9.3 Order of work

For each tool: run the converter, write the aliases, remove `draft`, build, add its test queries (Section 10), then run the accuracy test. Commit per tool or per category.

---

## 10. Accuracy testing

v1 used about 100 queries. v2 needs a larger, structured set in `tests/queries_v2.toml`, with at least **5 queries per tool** (about 300 or more in total):

```toml
[[q]]
kind   = "tool"                     # tool-qualified
query  = "docker logs follow"
tool   = "docker"
expect = "Inspect & Debug"

[[q]]
kind   = "task"                     # no tool named
query  = "undo last commit"
tool   = "git"
expect = "Undo & Rollback"

[[q]]
kind   = "ambiguous"                # should offer a chooser containing the right section
query  = "create app"
choices = ["nextjs", "expo", "flutter"]

[[q]]
kind   = "none"                     # should not match anything
query  = "bake a chocolate cake"

[[q]]
kind   = "guard"                    # a tool name that is also an ordinary word (Section 7.1)
query  = "go back"
tool   = "git"
expect = "Undo & Rollback"
```

Write at least these four `guard` queries: `go back`, `next window`, `clear npm cache`, `find files` (expected sections are in the Section 7.1 table).

Test each query on all three OSes unless it has an `os` field. Gates, enforced from V8 onward:

| Kind | Requirement |
|---|---|
| `tool` | Top-1 is the expected section for **at least 95%** of queries |
| `task` | Expected section appears in the top 3 (single hit, or inside a chooser) for **at least 90%** |
| `ambiguous` | The chooser appears and contains every listed tool, **at least 90%** |
| `none` | No match for **100%** |
| `guard` | Top-1 is the expected section for **100%** |

When the test fails, print each failing query with the actual ranking and scores. Fix with aliases or constants before touching thresholds.

Also keep the v1 query file. Its gates (top-1 at least 90%, top-3 at least 98%) must still pass on the v1 queries.

---

## 11. Testing plan

| Layer | What | How |
|---|---|---|
| **Parser** | Every row in Section 3.6, the error cases, and combinations with `--plain` | Unit tests in `cli.rs` |
| **Model** | OS visibility (commands, sections, tools hidden when they don't resolve), `matches_flag`, tool availability | Unit tests with explicit `Os` values |
| **Build script** | Duplicate slugs and aliases, OS-name collisions, missing category, draft skipping, v1-style file still builds, bad platform value, placeholder rule (redirects pass, an unclosed `<` fails), non-ASCII text, shadowed aliases (5.8) | Small fixture files under `tests/fixtures/` run through the same validation function (put it in a shared module, not only in `build.rs`) |
| **TOC rendering** | Level 1 and Level 2 snapshots, single-group tool (no group heading), OS filtering (for example CocoaPods hidden on Linux), `--plain`, narrow width | `insta` snapshots with fixed `RenderCtx` |
| **Section rendering** | Frame title shows `Tool: Section`, the omit-instead-of-"(no equivalent)" rule, tagged pairs per OS | `insta` snapshots |
| **Prompt** | Labels include tool names, up to 5 choices | Existing `Cursor` approach |
| **Content sanity** (`tests/content.rs`) | At least 63 tools (61 from the library plus `git` and `shell`), 307 sections (286 from the library plus the v1 minimums of 12 Git and 9 shell), and 1,639 command entries after V8; the shadowed-alias list in 5.8 is unchanged; every section has 3 or more aliases; no draft tools; every non-draft tool is visible on at least one OS; every tool slug and alias unique; `-git` Initializing a Repo section unchanged from v1 | Plain integration test |
| **Accuracy** | Section 10 | `tests/accuracy.rs` |
| **CLI end to end** | `--help` output contains category names and **no command text**; `--help -docker` lists `-Run a Container` and no commands; `--help -docker -run` prints a section; `--help -initialize` goes to a chooser or a section; exit codes; broken pipe; non-terminal behavior; `--help -linux -pods` exits 1 with the platforms message; `--help -history` runs a section search; `ezcmd docker` prints the Docker TOC | `assert_cmd` |
| **Regression** | All v1 tests still pass, except those for the old full-dump help, which are rewritten (see V4) | CI |

---

## 12. Milestones

Complete in order. Each ends with the Section 13 checks.

### V0: Baseline and review
- Check out the current code. Run all tests and record: release binary size, `hyperfine` startup time (for `ezcmd initializing repo` and `ezcmd --help`), and test results.
- List how the shipped code differs from `IMPLEMENTATION.md` (module names, flags, content files).
- List every tool slug and alias that already exists (for example `-shell`), so the new library cannot collide with it.
- Write the findings in a short `V2_BASELINE.md` (not shipped) and stop for approval.
- **Done when:** baseline numbers and the difference list are reported.

### V1: Data model and `build.rs`
- Add categories, groups, tool metadata, aliases, platforms, notes, `draft`, and `categories.toml`.
- Keep v1-style files working (5.3). Migrate `git.toml` and `shell.toml` (5.5).
- Add the validations in 5.7 and the shadowed-alias computation in 5.8, with the logic in a shared module that tests can call.
- **Done when:** the build produces `CATEGORIES`, all v1 tests pass, and every build-script rule has a test.

### V2: Visibility and tool lookup
- Implement `available_on`, `visible_commands`, `matches_flag`, and the "omit, don't show `(no equivalent)`" rule.
- **Done when:** model unit tests pass and v1 query and help outputs are unchanged apart from the removed placeholder.

### V3: CLI parser
- New help grammar (3.6), `CliError::TwoTools` and `StrayWord`, section-text forms, and `--` handling.
- **Done when:** every row of the 3.6 table has a passing test.

### V4: TOC rendering
- `usage_guide`, `toc_overview`, `toc_tool`, snapshots, `--plain`, narrow terminals.
- Replace the old full-dump `--help` behavior. Rewrite the tests and snapshots that expected a dump.
- **Done when:** `ezcmd --help` and `ezcmd --help -git` print only names, with no commands, and the snapshots are reviewed.

### V5: Section lookup through help
- Level 3 (`-<tool> -<section>`), the cross-tool section search, `--` forcing, chooser integration, and exit codes.
- **Done when:** all the Section 3.4 behaviors work end to end, including the chooser labels from 6.3.

### V6: Matcher and frame title
- Named and hint tool tokens with the guard (7.1), `SearchResult::ToolOnly`, `SearchOpts`, `MAX_CHOICES`, retuned constants, and the `Tool: Section` frame title in `render.rs`.
- Seed `tests/queries_v2.toml` with 30 queries (the first few tools) and make `tests/accuracy.rs` read it.
- **Done when:** the seed set runs, the four guard queries from Section 10 pass, v1 accuracy still passes, and `ezcmd docker` shows the Docker TOC.

### V7: Converter
- Write `tools/md_to_toml.py` (Section 8.2) and the round-trip check (8.3).
- Run it to generate all tool files as `draft = true`, plus `tools/generated/git_additions.toml`.
- **Done when:** the round-trip check passes and the generated counts match the library summary.

### V8: Content, one category at a time
Do each category as its own sub-milestone. For each tool: write aliases (Section 9), remove `draft`, add at least 5 queries (Section 10), run the build and accuracy test.

- **V8a:** JavaScript & Web
- **V8b:** Mobile Development
- **V8c:** Containers & DevOps
- **V8d:** Cloud & Backend Services
- **V8e:** Languages & Package Tools
- **V8f:** Databases
- **V8g:** Linux & System Tools
- **V8h:** Package Managers
- **V8i:** Version Control (append `tools/generated/git_additions.toml` to `git.toml`, then write its aliases)

**Done when (each):** the category's tools are non-draft, aliases validate, the accuracy gates for the queries so far pass, and I have approved the report.

### V9: Documentation and version
- Update `README.md`: the usage table and examples for the new help behavior, the feature list, and a short description of tool aliases. Remove any statement that `--help` prints all commands.
- Bump the version in `Cargo.toml` (2.0.0 suggested, see Section 14) and the `ezcmd --version` expectations, badges, and the install-script fallback URL.
- Add an archive note to `VERSION2_COMMANDS.md` (8.1).
- **Done when:** README examples are verified by running them, and docs match the behavior.

### V10: Performance, size, and release
- Run the full verification table in Section 13 and compare with the V0 baseline.
- Run the release workflow on a test tag if you have one.
- **Done when:** every target is met, or you've explained each miss.

---

## 13. Verification & definition of done

Run after every milestone:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
```

Run at V10 and before any release:

| Check | Command | Target |
|---|---|---|
| Startup (query) | `hyperfine --warmup 5 'target/release/ezcmd initializing repo'` | **< 20 ms** mean |
| Startup (help) | `hyperfine --warmup 5 'target/release/ezcmd --help'` | **< 20 ms** mean |
| Binary size | `ls -l target/release/ezcmd` | **< 3 MB** (aim under 1.5 MB) |
| Peak memory | `/usr/bin/time -v target/release/ezcmd --help` (Linux) | **< 10 MB** |
| Help shape | `ezcmd --help \| grep -cE "docker run\|git commit\|npm install"` | **0** command lines in the overview |
| Level 2 shape | `ezcmd --help -docker \| grep -c "docker run"` | **0** (section titles only) |
| Accuracy | `cargo test --test accuracy` | The Section 10 gates |
| Content | `cargo test --test content` | Counts and rules in Section 11 |
| No draft tools | Content test | **0** |
| Cross-platform | CI green on Windows, Linux, macOS | All pass |
| Dependencies | `cargo tree` | **No new runtime dependencies** |

**v2 is done when** every milestone's "Done when" has passed, the table above is green, and `README.md` matches the real behavior.

---

## 14. Decisions that extend or change earlier documents

If you disagree with any of these, change both this file and the document it affects.

| # | Decision | Changes |
|---|---|---|
| 1 | `ezcmd --help` no longer prints commands. It prints the guide and the categories and tools | PRD FR-7 and `IMPLEMENTATION.md` 9.5 |
| 2 | Commands that don't resolve on the OS are **omitted**. The `(no equivalent)` line is removed | PRD FR-5, FR-8, and `IMPLEMENTATION.md` 9.4 |
| 3 | An unknown dash word after `--help` starts a section search instead of being an error | `IMPLEMENTATION.md` 6.2 |
| 4 | A section search with no match exits with `1`, not `2` | `IMPLEMENTATION.md` 7 |
| 5 | Tools have tool-level aliases, which are also valid `-flags` | New |
| 6 | Group headings are hidden for tools with a single group | New |
| 7 | The section frame title is `Tool: Section` | `IMPLEMENTATION.md` 9.1 |
| 8 | `MAX_CHOICES` goes from 3 to 5 | `IMPLEMENTATION.md` 8.3 |
| 9 | Tool names in a query narrow results and the tool-only query (`ezcmd docker`) shows that tool's TOC | `IMPLEMENTATION.md` 8 |
| 10 | Data files stay flat in `data/`, with category order in `categories.toml` | `IMPLEMENTATION.md` 5 |
| 11 | `draft = true` tools are skipped by the build until their aliases are written | New |
| 12 | The version is suggested as **2.0.0** (the project's earlier releases were 0.x). Use 0.3.0 if you prefer to stay on 0.x | `Cargo.toml`, README |
| 13 | A tool alias works as a `-flag` only when no other tool's section title uses the same word (shadowing, 5.8) | New |
| 14 | Query tool tokens come in two groups, named and hint, with a guard that drops the narrowing when the named tool cannot answer the rest of the query (7.1) | `IMPLEMENTATION.md` 8 |
| 15 | The v1 rule "`<` and `>` are balanced" is replaced by a placeholder rule that allows shell redirects (5.7) | `IMPLEMENTATION.md` 5.3 |
| 16 | A tool named by flag but hidden on the chosen OS prints a platforms message and exits `1` (3.5) | New |
| 17 | `build.rs` rejects non-ASCII text (5.7) | `IMPLEMENTATION.md` 5.3 |
| 18 | The Git additions are generated to `tools/generated/git_additions.toml` and appended to `git.toml` in V8i (8.2) | New |
| 19 | A selector after the section text is an error, `SelectorAfterText` (3.6) | New |

### Open questions
1. **Full dump:** Should a way to print every command remain (for example `ezcmd --help --all`, useful for piping to `grep`)? Default for now: no.
2. **Numbering:** Should TOC section lines be numbered so users can pick by number? Default for now: no.
3. **Multi-column overview:** Should wide terminals show the tool list in two columns? Default for now: no.

---

## 15. Prompts to give the AI agent

### Kickoff prompt
```
You are implementing ezCMD version 2, on top of the shipped v1 code.

1. Read PRD.md, README.md, IMPLEMENTATION.md, VERSION2_COMMANDS.md, and
   IMPLEMENTATIONV2.md completely before writing code.
2. Inspect the real repository. Where the code differs from the guides, follow
   the code and tell me about the difference.
3. Follow IMPLEMENTATIONV2.md exactly. All v1 hard constraints still apply.
   VERSION2_COMMANDS.md has been verified against current documentation, so
   import it as written.
4. Work one milestone at a time, starting with V0. Do not begin the next
   milestone until I approve.
5. After each milestone, run the Section 13 checks and report:
   - what you built,
   - which tests you added and their results,
   - any deviation from the guide and why.
6. Do not add features outside the guide. If something is ambiguous, ask me.

Begin with V0.
```

### Per-milestone prompt
```
Implement milestone V<N> from IMPLEMENTATIONV2.md Section 12.
Re-read the sections it references first. Stop when its "Done when" list
passes and report using the format from the kickoff prompt.
```

### Converter prompt (V7)
```
Write tools/md_to_toml.py following IMPLEMENTATIONV2.md Section 8.2 and add the
round-trip check from Section 8.3. It must be deterministic and re-runnable, use
only the Python standard library, and stop with the line number on any line it
cannot parse. Report the generated counts next to the summary table in
VERSION2_COMMANDS.md, and show me the OS pairs it merged.
```

### Content prompt (V8)
```
Do milestone V8<letter> from IMPLEMENTATIONV2.md for the tools in that category.
For each tool: write at least 3 aliases per section following Section 9, remove
draft = true, add at least 5 queries to tests/queries_v2.toml following
Section 10, then run the build and the accuracy test. Show me the aliases you
wrote and any query that fails. Do not change a command or note unless it is
wrong, and tell me about each change.
```
