use ezcmd::content::{all_tools, CATEGORIES};
use ezcmd::os::Os;
use std::collections::HashSet;

#[test]
fn test_categories_and_tools_exist() {
    assert!(!CATEGORIES.is_empty(), "CATEGORIES must not be empty");
    assert_eq!(CATEGORIES[0].name, "Version Control");
    assert!(all_tools().count() >= 2, "seed tools must be present");
}

#[test]
fn test_seed_tools_present() {
    let names: Vec<&str> = all_tools().map(|tool| tool.slug).collect();
    assert!(names.contains(&"git"), "git tool missing");
    assert!(names.contains(&"shell"), "shell tool missing");
}

#[test]
fn test_javascript_web_v8a_tools_are_complete() {
    let category = CATEGORIES
        .iter()
        .find(|category| category.name == "JavaScript & Web")
        .expect("JavaScript & Web category present");
    let expected = [
        "npm",
        "pnpm",
        "yarn",
        "bun",
        "nextjs",
        "vite",
        "typescript",
        "eslint",
        "vercel",
        "netlify",
    ];
    let actual: Vec<&str> = category.tools.iter().map(|tool| tool.slug).collect();
    assert_eq!(actual, expected);
    assert_eq!(
        category
            .tools
            .iter()
            .map(|tool| tool.sections().count())
            .sum::<usize>(),
        47
    );
}

#[test]
fn test_minimum_section_counts() {
    let git = all_tools()
        .find(|tool| tool.slug == "git")
        .expect("git found");
    assert!(
        git.sections().count() >= 12,
        "git must have >= 12 sections, found {}",
        git.sections().count()
    );

    let shell = all_tools()
        .find(|tool| tool.slug == "shell")
        .expect("shell found");
    assert!(
        shell.sections().count() >= 9,
        "shell must have >= 9 sections, found {}",
        shell.sections().count()
    );
}

#[test]
fn test_section_rules() {
    for tool in all_tools() {
        let mut seen_titles = HashSet::new();
        let mut seen_aliases = HashSet::new();

        for sec in tool.sections() {
            assert!(
                seen_titles.insert(sec.title),
                "duplicate section title: {}",
                sec.title
            );
            assert!(
                sec.aliases.len() >= 3,
                "section {} must have >= 3 aliases",
                sec.title
            );

            for &alias in sec.aliases {
                assert_eq!(
                    alias,
                    alias.to_lowercase(),
                    "alias {} must be lowercase",
                    alias
                );
                assert!(
                    seen_aliases.insert(alias),
                    "duplicate alias {} in tool {}",
                    alias,
                    tool.slug
                );
            }

            assert!(
                !sec.commands.is_empty(),
                "section {} has no commands",
                sec.title
            );

            for cmd in sec.commands {
                assert!(
                    !cmd.note.is_empty() && cmd.note.chars().count() <= 60,
                    "invalid note length in {}: {}",
                    sec.title,
                    cmd.note
                );
                assert!(
                    !cmd.note.ends_with('.'),
                    "note ends with period in {}: {}",
                    sec.title,
                    cmd.note
                );
            }

            // Every seed section is available on all three OSes
            for os in Os::ALL {
                assert!(
                    sec.available_on(os),
                    "section {} not available on {:?}",
                    sec.title,
                    os
                );
            }
        }
    }
}

#[test]
fn test_initializing_repo_section() {
    let git = all_tools()
        .find(|tool| tool.slug == "git")
        .expect("git found");
    let init_sec = git
        .sections()
        .find(|s| s.title == "Initializing a Repo")
        .expect("Initializing a Repo section found");

    let expected_cmds = [
        ("git init", "Initializes a repo in the current folder"),
        ("git add .", "Stages all new and changed files"),
        (
            "git commit -m \"Initial commit\"",
            "Saves the staged files as the first commit",
        ),
        ("git branch -M main", "Renames the current branch to main"),
        (
            "git remote add origin <repo-url>",
            "Links the repo to a remote on GitHub",
        ),
        (
            "git push -u origin main",
            "Uploads main and sets it as the upstream",
        ),
    ];

    assert_eq!(init_sec.commands.len(), expected_cmds.len());
    for (i, (expected_cmd, expected_note)) in expected_cmds.iter().enumerate() {
        let cmd = &init_sec.commands[i];
        assert_eq!(cmd.resolve(Os::Linux), Some(*expected_cmd));
        assert_eq!(cmd.resolve(Os::Windows), Some(*expected_cmd));
        assert_eq!(cmd.resolve(Os::Macos), Some(*expected_cmd));
        assert_eq!(cmd.note, *expected_note);
    }
}
