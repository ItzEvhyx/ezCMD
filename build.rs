#[path = "src/content_validation.rs"]
mod content_validation;

use content_validation::{CommandSpec, GroupSpec, SectionSpec, ToolSpec};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
struct RawCategories {
    #[serde(default)]
    order: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct RawToolFile {
    tool: String,
    name: Option<String>,
    category: Option<String>,
    order: Option<u32>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    platforms: Vec<String>,
    #[serde(default)]
    notes: Vec<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    group: Vec<RawGroup>,
    #[serde(default)]
    section: Vec<RawSection>,
}

#[derive(Deserialize, Debug)]
struct RawGroup {
    title: String,
    #[serde(default)]
    section: Vec<RawSection>,
}

#[derive(Deserialize, Debug)]
struct RawSection {
    title: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    command: Vec<RawCommand>,
}

#[derive(Deserialize, Debug, Clone)]
struct RawCommand {
    note: String,
    cmd: Option<String>,
    unix: Option<String>,
    windows: Option<String>,
    powershell: Option<String>,
    linux: Option<String>,
    macos: Option<String>,
}

#[derive(Deserialize, Debug)]
struct RawRecipesFile {
    #[serde(default)]
    recipe: Vec<RawRecipe>,
}

#[derive(Deserialize, Debug)]
struct RawSafetyFile {
    #[serde(default)]
    dangerous_patterns: Vec<String>,
    #[serde(default)]
    path_denylist: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct RawContextFile {
    default_package_manager: String,
    #[serde(default)]
    package_manager: Vec<RawPackageManager>,
    #[serde(default)]
    skip_rule: Vec<RawContextSkipRule>,
}

#[derive(Deserialize, Debug)]
struct RawPackageManager {
    id: String,
    #[serde(default)]
    markers: Vec<String>,
    exec: String,
    run: String,
    install: String,
}

#[derive(Deserialize, Debug)]
struct RawContextSkipRule {
    command: String,
    signal: String,
    note: String,
}

#[derive(Deserialize, Debug)]
struct RawRecipe {
    id: String,
    tool: String,
    kind: String,
    title: String,
    action: Option<String>,
    object: Option<String>,
    #[serde(default)]
    triggers: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    checked: Option<String>,
    base: Option<String>,
    note: Option<String>,
    #[serde(default)]
    option: Vec<RawRecipeOption>,
    primary: Option<RawRecipeCommand>,
    #[serde(default)]
    variant: Vec<RawRecipeCommand>,
    #[serde(default, rename = "next")]
    next_commands: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct RawRecipeOption {
    id: String,
    #[serde(rename = "match", default)]
    matches: Vec<String>,
    add: String,
    group: Option<String>,
    #[serde(default)]
    required_default: bool,
    #[serde(default)]
    suggest: bool,
}

#[derive(Deserialize, Debug)]
struct RawRecipeCommand {
    cmd: Option<String>,
    unix: Option<String>,
    windows: Option<String>,
    powershell: Option<String>,
    linux: Option<String>,
    macos: Option<String>,
    note: Option<String>,
    when: Option<String>,
    #[serde(rename = "match", default)]
    matches: Vec<String>,
    risk: Option<String>,
    warn: Option<String>,
    pre: Option<String>,
}

#[derive(Debug)]
struct ParsedTool {
    raw: RawToolFile,
    spec: ToolSpec,
}

fn section_spec(section: &RawSection) -> SectionSpec {
    SectionSpec {
        title: section.title.clone(),
        aliases: section.aliases.clone(),
        commands: section
            .command
            .iter()
            .map(|command| CommandSpec {
                note: command.note.clone(),
                cmd: command.cmd.clone(),
                unix: command.unix.clone(),
                windows: command.windows.clone(),
                powershell: command.powershell.clone(),
                linux: command.linux.clone(),
                macos: command.macos.clone(),
            })
            .collect(),
    }
}

fn to_spec(source: String, raw: &RawToolFile) -> Result<ToolSpec, String> {
    let name = raw
        .name
        .clone()
        .ok_or_else(|| format!("{source}: non-draft tool is missing name"))?;
    let category = raw
        .category
        .clone()
        .ok_or_else(|| format!("{source}: non-draft tool is missing category"))?;
    let mut groups = Vec::new();
    if !raw.section.is_empty() {
        groups.push(GroupSpec {
            title: name.clone(),
            sections: raw.section.iter().map(section_spec).collect(),
        });
    }
    groups.extend(raw.group.iter().map(|group| GroupSpec {
        title: group.title.clone(),
        sections: group.section.iter().map(section_spec).collect(),
    }));
    Ok(ToolSpec {
        source,
        slug: raw.tool.clone(),
        name,
        category,
        aliases: raw.aliases.clone(),
        platforms: raw.platforms.clone(),
        notes: raw.notes.clone(),
        groups,
    })
}

fn emit_option_str(option: &Option<String>) -> String {
    option
        .as_ref()
        .map(|value| format!("Some({value:?})"))
        .unwrap_or_else(|| "None".to_string())
}

fn emit_os_list(platforms: &[String]) -> String {
    let variants = platforms.iter().map(|platform| match platform.as_str() {
        "windows" => "crate::os::Os::Windows",
        "linux" => "crate::os::Os::Linux",
        "macos" => "crate::os::Os::Macos",
        _ => unreachable!("platforms are validated before generation"),
    });
    format!("&[{}]", variants.collect::<Vec<_>>().join(", "))
}

fn emit_strings(values: &[String], indent: &str, out: &mut String) {
    if values.is_empty() {
        out.push_str("&[]");
        return;
    }
    out.push_str("&[\n");
    for value in values {
        out.push_str(&format!("{indent}    {value:?},\n"));
    }
    out.push_str(indent);
    out.push(']');
}

fn emit_section(section: &RawSection, indent: &str, out: &mut String) {
    out.push_str(&format!("{indent}Section {{\n"));
    out.push_str(&format!("{indent}    title: {:?},\n", section.title));
    out.push_str(&format!("{indent}    aliases: "));
    emit_strings(&section.aliases, &format!("{indent}    "), out);
    out.push_str(",\n");
    out.push_str(&format!("{indent}    commands: &[\n"));
    for command in &section.command {
        out.push_str(&format!("{indent}        Command {{\n"));
        out.push_str(&format!("{indent}            note: {:?},\n", command.note));
        for (field, value) in [
            ("cmd", &command.cmd),
            ("unix", &command.unix),
            ("windows", &command.windows),
            ("powershell", &command.powershell),
            ("linux", &command.linux),
            ("macos", &command.macos),
        ] {
            out.push_str(&format!(
                "{indent}            {field}: {},\n",
                emit_option_str(value)
            ));
        }
        out.push_str(&format!("{indent}        }},\n"));
    }
    out.push_str(&format!("{indent}    ],\n{indent}}}"));
}

fn emit_recipe_command(
    command: &RawRecipeCommand,
    fallback_note: &str,
    indent: &str,
    out: &mut String,
) {
    let risk = match command.risk.as_deref().unwrap_or("safe") {
        "safe" => "crate::model::Risk::Safe",
        "caution" => "crate::model::Risk::Caution",
        "danger" => "crate::model::Risk::Danger",
        _ => unreachable!("validated before generation"),
    };
    out.push_str(&format!("{indent}crate::model::RecipeCommand {{\n"));
    out.push_str(&format!(
        "{indent}    note: {:?},\n",
        command.note.as_deref().unwrap_or(fallback_note)
    ));
    for (field, value) in [
        ("when", &command.when),
        ("warn", &command.warn),
        ("pre", &command.pre),
        ("cmd", &command.cmd),
        ("unix", &command.unix),
        ("windows", &command.windows),
        ("powershell", &command.powershell),
        ("linux", &command.linux),
        ("macos", &command.macos),
    ] {
        out.push_str(&format!(
            "{indent}    {field}: {},\n",
            emit_option_str(value)
        ));
    }
    out.push_str(&format!("{indent}    matches: "));
    emit_strings(&command.matches, &format!("{indent}    "), out);
    out.push_str(&format!(",\n{indent}    risk: {risk},\n{indent}}}"));
}

fn validate_recipes(
    recipes: &[RawRecipe],
    tools: &[ToolSpec],
    safety: &RawSafetyFile,
) -> Result<(), String> {
    let tool_slugs: HashSet<&str> = tools.iter().map(|tool| tool.slug.as_str()).collect();
    let mut ids = HashSet::new();
    for recipe in recipes {
        if !ids.insert(recipe.id.as_str()) || recipe.id.trim().is_empty() {
            return Err(format!(
                "data/recipes.toml: duplicate or empty recipe id {:?}",
                recipe.id
            ));
        }
        if !tool_slugs.contains(recipe.tool.as_str()) {
            return Err(format!(
                "data/recipes.toml: recipe {:?} references unknown tool {:?}",
                recipe.id, recipe.tool
            ));
        }
        if recipe.triggers.is_empty() {
            return Err(format!(
                "data/recipes.toml: recipe {:?} needs a trigger",
                recipe.id
            ));
        }
        if !matches!(recipe.kind.as_str(), "single" | "compose") {
            return Err(format!(
                "data/recipes.toml: recipe {:?} has invalid kind",
                recipe.id
            ));
        }
        if recipe.kind == "compose" && recipe.base.is_none() {
            return Err(format!(
                "data/recipes.toml: compose recipe {:?} needs base",
                recipe.id
            ));
        }
        if recipe.kind == "single" && recipe.primary.is_none() {
            return Err(format!(
                "data/recipes.toml: single recipe {:?} needs primary",
                recipe.id
            ));
        }
        if recipe.kind == "compose" && recipe.note.as_deref().is_none_or(str::is_empty) {
            return Err(format!(
                "data/recipes.toml: compose recipe {:?} needs a note",
                recipe.id
            ));
        }
        if let Some(base) = &recipe.base {
            validate_recipe_template(base, &recipe.id)?;
            if is_dangerous(base, safety) {
                return Err(format!(
                    "data/recipes.toml: compose base in {:?} must not be destructive",
                    recipe.id
                ));
            }
        }
        for next in &recipe.next_commands {
            validate_recipe_template(next, &recipe.id)?;
        }
        let commands = recipe.primary.iter().chain(recipe.variant.iter());
        for command in commands {
            if [
                command.cmd.as_ref(),
                command.unix.as_ref(),
                command.windows.as_ref(),
                command.powershell.as_ref(),
                command.linux.as_ref(),
                command.macos.as_ref(),
            ]
            .iter()
            .all(Option::is_none)
            {
                return Err(format!(
                    "data/recipes.toml: recipe {:?} has command without OS value",
                    recipe.id
                ));
            }
            match command.risk.as_deref().unwrap_or("safe") {
                "safe" | "caution" | "danger" => {}
                other => {
                    return Err(format!(
                        "data/recipes.toml: recipe {:?} has invalid risk {:?}",
                        recipe.id, other
                    ))
                }
            }
            if command.risk.as_deref() == Some("danger")
                && command.warn.as_deref().is_none_or(str::is_empty)
            {
                return Err(format!(
                    "data/recipes.toml: danger command in {:?} needs warn",
                    recipe.id
                ));
            }
            if command.note.as_deref().is_none_or(str::is_empty)
                && command.when.as_deref().is_none_or(str::is_empty)
            {
                return Err(format!(
                    "data/recipes.toml: command in {:?} needs a note or when label",
                    recipe.id
                ));
            }
            let values = [
                &command.cmd,
                &command.unix,
                &command.windows,
                &command.powershell,
                &command.linux,
                &command.macos,
            ];
            if values
                .iter()
                .filter_map(|value| value.as_deref())
                .any(|value| {
                    let value = value.to_ascii_lowercase();
                    value.starts_with("dd ")
                        || safety
                            .dangerous_patterns
                            .iter()
                            // `dd ` is an executable, not a general text
                            // fragment: matching it inside `add <pkg>` would
                            // incorrectly reject harmless package commands.
                            .any(|pattern| pattern != "dd " && value.contains(pattern))
                })
                && command.risk.as_deref().unwrap_or("safe") == "safe"
            {
                return Err(format!(
                    "data/recipes.toml: unsafe command in {:?} must declare risk",
                    recipe.id
                ));
            }
            if values
                .iter()
                .filter_map(|value| value.as_deref())
                .any(|value| value.contains("--no-preserve-root"))
            {
                return Err(format!(
                    "data/recipes.toml: recipe {:?} must never use --no-preserve-root",
                    recipe.id
                ));
            }
            for value in values.iter().filter_map(|value| value.as_deref()) {
                validate_recipe_template(value, &recipe.id)?;
            }
        }
        let mut option_ids = HashSet::new();
        let mut required_defaults = HashSet::new();
        for option in &recipe.option {
            if !option_ids.insert(option.id.as_str()) || option.id.is_empty() {
                return Err(format!(
                    "data/recipes.toml: duplicate or empty option id {:?} in {:?}",
                    option.id, recipe.id
                ));
            }
            if option.matches.is_empty() {
                return Err(format!(
                    "data/recipes.toml: option {:?} in {:?} needs match",
                    option.id, recipe.id
                ));
            }
            if option.add.is_empty() {
                return Err(format!(
                    "data/recipes.toml: option {:?} in {:?} has an empty add fragment",
                    option.id, recipe.id
                ));
            }
            if option.required_default
                && option
                    .group
                    .as_deref()
                    .is_some_and(|group| !required_defaults.insert(group))
            {
                return Err(format!(
                    "data/recipes.toml: option group in {:?} has multiple required defaults",
                    recipe.id
                ));
            }
        }
    }
    Ok(())
}

fn is_dangerous(value: &str, safety: &RawSafetyFile) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("dd ")
        || safety
            .dangerous_patterns
            .iter()
            .any(|pattern| pattern != "dd " && lower.contains(pattern))
}

fn validate_recipe_template(value: &str, recipe_id: &str) -> Result<(), String> {
    const KNOWN_VARIABLES: &[&str] = &["pkg.exec", "pkg.run", "pkg.install"];
    let mut remaining = value;
    while let Some(start) = remaining.find('{') {
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find('}') else {
            return Err(format!(
                "data/recipes.toml: recipe {:?} has an unclosed template variable in {:?}",
                recipe_id, value
            ));
        };
        let variable = &after_start[..end];
        if !KNOWN_VARIABLES.contains(&variable) {
            return Err(format!(
                "data/recipes.toml: recipe {:?} has unknown template variable {{{}}}",
                recipe_id, variable
            ));
        }
        remaining = &after_start[end + 1..];
    }
    if remaining.contains('}') {
        return Err(format!(
            "data/recipes.toml: recipe {:?} has an unmatched }} in {:?}",
            recipe_id, value
        ));
    }
    Ok(())
}

fn generate_safety_rs(safety: &RawSafetyFile) -> String {
    let mut out = String::from("// Generated by build.rs - DO NOT EDIT\n\npub static SAFETY_POLICY: SafetyPolicy = SafetyPolicy {\n    dangerous_patterns: ");
    emit_strings(&safety.dangerous_patterns, "    ", &mut out);
    out.push_str(",\n    path_denylist: ");
    emit_strings(&safety.path_denylist, "    ", &mut out);
    out.push_str(",\n};\n");
    out
}

fn validate_context(context: &RawContextFile) -> Result<(), String> {
    if context.package_manager.is_empty() {
        return Err("data/context.toml: needs at least one package_manager".into());
    }
    let mut ids = HashSet::new();
    for manager in &context.package_manager {
        if manager.id.is_empty() || !ids.insert(manager.id.as_str()) || manager.markers.is_empty() {
            return Err(format!(
                "data/context.toml: invalid package manager {:?}",
                manager.id
            ));
        }
        if manager.exec.is_empty() || manager.run.is_empty() || manager.install.is_empty() {
            return Err(format!(
                "data/context.toml: package manager {:?} has an empty command",
                manager.id
            ));
        }
    }
    if !ids.contains(context.default_package_manager.as_str()) {
        return Err(format!(
            "data/context.toml: unknown default package manager {:?}",
            context.default_package_manager
        ));
    }
    for rule in &context.skip_rule {
        if rule.command.is_empty()
            || !matches!(rule.signal.as_str(), "git_repo" | "has_remote_origin")
            || rule.note.is_empty()
        {
            return Err("data/context.toml: invalid skip_rule".into());
        }
    }
    Ok(())
}

fn generate_context_rs(context: &RawContextFile) -> String {
    let mut out = String::from("// Generated by build.rs - DO NOT EDIT\n\npub static CONTEXT_POLICY: ContextPolicy = ContextPolicy {\n");
    out.push_str(&format!(
        "    default_package_manager: {:?},\n    package_managers: &[\n",
        context.default_package_manager
    ));
    for manager in &context.package_manager {
        out.push_str(&format!(
            "        PackageManager {{ id: {:?}, markers: ",
            manager.id
        ));
        emit_strings(&manager.markers, "        ", &mut out);
        out.push_str(&format!(
            ", exec: {:?}, run: {:?}, install: {:?} }},\n",
            manager.exec, manager.run, manager.install
        ));
    }
    out.push_str("    ],\n    skip_rules: &[\n");
    for rule in &context.skip_rule {
        out.push_str(&format!(
            "        ContextSkipRule {{ command: {:?}, signal: {:?}, note: {:?} }},\n",
            rule.command, rule.signal, rule.note
        ));
    }
    out.push_str("    ],\n};\n");
    out
}

fn generate_recipes_rs(recipes: &[RawRecipe]) -> String {
    let mut out = String::from(
        "// Generated by build.rs - DO NOT EDIT\n\npub static RECIPES: &[Recipe] = &[\n",
    );
    for recipe in recipes {
        let kind = if recipe.kind == "single" {
            "crate::model::RecipeKind::Single"
        } else {
            "crate::model::RecipeKind::Compose"
        };
        out.push_str("    Recipe {\n");
        out.push_str(&format!(
            "        id: {:?}, tool: {:?}, kind: {kind}, title: {:?},\n",
            recipe.id, recipe.tool, recipe.title
        ));
        out.push_str(&format!(
            "        action: {}, object: {},\n",
            emit_option_str(&recipe.action),
            emit_option_str(&recipe.object)
        ));
        for (field, values) in [
            ("triggers", &recipe.triggers),
            ("tags", &recipe.tags),
            ("next", &recipe.next_commands),
        ] {
            out.push_str(&format!("        {field}: "));
            emit_strings(values, "        ", &mut out);
            out.push_str(",\n");
        }
        out.push_str(&format!(
            "        checked: {},\n",
            emit_option_str(&recipe.checked)
        ));
        if let Some(base) = &recipe.base {
            let base_command = RawRecipeCommand {
                cmd: Some(base.clone()),
                unix: None,
                windows: None,
                powershell: None,
                linux: None,
                macos: None,
                note: recipe.note.clone(),
                when: None,
                matches: Vec::new(),
                risk: Some("safe".into()),
                warn: None,
                pre: None,
            };
            out.push_str("        base: Some(");
            emit_recipe_command(&base_command, "", "", &mut out);
            out.push_str("),\n");
        } else {
            out.push_str("        base: None,\n");
        }
        out.push_str("        options: &[\n");
        for option in &recipe.option {
            out.push_str(&format!(
                "            RecipeOption {{ id: {:?}, matches: ",
                option.id
            ));
            emit_strings(&option.matches, "            ", &mut out);
            out.push_str(&format!(
                ", add: {:?}, group: {}, required_default: {}, suggest: {} }},\n",
                option.add,
                emit_option_str(&option.group),
                option.required_default,
                option.suggest
            ));
        }
        out.push_str("        ],\n");
        if let Some(primary) = &recipe.primary {
            out.push_str("        primary: Some(");
            emit_recipe_command(primary, "", "", &mut out);
            out.push_str("),\n");
        } else {
            out.push_str("        primary: None,\n");
        }
        out.push_str("        variants: &[\n");
        for variant in &recipe.variant {
            emit_recipe_command(variant, "", "            ", &mut out);
            out.push_str(",\n");
        }
        out.push_str("        ],\n    },\n");
    }
    out.push_str("];\n");
    out
}

fn generate_content_rs(
    categories: &[(String, Vec<ParsedTool>)],
    shadowed: &[(String, String)],
) -> String {
    let shadowed: HashSet<(&str, &str)> = shadowed
        .iter()
        .map(|(slug, alias)| (slug.as_str(), alias.as_str()))
        .collect();
    let mut out = String::from("// Generated by build.rs - DO NOT EDIT\n\n");
    out.push_str("pub static CATEGORIES: &[Category] = &[\n");
    for (category_name, tools) in categories {
        out.push_str("    Category {\n");
        out.push_str(&format!(
            "        name: {category_name:?},\n        tools: &[\n"
        ));
        for parsed in tools {
            let raw = &parsed.raw;
            out.push_str("            Tool {\n");
            out.push_str(&format!("                slug: {:?},\n", raw.tool));
            out.push_str(&format!("                name: {:?},\n", parsed.spec.name));
            out.push_str(&format!(
                "                category: {:?},\n",
                parsed.spec.category
            ));
            out.push_str("                aliases: ");
            emit_strings(&raw.aliases, "                ", &mut out);
            out.push_str(",\n                flag_aliases: ");
            let flag_aliases: Vec<String> = raw
                .aliases
                .iter()
                .filter(|alias| !shadowed.contains(&(raw.tool.as_str(), alias.as_str())))
                .cloned()
                .collect();
            emit_strings(&flag_aliases, "                ", &mut out);
            out.push_str(&format!(
                ",\n                platforms: {},\n",
                emit_os_list(&raw.platforms)
            ));
            out.push_str("                notes: ");
            emit_strings(&raw.notes, "                ", &mut out);
            out.push_str(",\n                groups: &[\n");
            if !raw.section.is_empty() {
                out.push_str("                    Group {\n");
                out.push_str(&format!(
                    "                        title: {:?},\n",
                    parsed.spec.name
                ));
                out.push_str("                        sections: &[\n");
                for section in &raw.section {
                    emit_section(section, "                            ", &mut out);
                    out.push_str(",\n");
                }
                out.push_str("                        ],\n                    },\n");
            }
            for group in &raw.group {
                out.push_str("                    Group {\n");
                out.push_str(&format!(
                    "                        title: {:?},\n",
                    group.title
                ));
                out.push_str("                        sections: &[\n");
                for section in &group.section {
                    emit_section(section, "                            ", &mut out);
                    out.push_str(",\n");
                }
                out.push_str("                        ],\n                    },\n");
            }
            out.push_str("                ],\n            },\n");
        }
        out.push_str("        ],\n    },\n");
    }
    out.push_str("];\n");
    out
}

fn category_rank(order: &[String], category: &str) -> (usize, String) {
    match order.iter().position(|entry| entry == category) {
        Some(index) => (index, String::new()),
        None => (usize::MAX, category.to_string()),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=data");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/content_validation.rs");

    let data_dir = Path::new("data");
    let categories_path = data_dir.join("categories.toml");
    let categories: RawCategories = toml::from_str(
        &fs::read_to_string(&categories_path).expect("failed to read data/categories.toml"),
    )
    .expect("data/categories.toml: failed to parse TOML");

    let mut files: Vec<PathBuf> = fs::read_dir(data_dir)
        .expect("failed to read data directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .filter(|path| {
            path.file_name().is_some_and(|name| {
                name != "categories.toml"
                    && name != "recipes.toml"
                    && name != "safety.toml"
                    && name != "context.toml"
            })
        })
        .collect();
    files.sort();

    let mut parsed = Vec::new();
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let source = path.display().to_string();
        let raw: RawToolFile = toml::from_str(
            &fs::read_to_string(&path).unwrap_or_else(|error| panic!("{source}: {error}")),
        )
        .unwrap_or_else(|error| panic!("{source}: failed to parse TOML: {error}"));
        if raw.draft {
            println!("cargo:warning={source}: draft tool {:?} skipped", raw.tool);
            continue;
        }
        let spec = to_spec(source.clone(), &raw).unwrap_or_else(|error| panic!("{error}"));
        content_validation::validate_tool(&spec).unwrap_or_else(|error| panic!("{error}"));
        for alias in content_validation::duplicate_section_aliases(&spec) {
            println!(
                "cargo:warning={source}: duplicate section alias {:?}",
                alias
            );
        }
        parsed.push(ParsedTool { raw, spec });
    }

    let specs: Vec<ToolSpec> = parsed.iter().map(|tool| tool.spec.clone()).collect();
    content_validation::validate_namespace(&specs).unwrap_or_else(|error| panic!("{error}"));
    let shadowed = content_validation::shadowed_aliases(&specs);
    if !shadowed.is_empty() {
        println!("cargo:warning=shadowed tool aliases: {:?}", shadowed);
    }

    let known_categories: HashSet<&str> = categories.order.iter().map(String::as_str).collect();
    for tool in &parsed {
        if !known_categories.contains(tool.spec.category.as_str()) {
            println!(
                "cargo:warning=unlisted category {:?} sorts after categories.toml",
                tool.spec.category
            );
        }
    }
    parsed.sort_by(|left, right| {
        category_rank(&categories.order, &left.spec.category)
            .cmp(&category_rank(&categories.order, &right.spec.category))
            .then_with(|| left.spec.category.cmp(&right.spec.category))
            .then_with(|| {
                left.raw
                    .order
                    .unwrap_or(u32::MAX)
                    .cmp(&right.raw.order.unwrap_or(u32::MAX))
            })
            .then_with(|| left.spec.name.cmp(&right.spec.name))
    });
    let mut grouped: BTreeMap<(usize, String, String), Vec<ParsedTool>> = BTreeMap::new();
    for tool in parsed {
        let (rank, fallback) = category_rank(&categories.order, &tool.spec.category);
        grouped
            .entry((rank, fallback, tool.spec.category.clone()))
            .or_default()
            .push(tool);
    }
    let ordered_categories: Vec<(String, Vec<ParsedTool>)> = grouped
        .into_values()
        .map(|tools| {
            let name = tools[0].spec.category.clone();
            (name, tools)
        })
        .collect();
    let generated = generate_content_rs(&ordered_categories, &shadowed);
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    fs::write(Path::new(&out_dir).join("content.rs"), generated)
        .expect("failed to write generated content.rs");
    let recipes_path = data_dir.join("recipes.toml");
    println!("cargo:rerun-if-changed={}", recipes_path.display());
    let raw_recipes: RawRecipesFile = toml::from_str(
        &fs::read_to_string(&recipes_path).expect("failed to read data/recipes.toml"),
    )
    .expect("data/recipes.toml: failed to parse TOML");
    let safety_path = data_dir.join("safety.toml");
    println!("cargo:rerun-if-changed={}", safety_path.display());
    let safety: RawSafetyFile =
        toml::from_str(&fs::read_to_string(&safety_path).expect("failed to read data/safety.toml"))
            .expect("data/safety.toml: failed to parse TOML");
    if safety.dangerous_patterns.is_empty() || safety.path_denylist.is_empty() {
        panic!("data/safety.toml: dangerous_patterns and path_denylist cannot be empty");
    }
    validate_recipes(&raw_recipes.recipe, &specs, &safety)
        .unwrap_or_else(|error| panic!("{error}"));
    fs::write(
        Path::new(&out_dir).join("recipes.rs"),
        generate_recipes_rs(&raw_recipes.recipe),
    )
    .expect("failed to write generated recipes.rs");
    fs::write(
        Path::new(&out_dir).join("safety.rs"),
        generate_safety_rs(&safety),
    )
    .expect("failed to write generated safety.rs");
    let context_path = data_dir.join("context.toml");
    println!("cargo:rerun-if-changed={}", context_path.display());
    let context: RawContextFile = toml::from_str(
        &fs::read_to_string(&context_path).expect("failed to read data/context.toml"),
    )
    .expect("data/context.toml: failed to parse TOML");
    validate_context(&context).unwrap_or_else(|error| panic!("{error}"));
    fs::write(
        Path::new(&out_dir).join("context.rs"),
        generate_context_rs(&context),
    )
    .expect("failed to write generated context.rs");
}
