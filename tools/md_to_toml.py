#!/usr/bin/env python3
"""Import the v2 Markdown command library into deterministic TOML files.

This is a development-only, standard-library tool. Run it from any directory:

    python3 tools/md_to_toml.py          # regenerate and verify
    python3 tools/md_to_toml.py --check  # verify existing generated files
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "VERSION2_COMMANDS.md"
DATA_DIR = ROOT / "data"
GIT_ADDITIONS = ROOT / "tools" / "generated" / "git_additions.toml"

SUMMARY_TOOLS = 62
SUMMARY_SECTIONS = 286
SUMMARY_COMMAND_LINES = 1651
SUMMARY_ENTRIES = 1639
SUMMARY_MERGED_PAIRS = 12

PLATFORM_MAP = {
    "Windows, Linux, macOS": ("windows", "linux", "macos"),
    "Linux, macOS": ("linux", "macos"),
    "Linux": ("linux",),
    "macOS": ("macos",),
    "Windows": ("windows",),
}
TAG_TO_KEY = {"win": "windows", "linux": "linux", "mac": "macos", "unix": "unix"}
KEY_TO_TAG = {value: key for key, value in TAG_TO_KEY.items()}
ALL_PLATFORMS = ("windows", "linux", "macos")
FLAG_RE = re.compile(
    r"^\*\*Flag:\*\* `-(?P<slug>[^`]+)`  \|  \*\*Also matches:\*\* "
    r"(?P<aliases>.*?)  \|  \*\*Platforms:\*\* (?P<platforms>.+)$"
)
SECTION_RE = re.compile(r"^# ---(?P<title>.+)---$")
TAG_RE = re.compile(r"^\[(?P<tag>win|linux|mac|unix)\]\s+(?P<command>.+)$")
SEPARATOR_RE = re.compile(r"\s{2,}---\s*")


class ParseError(RuntimeError):
    pass


@dataclass
class Command:
    note: str
    values: dict[str, str] = field(default_factory=dict)
    source_lines: list[int] = field(default_factory=list)


@dataclass
class Section:
    title: str
    commands: list[Command] = field(default_factory=list)


@dataclass
class Group:
    title: str
    sections: list[Section] = field(default_factory=list)


@dataclass
class Tool:
    name: str
    slug: str
    aliases: list[str]
    platforms: tuple[str, ...]
    category: str
    order: int
    notes: list[str] = field(default_factory=list)
    groups: list[Group] = field(default_factory=list)


def error(line_number: int, message: str) -> ParseError:
    return ParseError(f"{SOURCE}:{line_number}: {message}")


def library_lines() -> list[tuple[int, str]]:
    lines = SOURCE.read_text(encoding="utf-8").splitlines()
    start = next(
        (index for index, line in enumerate(lines) if line == "## 4. Command library"),
        None,
    )
    end = next(
        (index for index, line in enumerate(lines) if line.startswith("## 5. Open items")),
        None,
    )
    if start is None or end is None or start >= end:
        raise ParseError(f"{SOURCE}: cannot find command-library boundaries")
    selected = [(index + 1, line) for index, line in enumerate(lines[start + 1 : end], start + 1)]
    for line_number, line in selected:
        if not line.isascii():
            raise error(line_number, "non-ASCII text in command library")
    return selected


def parse_library() -> list[Tool]:
    tools: list[Tool] = []
    category: str | None = None
    tool: Tool | None = None
    group: Group | None = None
    section: Section | None = None
    in_fence = False
    last_tagged: Command | None = None
    last_command_line: int | None = None

    for line_number, line in library_lines():
        if line.startswith("```"):
            in_fence = not in_fence
            if not in_fence:
                section = None
                last_tagged = None
                last_command_line = None
            continue

        if in_fence:
            if not line:
                last_tagged = None
                last_command_line = None
                continue
            section_match = SECTION_RE.match(line)
            if section_match:
                if group is None:
                    raise error(line_number, "section appears before a group")
                section = Section(section_match.group("title"))
                group.sections.append(section)
                last_tagged = None
                last_command_line = None
                continue
            if section is None:
                raise error(line_number, "command appears before a section header")
            separator = SEPARATOR_RE.search(line)
            if separator is None:
                raise error(
                    line_number,
                    "command line needs a --- separator preceded by at least two spaces",
                )
            command_text = line[: separator.start()].rstrip()
            note = line[separator.end() :].strip()
            if not command_text or not note:
                raise error(line_number, "command and note must both be non-empty")
            tagged = TAG_RE.match(command_text)
            if tagged:
                key = TAG_TO_KEY[tagged.group("tag")]
                command_text = tagged.group("command")
                if (
                    last_tagged is not None
                    and last_command_line == line_number - 1
                    and last_tagged.note == note
                    and key not in last_tagged.values
                ):
                    last_tagged.values[key] = command_text
                    last_tagged.source_lines.append(line_number)
                else:
                    command = Command(note, {key: command_text}, [line_number])
                    section.commands.append(command)
                    last_tagged = command
                last_command_line = line_number
            else:
                command = Command(note, {"cmd": command_text}, [line_number])
                section.commands.append(command)
                last_tagged = None
                last_command_line = line_number
            continue

        if line.startswith("## "):
            category = line[3:].strip()
            tool = None
            group = None
            continue
        if line.startswith("### "):
            if category is None:
                raise error(line_number, "tool appears before a category")
            tool = Tool(
                name=line[4:].strip(),
                slug="",
                aliases=[],
                platforms=(),
                category=category,
                order=1 + sum(existing.category == category for existing in tools),
            )
            tools.append(tool)
            group = None
            continue
        if line.startswith("**Flag:"):
            if tool is None:
                raise error(line_number, "flag metadata appears before a tool")
            match = FLAG_RE.match(line)
            if match is None:
                raise error(line_number, "unrecognized tool flag metadata")
            platform_text = match.group("platforms")
            if platform_text not in PLATFORM_MAP:
                raise error(line_number, f"unrecognized platform list {platform_text!r}")
            tool.slug = match.group("slug")
            tool.aliases = [alias.strip() for alias in match.group("aliases").split(",")]
            tool.platforms = PLATFORM_MAP[platform_text]
            continue
        if line.startswith("> **Note:** "):
            if tool is None:
                raise error(line_number, "tool note appears before a tool")
            tool.notes.append(line.removeprefix("> **Note:** "))
            continue
        if line.startswith("#### "):
            if tool is None:
                raise error(line_number, "group appears before a tool")
            group = Group(line[5:].strip())
            tool.groups.append(group)

    if in_fence:
        raise ParseError(f"{SOURCE}: unterminated fenced block")
    for parsed_tool in tools:
        if not parsed_tool.slug:
            raise ParseError(f"{SOURCE}: tool {parsed_tool.name!r} has no flag metadata")
        if not parsed_tool.groups:
            raise ParseError(f"{SOURCE}: tool {parsed_tool.name!r} has no groups")
        for parsed_group in parsed_tool.groups:
            if not parsed_group.sections:
                raise ParseError(
                    f"{SOURCE}: group {parsed_group.title!r} in {parsed_tool.name!r} is empty"
                )
            for parsed_section in parsed_group.sections:
                if not parsed_section.commands:
                    raise ParseError(
                        f"{SOURCE}: section {parsed_section.title!r} in {parsed_tool.name!r} is empty"
                    )
    return tools


def toml_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=True)


def toml_list(values: list[str] | tuple[str, ...]) -> str:
    return "[" + ", ".join(toml_string(value) for value in values) + "]"


def emit_groups(groups: list[Group]) -> list[str]:
    lines: list[str] = []
    for group in groups:
        lines.extend(("[[group]]", f"title = {toml_string(group.title)}", ""))
        for section in group.sections:
            lines.extend(
                (
                    "[[group.section]]",
                    f"title = {toml_string(section.title)}",
                    "aliases = []",
                    "",
                )
            )
            for command in section.commands:
                lines.append("[[group.section.command]]")
                lines.append(f"note = {toml_string(command.note)}")
                for key, value in command.values.items():
                    lines.append(f"{key} = {toml_string(value)}")
                lines.append("")
    return lines


def emit_tool(tool: Tool) -> str:
    lines = [
        f"tool = {toml_string(tool.slug)}",
        f"name = {toml_string(tool.name)}",
        f"category = {toml_string(tool.category)}",
        f"order = {tool.order}",
        f"aliases = {toml_list(tool.aliases)}",
    ]
    if tool.platforms != ALL_PLATFORMS:
        lines.append(f"platforms = {toml_list(tool.platforms)}")
    if tool.notes:
        lines.append(f"notes = {toml_list(tool.notes)}")
    lines.extend(("draft = true", ""))
    lines.extend(emit_groups(tool.groups))
    return "\n".join(lines).rstrip() + "\n"


def emit_git_additions(tool: Tool) -> str:
    return "\n".join(emit_groups(tool.groups)).rstrip() + "\n"


def generated_paths(tools: list[Tool]) -> dict[str, Path]:
    paths = {}
    for tool in tools:
        paths[tool.slug] = GIT_ADDITIONS if tool.slug == "git" else DATA_DIR / f"{tool.slug}.toml"
    return paths


def write_generated(tools: list[Tool]) -> None:
    GIT_ADDITIONS.parent.mkdir(parents=True, exist_ok=True)
    for tool in tools:
        path = generated_paths(tools)[tool.slug]
        text = emit_git_additions(tool) if tool.slug == "git" else emit_tool(tool)
        path.write_text(text, encoding="ascii", newline="\n")


def flattened_source(tools: list[Tool]) -> list[tuple[str, str, str, str, str, str]]:
    lines = []
    for tool in tools:
        for group in tool.groups:
            for section in group.sections:
                for command in section.commands:
                    for key, value in command.values.items():
                        lines.append(
                            (tool.slug, group.title, section.title, KEY_TO_TAG.get(key, ""), value, command.note)
                        )
    return lines


def read_generated_tool(tool: Tool, path: Path) -> Tool:
    parsed = tomllib.loads(path.read_text(encoding="ascii"))
    if tool.slug == "git":
        groups = parsed.get("group", [])
        metadata = {
            "tool": tool.slug,
            "name": tool.name,
            "category": tool.category,
            "aliases": tool.aliases,
            "platforms": list(tool.platforms),
            "notes": tool.notes,
        }
    else:
        groups = parsed.get("group", [])
        metadata = parsed
    actual = Tool(
        name=metadata["name"],
        slug=metadata["tool"],
        aliases=list(metadata.get("aliases", [])),
        platforms=tuple(metadata.get("platforms", ALL_PLATFORMS)),
        category=metadata["category"],
        order=tool.order,
        notes=list(metadata.get("notes", [])),
    )
    for raw_group in groups:
        group = Group(raw_group["title"])
        actual.groups.append(group)
        for raw_section in raw_group.get("section", []):
            section = Section(raw_section["title"])
            group.sections.append(section)
            for raw_command in raw_section.get("command", []):
                values = {
                    key: value
                    for key, value in raw_command.items()
                    if key in {"cmd", "windows", "linux", "macos", "unix"}
                }
                section.commands.append(Command(raw_command["note"], values))
    return actual


def verify(tools: list[Tool]) -> None:
    paths = generated_paths(tools)
    actual_tools = []
    for tool in tools:
        path = paths[tool.slug]
        if not path.exists():
            raise ParseError(f"missing generated file: {path}")
        actual_tools.append(read_generated_tool(tool, path))

    expected_shape = [
        (tool.slug, tool.name, tool.category, tool.aliases, tool.platforms, tool.notes,
         [(group.title, [section.title for section in group.sections]) for group in tool.groups])
        for tool in tools
    ]
    actual_shape = [
        (tool.slug, tool.name, tool.category, tool.aliases, tool.platforms, tool.notes,
         [(group.title, [section.title for section in group.sections]) for group in tool.groups])
        for tool in actual_tools
    ]
    if actual_shape != expected_shape:
        raise ParseError("generated TOML does not round-trip tool, group, or section metadata")
    expected_lines = flattened_source(tools)
    actual_lines = flattened_source(actual_tools)
    if actual_lines != expected_lines:
        for index, (expected, actual) in enumerate(zip(expected_lines, actual_lines), start=1):
            if expected != actual:
                raise ParseError(
                    f"round-trip mismatch at command {index}: expected {expected!r}, got {actual!r}"
                )
        raise ParseError("round-trip command count differs")

    sections = sum(len(group.sections) for tool in tools for group in tool.groups)
    command_lines = len(expected_lines)
    entries = sum(
        len(section.commands)
        for tool in tools
        for group in tool.groups
        for section in group.sections
    )
    merged_pair_count = command_lines - entries
    print(
        "Generated: "
        f"{len(tools)} tools ({len(tools) - 1} files + Git additions), "
        f"{sections} sections, {command_lines} command lines, "
        f"{entries} command entries ({merged_pair_count} OS pairs merged)"
    )
    print(
        "Library summary: "
        f"{SUMMARY_TOOLS} tools (61 files + Git additions), {SUMMARY_SECTIONS} sections, "
        f"{SUMMARY_COMMAND_LINES} command lines, {SUMMARY_ENTRIES} command entries "
        f"({SUMMARY_MERGED_PAIRS} OS pairs merged)"
    )
    if (len(tools), sections, command_lines, entries, merged_pair_count) != (
        SUMMARY_TOOLS,
        SUMMARY_SECTIONS,
        SUMMARY_COMMAND_LINES,
        SUMMARY_ENTRIES,
        SUMMARY_MERGED_PAIRS,
    ):
        raise ParseError("generated counts do not match the library summary")

    print("Merged OS pairs:")
    for tool in tools:
        for group in tool.groups:
            for section in group.sections:
                for command in section.commands:
                    if len(command.values) < 2:
                        continue
                    variants = "; ".join(
                        f"[{KEY_TO_TAG[key]}] {value}"
                        for key, value in command.values.items()
                    )
                    print(f"  - {tool.slug} / {section.title}: {variants} --- {command.note}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify without rewriting generated files")
    args = parser.parse_args()
    tools = parse_library()
    if not args.check:
        write_generated(tools)
    verify(tools)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ParseError, tomllib.TOMLDecodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
