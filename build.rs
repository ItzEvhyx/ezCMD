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
            path.file_name()
                .is_some_and(|name| name != "categories.toml")
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
}
