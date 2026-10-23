#!/usr/bin/env python3
"""Check local documentation links and Rust's literal documentation inputs.

Project documentation is checked without network access. Original upstream prose
under docs/dependencies/origins/<origin>/ remains reference material; its relative
links can describe files outside the retained product. The origins index itself
and all current guides are checked normally.
"""
from __future__ import annotations

import html
import json
import os
from pathlib import Path
import re
import sys
import tomllib
import unicodedata
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[2]
DOC_SUFFIXES = {'.md', '.markdown', '.rst', '.adoc'}


def blocks(text: str) -> str:
    """Mask comments and fenced code, preserving offsets and line numbers."""
    text = re.sub(r'<!--[\s\S]*?-->', lambda m: re.sub(r'[^\n]', ' ', m[0]), text)
    lines = []
    fence = None
    for line in text.splitlines(keepends=True):
        match = re.match(r' {0,3}(`{3,}|~{3,})', line)
        if fence:
            closing = re.fullmatch(r' {0,3}(`{3,}|~{3,})[ \t]*(?:\r?\n)?', line)
            if closing and closing[1][0] == fence[0] and len(closing[1]) >= len(fence):
                fence = None
            lines.append(re.sub(r'[^\n]', ' ', line))
        elif match and (match[1][0] != '`' or '`' not in line[match.end():]):
            fence = match[1]
            lines.append(re.sub(r'[^\n]', ' ', line))
        else:
            lines.append(line)
    return ''.join(lines)


def prose(text: str) -> str:
    """Mask comments, fenced blocks and inline code, preserving line numbers."""
    return re.sub(r'(`+)([^`]|(?!\1)`)*?\1', lambda m: re.sub(r'[^\n]', ' ', m[0]), blocks(text))


def anchors(text: str) -> set[str]:
    # Keep inline code in headings: its content contributes to GitHub's slug.
    visible = blocks(text)
    lines = re.sub(r'(`+)(.*?)\1', r'\2', visible).splitlines()
    result = set(re.findall(r'<a\s[^>]*(?:id|name)=[\'\"]([^\'\"]+)[\'\"]', prose(visible), re.I))
    counts: dict[str, int] = {}
    headings = set()
    for index, line in enumerate(lines):
        heading = re.match(r' {0,3}#{1,6}\s+(.+?)(?:\s+#+)?\s*$', line)
        if heading:
            title = heading[1]
        elif index + 1 < len(lines) and line.strip() and re.fullmatch(r' {0,3}(?:=+|-+)\s*', lines[index + 1]):
            title = line
        else:
            continue
        title = re.sub(r'!?\[([^\]]+)\]\([^)]*\)', r'\1', title)
        title = html.unescape(re.sub(r'<[^>]*>', '', title)).lower()
        slug = ''.join(c for c in title if c in ' _-' or unicodedata.category(c)[0] in 'LMN')
        slug = slug.replace(' ', '-')
        count = counts.get(slug, 0)
        candidate = f'{slug}-{count}' if count else slug
        while candidate in headings:
            count += 1
            candidate = f'{slug}-{count}'
        counts[slug] = count
        headings.add(candidate)
        result.add(candidate)
    return result


def destination(text: str, start: int) -> str | None:
    """Read an inline destination, allowing balanced parentheses and titles."""
    index = start
    while index < len(text) and text[index].isspace():
        index += 1
    if index == len(text):
        return None
    if text[index] == '<':
        end = text.find('>', index + 1)
        return text[index + 1:end] if end != -1 else None
    begin, depth = index, 0
    while index < len(text):
        char = text[index]
        if char == '\\':
            index += 2
            continue
        if char == '(':
            depth += 1
        elif char == ')':
            if not depth:
                return text[begin:index]
            depth -= 1
        elif char.isspace() and not depth:
            return text[begin:index]
        index += 1
    return None


def links(text: str) -> list[tuple[int, str]]:
    text = prose(text)
    references = {}
    definition = re.compile(r'^ {0,3}\[([^\]]+)\]:\s*(?:<([^>]+)>|(\S+))', re.M)
    for match in definition.finditer(text):
        references[' '.join(match[1].split()).casefold()] = match[2] or match[3]
    found = []
    for match in re.finditer(r'\]\(', text):
        target = destination(text, match.end())
        if target is not None:
            found.append((match.start(), target))
    for match in re.finditer(r'!?\[([^\[\]\n]+)\](?:\[([^\]\n]*)\])?', text):
        if text[match.end():match.end() + 1] in {'(', ':'}:
            continue
        key = ' '.join((match[2] or match[1]).split()).casefold()
        if key in references:
            found.append((match.start(), references[key]))
    for match in re.finditer(r'\b(?:href|src)=[\'\"]([^\'\"]+)[\'\"]', text, re.I):
        found.append((match.start(), match[1]))
    return [(text.count('\n', 0, index) + 1, value) for index, value in found]


def documents(root: Path) -> list[Path]:
    paths = [root / 'README.md', root / 'test/README.md']
    paths.extend((root / 'docs').rglob('*.md'))
    paths.extend((root / 'deploy').rglob('*.md'))
    paths.extend((root / 'src/tools').rglob('*.md'))
    paths.extend(root / name for name in (
        'src/interface/README.md', 'src/interface/dom/README.md', 'src/interface/macros/README.md'))
    origin_root = root / 'docs/dependencies/origins'
    return sorted(path for path in set(paths) if path.is_file() and
                  not (path.is_relative_to(origin_root) and len(path.relative_to(origin_root).parts) > 1))


def check_links(root: Path) -> tuple[list[str], int]:
    errors = []
    count = 0
    anchor_cache: dict[Path, set[str]] = {}
    for path in documents(root):
        for line, raw in links(path.read_text()):
            value = re.sub(r'\\([\\`*{}\[\]()#+.!_> -])', r'\1', html.unescape(raw))
            url = urlsplit(value)
            if url.scheme or url.netloc or not value:
                continue
            count += 1
            target = root / unquote(url.path).lstrip('/') if url.path.startswith('/') else path.parent / unquote(url.path)
            target = target.resolve() if url.path else path.resolve()
            label = f'{path.relative_to(root)}:{line}: {raw}'
            if not target.is_relative_to(root.resolve()):
                errors.append(f'{label}: escapes repository')
            elif not target.exists():
                errors.append(f'{label}: target does not exist')
            elif url.fragment:
                if target.is_dir():
                    target = (target / 'README.md').resolve()
                    if not target.is_relative_to(root.resolve()):
                        errors.append(f'{label}: directory readme escapes repository')
                        continue
                    if not target.is_file():
                        errors.append(f'{label}: directory has no README.md for heading')
                        continue
                if target.suffix.lower() not in {'.md', '.markdown'}:
                    continue
                if target not in anchor_cache:
                    anchor_cache[target] = anchors(target.read_text())
                fragment = unquote(url.fragment).removeprefix('user-content-')
                if fragment not in anchor_cache[target]:
                    errors.append(f'{label}: heading does not exist')
    return errors, count


def rust_string(text: str, start: int) -> tuple[str, int] | None:
    raw = re.match(r'r(#{0,255})"', text[start:])
    if raw:
        begin = start + raw.end()
        end = text.find('"' + raw[1], begin)
        return (text[begin:end], end + 1 + len(raw[1])) if end != -1 else None
    if text[start:start + 1] == '"':
        match = re.match(r'"(?:\\.|[^"\\])*"', text[start:], re.S)
        if match:
            try:
                return json.loads(match[0]), start + match.end()
            except ValueError:
                # Rust also permits escapes such as \u{...}. They are not
                # used in documentation paths, but still delimit a string.
                return match[0][1:-1], start + match.end()
    return None


RUST_TOKEN = re.compile(
    r'//|/\*|(?:br|r)#{0,255}"|"(?:\\.|[^"\\])*"|'
    r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'|"
    r'\binclude_(?:str|bytes)!\s*\(\s*', re.S)
RUST_COMMENT = re.compile(r'/\*|\*/')


def rust_includes(text: str) -> list[tuple[int, str]]:
    """Find literal include paths outside comments and quoted examples."""
    result = []
    index = 0
    while token := RUST_TOKEN.search(text, index):
        value = token[0]
        index = token.end()
        if value == '//':
            end = text.find('\n', index)
            index = end if end != -1 else len(text)
        elif value == '/*':
            depth = 1
            while depth and (comment := RUST_COMMENT.search(text, index)):
                depth += 1 if comment[0] == '/*' else -1
                index = comment.end()
            if depth:
                break
        elif value.startswith('include_'):
            if string := rust_string(text, index):
                result.append((text.count('\n', 0, token.start()) + 1, string[0]))
                index = string[1]
        elif value.startswith(('r', 'br')):
            if string := rust_string(text, token.start() + (1 if value.startswith('br') else 0)):
                index = string[1]
    return result


def source_files(root: Path, suffix: str) -> list[Path]:
    result = []
    for directory, children, files in os.walk(root):
        children[:] = sorted(child for child in children if child not in {'.git', 'target', 'node_modules'})
        result.extend(Path(directory) / name for name in files if name.endswith(suffix))
    return result


def check_inputs(root: Path) -> tuple[list[str], int]:
    errors = []
    count = 0
    for directory in ('src', 'test'):
        for path in source_files(root / directory, '.rs'):
            text = path.read_text()
            if 'include_' not in text:
                continue
            for line, name in rust_includes(text):
                if Path(name).suffix.lower() not in DOC_SUFFIXES:
                    continue
                count += 1
                target = (path.parent / name).resolve()
                if not target.is_relative_to(root.resolve()) or not target.is_file():
                    errors.append(f'{path.relative_to(root)}:{line}: missing local documentation input: {name}')
    for path in source_files(root, 'Cargo.toml'):
        readme = tomllib.loads(path.read_text()).get('package', {}).get('readme')
        if isinstance(readme, str):
            count += 1
            target = (path.parent / readme).resolve()
            if not target.is_relative_to(root.resolve()) or not target.is_file():
                errors.append(f'{path.relative_to(root)}: missing local package readme: {readme}')
    return errors, count


def main() -> int:
    link_errors, link_count = check_links(ROOT)
    input_errors, input_count = check_inputs(ROOT)
    errors = link_errors + input_errors
    for error in errors:
        print(f'check-docs: {error}', file=sys.stderr)
    if errors:
        return 1
    print(f'Documentation: {len(documents(ROOT))} pages, {link_count} local links, {input_count} Rust/package inputs checked.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
