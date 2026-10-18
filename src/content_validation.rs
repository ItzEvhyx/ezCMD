//! Build-time content validation shared by `build.rs` and unit tests.

use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub source: String,
    pub slug: String,
    pub name: String,
    pub category: String,
    pub aliases: Vec<String>,
    pub platforms: Vec<String>,
    pub notes: Vec<String>,
    pub groups: Vec<GroupSpec>,
}

#[derive(Debug, Clone)]
pub struct GroupSpec {
    pub title: String,
    pub sections: Vec<SectionSpec>,
}

#[derive(Debug, Clone)]
pub struct SectionSpec {
    pub title: String,
    pub aliases: Vec<String>,
    pub commands: Vec<CommandSpec>,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub note: String,
    pub cmd: Option<String>,
    pub unix: Option<String>,
    pub windows: Option<String>,
    pub powershell: Option<String>,
    pub linux: Option<String>,
    pub macos: Option<String>,
}

const OS_NAMES: &[&str] = &["windows", "linux", "macos"];

pub fn validate_tool(tool: &ToolSpec) -> Result<(), String> {
    require_ascii(&tool.source, "source")?;
    if tool.slug.is_empty() || tool.slug.trim() != tool.slug {
        return Err(format!("{}: tool slug cannot be empty", tool.source));
    }
    if tool.slug != tool.slug.to_ascii_lowercase() || tool.slug.contains(char::is_whitespace) {
        return Err(format!(
            "{}: tool slug {:?} must be lowercase with no spaces",
            tool.source, tool.slug
        ));
    }
    if OS_NAMES.contains(&tool.slug.as_str()) {
        return Err(format!(
            "{}: tool slug {:?} conflicts with an OS name",
            tool.source, tool.slug
        ));
    }
    for (label, value) in [("tool name", &tool.name), ("category", &tool.category)] {
        if value.trim().is_empty() {
            return Err(format!("{}: {label} cannot be empty", tool.source));
        }
        require_ascii(value, label)?;
    }
    for alias in &tool.aliases {
        if alias.trim().is_empty() || alias != &alias.to_ascii_lowercase() {
            return Err(format!(
                "{}: tool alias {:?} must be non-empty and lowercase",
                tool.source, alias
            ));
        }
        require_ascii(alias, "tool alias")?;
    }
    for platform in &tool.platforms {
        if !OS_NAMES.contains(&platform.as_str()) {
            return Err(format!("{}: unknown platform {:?}", tool.source, platform));
        }
    }
    for note in &tool.notes {
        require_ascii(note, "tool note")?;
    }
    if tool.groups.is_empty() {
        return Err(format!(
            "{}: tool must have at least one group",
            tool.source
        ));
    }

    let mut titles = HashSet::new();
    let mut visible_anywhere = false;
    for group in &tool.groups {
        if group.title.trim().is_empty() {
            return Err(format!("{}: group title cannot be empty", tool.source));
        }
        require_ascii(&group.title, "group title")?;
        if group.sections.is_empty() {
            return Err(format!(
                "{}: group {:?} has no sections",
                tool.source, group.title
            ));
        }
        for section in &group.sections {
            if section.title.trim().is_empty() || !titles.insert(section.title.as_str()) {
                return Err(format!(
                    "{}: section {:?} is empty or duplicated",
                    tool.source, section.title
                ));
            }
            require_ascii(&section.title, "section title")?;
            if section.aliases.len() < 3 {
                return Err(format!(
                    "{}: section {:?} needs at least 3 aliases",
                    tool.source, section.title
                ));
            }
            for alias in &section.aliases {
                if alias.trim().is_empty() || alias != &alias.to_ascii_lowercase() {
                    return Err(format!(
                        "{}: section {:?}: invalid alias {:?}",
                        tool.source, section.title, alias
                    ));
                }
                require_ascii(alias, "section alias")?;
            }
            if !(2..=12).contains(&section.commands.len()) {
                return Err(format!(
                    "{}: section {:?} needs 2 to 12 command entries",
                    tool.source, section.title
                ));
            }
            for command in &section.commands {
                if command.note.trim().is_empty()
                    || command.note.chars().count() > 60
                    || command.note.ends_with('.')
                {
                    return Err(format!(
                        "{}: section {:?}: invalid command note {:?}",
                        tool.source, section.title, command.note
                    ));
                }
                require_ascii(&command.note, "command note")?;
                let values = [
                    &command.cmd,
                    &command.unix,
                    &command.windows,
                    &command.powershell,
                    &command.linux,
                    &command.macos,
                ];
                if !values.iter().any(|value| value.is_some()) {
                    return Err(format!(
                        "{}: section {:?}: command has no OS value",
                        tool.source, section.title
                    ));
                }
                for value in values.into_iter().flatten() {
                    if value.contains(['\n', '\r', '\t']) || !validate_placeholders(value) {
                        return Err(format!(
                            "{}: section {:?}: invalid command {:?}",
                            tool.source, section.title, value
                        ));
                    }
                    require_ascii(value, "command")?;
                }
            }
            visible_anywhere |= section_visible_anywhere(section);
        }
    }
    if !visible_anywhere {
        return Err(format!(
            "{}: no section is visible on a supported OS",
            tool.source
        ));
    }
    Ok(())
}

pub fn validate_namespace(tools: &[ToolSpec]) -> Result<(), String> {
    let mut names = HashSet::new();
    for tool in tools {
        for name in std::iter::once(&tool.slug).chain(tool.aliases.iter()) {
            if OS_NAMES.contains(&name.as_str()) {
                return Err(format!(
                    "{}: {:?} conflicts with an OS name",
                    tool.source, name
                ));
            }
            if !names.insert(name.as_str()) {
                return Err(format!(
                    "{}: duplicate tool slug or alias {:?}",
                    tool.source, name
                ));
            }
        }
    }
    Ok(())
}

pub fn shadowed_aliases(tools: &[ToolSpec]) -> Vec<(String, String)> {
    let mut shadowed = Vec::new();
    for tool in tools {
        for alias in &tool.aliases {
            if alias.split_whitespace().count() != 1 {
                continue;
            }
            let used_elsewhere =
                tools
                    .iter()
                    .filter(|other| other.slug != tool.slug)
                    .any(|other| {
                        other
                            .groups
                            .iter()
                            .flat_map(|group| group.sections.iter())
                            .any(|section| {
                                section
                                    .title
                                    .split(|c: char| !c.is_ascii_alphanumeric())
                                    .any(|word| word.eq_ignore_ascii_case(alias))
                            })
                    });
            if used_elsewhere {
                shadowed.push((tool.slug.clone(), alias.clone()));
            }
        }
    }
    shadowed.sort();
    shadowed
}

pub fn duplicate_section_aliases(tool: &ToolSpec) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();
    for alias in tool
        .groups
        .iter()
        .flat_map(|group| group.sections.iter())
        .flat_map(|section| section.aliases.iter())
    {
        if !seen.insert(alias.as_str()) {
            duplicates.push(alias.clone());
        }
    }
    duplicates.sort();
    duplicates.dedup();
    duplicates
}

pub fn validate_placeholders(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'<' if index + 1 < bytes.len()
                && (bytes[index + 1].is_ascii_alphabetic() || bytes[index + 1] == b'_') =>
            {
                index += 2;
                while index < bytes.len()
                    && bytes[index] != b'>'
                    && !bytes[index].is_ascii_whitespace()
                {
                    index += 1;
                }
                if index == bytes.len() || bytes[index] != b'>' {
                    return false;
                }
            }
            b'<' if is_redirect_left(bytes, index) => {}
            b'>' if is_redirect_right(bytes, index) => {}
            b'<' | b'>' => return false,
            _ => {}
        }
        index += 1;
    }
    true
}

fn section_visible_anywhere(section: &SectionSpec) -> bool {
    section.commands.iter().any(|command| {
        command.cmd.is_some()
            || command.windows.is_some()
            || command.powershell.is_some()
            || command.unix.is_some()
            || command.linux.is_some()
            || command.macos.is_some()
    })
}

fn is_redirect_left(bytes: &[u8], index: usize) -> bool {
    index > 0
        && index + 1 < bytes.len()
        && bytes[index - 1].is_ascii_whitespace()
        && bytes[index + 1].is_ascii_whitespace()
}

fn is_redirect_right(bytes: &[u8], index: usize) -> bool {
    let previous = index.checked_sub(1).and_then(|i| bytes.get(i)).copied();
    let next = bytes.get(index + 1).copied();
    matches!(previous, Some(b'0'..=b'9'))
        || matches!(previous, Some(b'>'))
        || matches!(previous, Some(b'&'))
        || (matches!(previous, Some(byte) if byte.is_ascii_whitespace())
            && matches!(next, Some(byte) if byte.is_ascii_whitespace() || byte == b'>' || byte == b'&'))
}

fn require_ascii(value: &str, label: &str) -> Result<(), String> {
    if value.is_ascii() {
        Ok(())
    } else {
        Err(format!("{label} contains non-ASCII text"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_tool() -> ToolSpec {
        ToolSpec {
            source: "fixture.toml".into(),
            slug: "fixture".into(),
            name: "Fixture".into(),
            category: "Tests".into(),
            aliases: vec!["fixture-cli".into()],
            platforms: vec![],
            notes: vec![],
            groups: vec![GroupSpec {
                title: "Everything Fixture".into(),
                sections: vec![SectionSpec {
                    title: "Run Fixture".into(),
                    aliases: vec![
                        "run fixture".into(),
                        "start fixture".into(),
                        "fixture run".into(),
                    ],
                    commands: vec![
                        CommandSpec {
                            note: "Runs the fixture".into(),
                            cmd: Some("fixture run <file>".into()),
                            unix: None,
                            windows: None,
                            powershell: None,
                            linux: None,
                            macos: None,
                        },
                        CommandSpec {
                            note: "Checks the fixture".into(),
                            cmd: Some("fixture check".into()),
                            unix: None,
                            windows: None,
                            powershell: None,
                            linux: None,
                            macos: None,
                        },
                    ],
                }],
            }],
        }
    }

    #[test]
    fn placeholders_allow_redirects_but_reject_malformed_values() {
        assert!(validate_placeholders(
            "psql -U <user> <database> < <backup.sql>"
        ));
        assert!(validate_placeholders("adb logcat -d > <log.txt>"));
        assert!(validate_placeholders("<command> > <file> 2>&1"));
        assert!(!validate_placeholders("echo <unclosed"));
        assert!(!validate_placeholders("echo >broken"));
    }

    #[test]
    fn validation_rejects_invalid_schema_values() {
        let mut tool = valid_tool();
        assert!(validate_tool(&tool).is_ok());

        tool.platforms.push("bsd".into());
        assert!(validate_tool(&tool).is_err());
        tool.platforms.clear();

        tool.groups[0].sections[0].aliases.truncate(2);
        assert!(validate_tool(&tool).is_err());
        tool.groups[0].sections[0]
            .aliases
            .push("fixture execute".into());

        tool.groups[0].sections[0].commands.truncate(1);
        assert!(validate_tool(&tool).is_err());
    }

    #[test]
    fn namespace_and_shadowing_rules_are_explicit() {
        let first = valid_tool();
        let mut second = valid_tool();
        second.slug = "other".into();
        second.aliases = vec!["fixture".into()];
        assert!(validate_namespace(&[first.clone(), second]).is_err());

        let mut other = valid_tool();
        other.slug = "other".into();
        other.aliases = vec!["run".into()];
        other.groups[0].sections[0].title = "Check Results".into();
        let shadowed = shadowed_aliases(&[first, other]);
        assert_eq!(shadowed, vec![("other".into(), "run".into())]);
    }
}
