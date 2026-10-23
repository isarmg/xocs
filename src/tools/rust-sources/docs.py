#!/usr/bin/env python3
"""Render local dependency documentation indexes from current source ownership."""

import argparse
import json
from pathlib import Path
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / 'docs/dependencies'


def link(label, target):
    return f'[{label}]({target})'


def source_entry(directory, manifest):
    library = manifest.get('lib', {})
    path = directory / library.get('path', 'src/lib.rs')
    if path.is_file():
        return path
    for binary in manifest.get('bin', []):
        path = directory / binary.get('path', 'src/main.rs')
        if path.is_file():
            return path
    return directory / 'src/main.rs'


def render(root=ROOT):
    policy = json.loads((root / 'src/tools/rust-sources/workspaces.json').read_text())
    origins = {item['id']: item for item in json.loads(
        (root / 'compliance/provenance/origins.json').read_text())['packages']}
    owners = {}
    catalog = ['# 本地包与文档目录', '',
               '此页由 `python3 src/tools/rust-sources/docs.py` 按当前 Cargo 包和源码所有者生成。'
               '先阅读[依赖指南](README.md)了解支持范围；具体 API 以本地 Rust 文档为准。', '']
    groups = [('project', policy['owned_sources'])]
    layers = sorted({item['layer'] for item in policy['maintained_sources']})
    groups.extend((layer, [item for item in policy['maintained_sources']
                           if item['layer'] == layer]) for layer in layers)
    for layer, sources in groups:
        catalog.extend([f'## {layer}', '',
                        '| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |',
                        '| --- | --- | --- | --- | --- |'])
        for item in sources:
            directory = root / item['directory']
            manifest = tomllib.loads((directory / 'Cargo.toml').read_text())
            name = manifest['package']['name']
            entry = source_entry(directory, manifest)
            if not entry.is_file():
                raise ValueError(f'API entry missing: {entry.relative_to(root)}')
            docs = [path for path in directory.rglob('*.md')
                    if path.is_file() and not path.is_symlink()
                    and not any(part in {'target', 'node_modules', 'dist', 'pkg', '.git'}
                                for part in path.relative_to(directory).parts)]
            # A project member can contain another member, whose docs belong to that package.
            nested = [root / source['directory'] for source in policy['owned_sources']
                      if source['directory'] != item['directory']
                      and source['directory'].startswith(item['directory'] + '/')]
            docs = sorted(path for path in docs
                          if not any(path.is_relative_to(child) for child in nested))
            local = '<br>'.join(link(str(path.relative_to(directory)),
                                     '../../' + path.relative_to(root).as_posix())
                                for path in docs) or 'Rust 文档注释'
            references = []
            for identity in item.get('origins', []):
                origin = origins[identity]
                dirname = f"{origin['name']}-{origin['version']}"
                readme = root / 'docs/dependencies/origins' / dirname / 'README.md'
                if not readme.is_file():
                    raise ValueError(f'local origin guide missing: {readme.relative_to(root)}')
                references.append(link(identity, f'origins/{dirname}/README.md'))
                owners.setdefault(identity, []).append((name, item['directory']))
            catalog.append('| ' + ' | '.join([
                link(name, '../../' + item['directory'] + '/Cargo.toml'),
                item['purpose'].replace('|', '\\|'),
                link(entry.name, '../../' + entry.relative_to(root).as_posix()),
                local, '<br>'.join(references) or '本项目实现']) + ' |')
        catalog.append('')
    source_index = ['# 原始来源参考', '',
                    '此目录收录当前实现所用来源的本地说明。原始 README 的库名、安装方式、'
                    '平台、特性和链接描述原始发布版本；本项目的入口、支持范围和包组合以'
                    '[依赖指南](../README.md)及[本地包目录](../catalog.md)为准。', '',
                    '没有独立 README 的来源提供本地源码入口。只保留使用、设计和格式说明；'
                    '原始完整档案及法律材料位于 `compliance/`。', '',
                    '| 来源 | 当前所有者 | 本地资料 | 许可证 |',
                    '| --- | --- | --- | --- |']
    for identity in sorted(owners):
        origin = origins[identity]
        dirname = f"{origin['name']}-{origin['version']}"
        directory = root / 'docs/dependencies/origins' / dirname
        documents = sorted(path for path in directory.iterdir() if path.is_file())
        source_index.append('| ' + ' | '.join([
            f'`{identity}`',
            '<br>'.join(link(name, '../../../' + directory + '/Cargo.toml')
                       for name, directory in owners[identity]),
            '<br>'.join(link(path.name, dirname + '/' + path.name) for path in documents),
            '<br>'.join(link(Path(path).name, '../../../' + path)
                       for path in origin['license_materials']) or origin['license']]) + ' |')
    source_index.append('')
    return {'catalog.md': '\n'.join(catalog),
            'origins/README.md': '\n'.join(source_index)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='check without writing')
    args = parser.parse_args()
    try:
        outputs = render()
        stale = []
        for name, contents in outputs.items():
            path = DOCS / name
            if args.check:
                if not path.is_file() or path.read_text() != contents:
                    stale.append(str(path.relative_to(ROOT)))
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(contents)
        if stale:
            raise ValueError('stale documentation indexes: ' + ', '.join(stale)
                             + '; run python3 src/tools/rust-sources/docs.py')
    except (OSError, ValueError, KeyError) as error:
        print(f'dependency documentation: {error}', file=sys.stderr)
        return 1
    print('dependency documentation indexes ' + ('verified' if args.check else 'generated'))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
