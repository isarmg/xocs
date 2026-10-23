#!/usr/bin/env python3
"""Verify Blog's local Rust sources, immutable imports and offline Cargo closure."""
import argparse
from collections import deque
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
POLICY = 'tools/rust-sources/workspaces.json'
CATALOG = 'compliance/rust-sources.json'
ORIGINS = 'compliance/provenance/origins.json'
DOC = 'docs/THIRD_PARTY_RUST_SOURCES.md'
LOCAL_MODIFICATIONS = 'compliance/provenance/local-modifications.json'
MIGRATIONS = 'compliance/provenance/source-migrations.json'
ARCHITECTURE_DOC = 'docs/RUST_SOURCE_ARCHITECTURE.md'
TARGETS = {'x86_64-unknown-linux-gnu', 'wasm32-unknown-unknown'}
HEX = re.compile(r'[0-9a-f]{64}')
SKIP_ROOTS = ('.git', 'target', '.local-run', 'web/node_modules', 'web/dist', 'web/pkg')


def digest(value):
    return hashlib.sha256(value).hexdigest()


def encoded(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()


def read_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f'duplicate JSON field: {key}')
            result[key] = value
        return result
    return json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=unique)


def safe(root, relative, *, allow_root=False):
    if allow_root and relative == '.':
        return root
    if (not isinstance(relative, str) or not relative or '\\' in relative
            or any(part in ('', '.', '..') or ':' in part for part in relative.split('/'))
            or Path(relative).is_absolute()):
        raise ValueError(f'unsafe repository path: {relative}')
    current = root
    for part in Path(relative).parts:
        current /= part
        if current.is_symlink():
            raise ValueError(f'repository path uses a symlink: {relative}')
    return current


def relative(root, path):
    value = Path(os.path.normpath(path))
    if not value.is_relative_to(root):
        raise ValueError(f'external source path: {path}')
    result = value.relative_to(root).as_posix()
    safe(root, result, allow_root=True)
    return result


def files(root, directory, *, excluded=()):
    base = safe(root, directory, allow_root=True)
    if not base.is_dir():
        raise ValueError(f'missing source directory: {directory}')
    result = {}
    for current, children, names in os.walk(base, followlinks=False):
        parent = Path(current)
        for name in children + names:
            path = parent / name
            name_relative = path.relative_to(root).as_posix()
            if any(name_relative == item or name_relative.startswith(item + '/') for item in excluded):
                if name in children:
                    children.remove(name)
                continue
            if path.is_symlink():
                raise ValueError(f'source symlink: {name_relative}')
            if name in children:
                continue
            if not path.is_file():
                raise ValueError(f'source special file: {name_relative}')
            result[path.relative_to(base).as_posix()] = digest(path.read_bytes())
    return dict(sorted(result.items()))


def generated_roots(documents):
    # Legacy standalone builds can leave target/ beside any active manifest.
    # Match only package roots; src/target in crates such as cc is source code.
    package_outputs = (str(Path(name).parent / 'target')
                       for name, document in documents.items() if 'package' in document)
    return tuple(sorted({*SKIP_ROOTS, *package_outputs}))


def dependencies(root, manifest_relative, document, workspace):
    sections = [document, *document.get('target', {}).values()]
    if manifest_relative == 'Cargo.toml':
        sections.append(document['workspace'])
    for section in sections:
        for kind in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            for alias, declaration in section.get(kind, {}).items():
                base = safe(root, manifest_relative).parent
                dependency = declaration
                if isinstance(dependency, dict) and dependency.get('workspace') is True:
                    dependency = workspace.get(alias)
                    base = root
                    if not isinstance(dependency, dict):
                        raise ValueError(f'missing workspace dependency: {alias}')
                    dependency = {**dependency, **{key: value for key, value in declaration.items() if key != 'workspace'}}
                if (not isinstance(dependency, dict) or not isinstance(dependency.get('path'), str)
                        or any(key in dependency for key in ('git', 'registry', 'registry-index', 'branch', 'tag', 'rev'))):
                    raise ValueError(f'non-path dependency: {manifest_relative} {kind} {alias}')
                target = relative(root, base / dependency['path'])
                target_manifest = ('Cargo.toml' if target == '.' else target + '/Cargo.toml')
                if not safe(root, target_manifest).is_file():
                    raise ValueError(f'missing local dependency: {manifest_relative} {alias} -> {target_manifest}')
                yield alias, dependency, target_manifest


def static_structure(root, policy):
    document = tomllib.loads(safe(root, 'Cargo.toml').read_text())
    if 'workspace' not in document:
        raise ValueError('one root Cargo workspace is required')
    if document.get('patch') or document.get('replace'):
        raise ValueError('registry patch/replacement tables are not part of the path-only source policy')
    workspace = document['workspace'].get('dependencies', {})
    pending = {'Cargo.toml'}
    for pattern in document['workspace'].get('members', []):
        if Path(pattern).is_absolute() or '..' in Path(pattern).parts:
            raise ValueError(f'external workspace member: {pattern}')
        matches = sorted(root.glob(pattern))
        if not matches:
            raise ValueError(f'missing workspace member pattern: {pattern}')
        for path in matches:
            name = relative(root, path)
            pending.add(('Cargo.toml' if name == '.' else name + '/Cargo.toml'))
    # Explicit source ownership also catches retained optional source packages.
    pending.update(item['directory'] + '/Cargo.toml' for item in policy['owned_sources'] if item['directory'] != '.')
    foundation = safe(root, policy['foundation'])
    if not foundation.is_dir():
        raise ValueError('missing local foundation sources')
    if 'maintained_sources' in policy:
        for item in policy['maintained_sources']:
            path = item['directory']
            safe(root, path)
            if not path.startswith(policy['foundation'] + '/'):
                raise ValueError(f'maintained dependency outside foundation: {path}')
            pending.add(path + '/Cargo.toml')
    else:
        for child in foundation.iterdir():
            if child.is_symlink() or not child.is_dir() or not (child / 'Cargo.toml').is_file():
                raise ValueError(f'foundation entries must be local Cargo packages: {child}')
            pending.add(child.relative_to(root).as_posix() + '/Cargo.toml')
    documents, edges = {}, {}
    while pending:
        name = min(pending)
        pending.remove(name)
        if name in documents:
            continue
        current = tomllib.loads(safe(root, name).read_text())
        if name != 'Cargo.toml' and 'workspace' in current:
            raise ValueError(f'nested Cargo workspace: {name}')
        if current.get('patch') or current.get('replace'):
            raise ValueError(f'nested registry patch/replacement: {name}')
        if 'package' not in current and name != 'Cargo.toml':
            raise ValueError(f'manifest does not declare a package: {name}')
        documents[name] = current
        edges[name] = list(dependencies(root, name, current, workspace))
        pending.update(target for _, _, target in edges[name] if target not in documents)
    verify_features(documents, edges)
    # Exclude immutable archive material and generated outputs, never arbitrary
    # Rust source subdirectories named target (for example cc/src/target).
    outputs = generated_roots(documents)
    for current, children, names in os.walk(root, followlinks=False):
        parent = Path(current)
        here = parent.relative_to(root).as_posix()
        for name in children[:]:
            item = (parent / name).relative_to(root).as_posix()
            if any(item == path or item.startswith(path + '/') for path in (*outputs, 'compliance', 'docs/dependencies/origins')):
                children.remove(name)
                continue
            if (parent / name).is_symlink():
                raise ValueError(f'source directory symlink: {item}')
            if name == '.git':
                raise ValueError(f'nested Git checkout: {item}')
        if here != '.' and '.git' in names:
            raise ValueError(f'nested Git checkout file: {here}')
        if here != '.' and 'Cargo.lock' in names:
            raise ValueError(f'nested Cargo.lock: {here}')
        if 'Cargo.toml' in names:
            name = ('Cargo.toml' if here == '.' else here + '/Cargo.toml')
            # Test fixtures in imported archives are source inputs, not new
            # build entrypoints. Maintained project manifests must be connected.
            if not name.startswith(policy['foundation'] + '/') and name not in documents:
                raise ValueError(f'unregistered project Cargo manifest: {name}')
    config = tomllib.loads(safe(root, '.cargo/config.toml').read_text())
    if config.get('net', {}).get('offline') is not True:
        raise ValueError('Cargo must default to offline mode')
    source = config.get('source', {})
    replacement = source.get('crates-io', {}).get('replace-with')
    empty = source.get(replacement, {}).get('directory')
    if not isinstance(empty, str):
        raise ValueError('an empty local registry fallback is required')
    empty_path = safe(root, empty)
    if not empty_path.is_dir() or any(path.is_symlink() or not path.is_file() or path.name == 'Cargo.toml' for path in empty_path.iterdir()):
        raise ValueError('registry fallback must contain no package directories or links')
    lock = tomllib.loads(safe(root, 'Cargo.lock').read_text())
    for package in lock.get('package', []):
        if package.get('source') is not None or package.get('checksum') is not None:
            raise ValueError(f'root lock contains an external source: {package["name"]}')
    profiles = policy['profiles']
    if not profiles or len({item['id'] for item in profiles}) != len(profiles):
        raise ValueError('source closure profiles must be unique and nonempty')
    if {item['target'] for item in profiles} != TARGETS:
        raise ValueError('profiles must cover Linux amd64 and Web WASM only')
    for item in profiles:
        if item['target'] not in TARGETS or item['role'] not in ('runtime', 'compile') or not item['manifests']:
            raise ValueError(f'invalid source profile: {item["id"]}')
        if not isinstance(item.get('include_dev', False), bool):
            raise ValueError(f'source profile include_dev must be boolean: {item["id"]}')
        if any(name not in documents for name in item['manifests']):
            raise ValueError(f'profile references unknown manifest: {item["id"]}')
    verify_platform_guard(root, policy, documents)
    return documents


def verify_platform_guard(root, policy, documents):
    guard = policy.get('platform_guard')
    if not isinstance(guard, str) or not guard:
        raise ValueError('source policy must declare the shared platform guard')
    if not safe(root, guard).is_file():
        raise ValueError('missing shared platform guard')
    if not any(item['directory'] == '.' or guard.startswith(item['directory'] + '/')
               for item in policy['owned_sources']):
        raise ValueError('shared platform guard needs a current project source owner')
    for item in policy['owned_sources']:
        directory = item['directory']
        manifest = 'Cargo.toml' if directory == '.' else directory + '/Cargo.toml'
        package = documents[manifest]['package']
        build = package.get('build', 'build.rs')
        if not isinstance(build, str):
            raise ValueError(f'project package disables its platform guard: {directory}')
        script = relative(root, safe(root, directory, allow_root=True) / build)
        if script == guard:
            continue
        path = safe(root, script)
        if not path.is_file():
            raise ValueError(f'project package lacks a platform build script: {directory}')
        source = path.read_text()
        module = re.search(r'#\[path\s*=\s*"([^"]+)"\]\s*mod\s+platform_contract\s*;', source)
        call = re.search(r'\bfn\s+main\s*\(\s*\)\s*\{\s*platform_contract::check\(\);', source)
        if (module is None or call is None
                or relative(root, path.parent / module[1]) != guard):
            raise ValueError(f'build script must invoke the shared platform guard first: {directory}')


def verify_features(documents, edges):
    features = {}
    for name, document in documents.items():
        declared = document.get('features', {})
        hidden = {value[4:] for values in declared.values() for value in values if value.startswith('dep:')}
        features[name] = set(declared) | {alias for alias, dependency, _ in edges[name] if alias not in hidden and dependency.get('optional')}
    for name, document in documents.items():
        aliases = {}
        for alias, dependency, target in edges[name]:
            aliases.setdefault(alias, []).append((dependency, target))
            for feature in dependency.get('features', []):
                if feature not in features[target]:
                    raise ValueError(f'unknown dependency feature: {name} {alias}/{feature}')
        for feature, values in document.get('features', {}).items():
            for value in values:
                if value.startswith('dep:'):
                    valid = any(item.get('optional') for item, _ in aliases.get(value[4:], []))
                elif '/' in value:
                    alias, selected = value.split('/', 1)
                    weak = alias.endswith('?')
                    alias = alias.removesuffix('?')
                    valid = (alias in aliases and all(selected in features[target] for _, target in aliases[alias])
                             and (not weak or any(item.get('optional') for item, _ in aliases[alias])))
                else:
                    valid = value in features[name]
                if not valid:
                    raise ValueError(f'dangling feature edge: {name} {feature} -> {value}')


def archive_files(path, name, version):
    if path.stat().st_size > 128 * 1024 * 1024:
        raise ValueError(f'upstream archive exceeds its bound: {path}')
    prefix, seen, result, total = f'{name}-{version}', set(), {}, 0
    with tarfile.open(path, 'r:gz') as archive:
        for index, member in enumerate(archive):
            raw = member.name.rstrip('/') if member.isdir() else member.name
            parts = raw.split('/')
            if (index >= 100000 or not raw or raw.startswith('/') or '\\' in raw
                    or any(part in ('', '.', '..') or ':' in part for part in parts)
                    or parts[0] != prefix or raw in seen):
                raise ValueError(f'unsafe/duplicate upstream archive path: {member.name}')
            seen.add(raw)
            if not member.isfile() and not member.isdir():
                raise ValueError(f'upstream archive links/special files are forbidden: {member.name}')
            if member.size < 0 or member.size > 512 * 1024 * 1024:
                raise ValueError('upstream archive file exceeds its bound')
            if len(parts) == 1:
                if not member.isdir():
                    raise ValueError('archive prefix must be a directory')
                continue
            if member.isdir():
                continue
            total += member.size
            if total > 512 * 1024 * 1024:
                raise ValueError('upstream archive exceeds its unpacked bound')
            source = archive.extractfile(member)
            if source is None:
                raise ValueError('missing upstream archive file bytes')
            with source:
                result['/'.join(parts[1:])] = hashlib.file_digest(source, 'sha256').hexdigest()
    return dict(sorted(result.items()))


def original_sources(root):
    document = read_json(safe(root, ORIGINS))
    if document.get('schema') != 1 or not isinstance(document.get('packages'), list):
        raise ValueError('unsupported immutable origin schema')
    records, materials = {}, {}
    for record in document['packages']:
        name, version = record['name'], record['version']
        identity = f'{name}@{version}'
        if (record['id'] != identity or identity in records or not re.fullmatch(r'[A-Za-z0-9_-]+', name)
                or not re.fullmatch(r'[0-9][A-Za-z0-9.+-]*', version)):
            raise ValueError(f'invalid/duplicate upstream identity: {identity}')
        upstream = record['upstream']
        expected_url = f'https://static.crates.io/crates/{name}/{name}-{version}.crate'
        if ((upstream['name'], upstream['version'], upstream['archive_url']) != (name, version, expected_url)
                or not HEX.fullmatch(upstream['archive_sha256'])):
            raise ValueError(f'invalid exact upstream archive identity: {identity}')
        archive = safe(root, upstream['archive'])
        if not upstream['archive'].startswith('compliance/provenance/archives/') or not archive.is_file():
            raise ValueError(f'missing repository-controlled original archive: {identity}')
        actual = digest(archive.read_bytes())
        if actual != upstream['archive_sha256']:
            raise ValueError(f'original archive SHA-256 differs: {identity}')
        original = record['original_files_sha256']
        if (not original or any(not HEX.fullmatch(value) for value in original.values())
                or archive_files(archive, name, version) != original):
            raise ValueError(f'original upstream file receipts differ: {identity}')
        materials[upstream['archive']] = actual
        for material in record.get('license_materials', []):
            value = digest(safe(root, material).read_bytes())
            if value not in original.values():
                raise ValueError(f'license is not an original archive file: {material}')
            materials[material] = value
        records[identity] = record
    for directory in ('compliance/provenance', 'compliance/licenses', 'compliance/web-origins', 'compliance/native-sources'):
        if safe(root, directory).is_dir():
            for name, value in files(root, directory).items():
                path = directory + '/' + name
                if path not in (ORIGINS, LOCAL_MODIFICATIONS, MIGRATIONS):
                    materials[path] = value
    return dict(sorted(records.items())), dict(sorted(materials.items()))


def local_modifications(root, historical_directories=None):
    path = safe(root, LOCAL_MODIFICATIONS)
    if not path.exists():
        return None
    record = read_json(path)
    if record.get('schema') != 1 or not isinstance(record.get('changes'), list):
        raise ValueError('unsupported local modification ledger schema')
    for item in record['changes']:
        if not isinstance(item, dict) or not isinstance(item.get('reason'), str) or not item['reason'].strip():
            raise ValueError('local modification entries need a nonempty reason')
        safe(root, item['directory'])
        if historical_directories is not None and item['directory'] not in historical_directories:
            raise ValueError(f'local modification directory has no current or migration identity: {item["directory"]}')
        names = [item['file']] if 'file' in item else []
        if 'files' in item:
            if not isinstance(item['files'], list):
                raise ValueError('local modification files must be a list')
            names.extend(item['files'])
        for name in names:
            if not isinstance(name, str):
                raise ValueError('local modification file must be a relative path')
            safe(root, item['directory'] + '/' + name)
    return record


def package_identity(root, manifest, workspace):
    package = manifest['package']
    result = {}
    for name in ('name', 'version'):
        value = package.get(name, '0.0.0') if name == 'version' else package[name]
        if isinstance(value, dict) and value.get('workspace') is True:
            value = workspace[name]
        if not isinstance(value, str):
            raise ValueError(f'invalid current Cargo package {name}')
        result[name] = value
    return result


def identity(package):
    origins = package.get('origins')
    if origins is None:
        origins = [package['origin']] if package.get('origin') else []
    return {key: package[key] for key in ('directory', 'name', 'version')} | {'origins': origins}


def source_migrations(root, origins):
    path = safe(root, MIGRATIONS)
    if not path.exists():
        return None, None
    ledger = read_json(path)
    if (set(ledger) != {'schema', 'baseline', 'events'} or ledger['schema'] != 1
            or not isinstance(ledger['baseline'], list) or not isinstance(ledger['events'], list)):
        raise ValueError('unsupported source migration ledger schema')

    def identities(entries):
        if not isinstance(entries, list):
            raise ValueError('migration identities must be a list')
        result = {}
        for item in entries:
            if not isinstance(item, dict) or set(item) != {'directory', 'name', 'version', 'origins'}:
                raise ValueError('migration identities need directory/name/version/origins')
            safe(root, item['directory'])
            if (item['directory'] in result or not isinstance(item['name'], str)
                    or not re.fullmatch(r'[A-Za-z0-9_-]+', item['name'])
                    or not isinstance(item['version'], str) or not re.fullmatch(r'[0-9][A-Za-z0-9.+-]*', item['version'])):
                raise ValueError('invalid or duplicate migration package identity')
            lineage = item['origins']
            if (not isinstance(lineage, list) or any(not isinstance(value, str) for value in lineage)
                    or lineage != sorted(set(lineage)) or any(value not in origins for value in lineage)):
                raise ValueError('migration origins must be unique sorted original identities')
            result[item['directory']] = item
        return result

    state = identities(ledger['baseline'])
    if not state:
        raise ValueError('source migration baseline cannot be empty')
    historical_directories = set(state)
    seen_origins = {value for item in state.values() for value in item['origins']}
    retired, event_ids = {}, set()
    for event in ledger['events']:
        if (not isinstance(event, dict) or set(event) not in
                ({'id', 'reason', 'from', 'to', 'retired_origins'},
                 {'id', 'reason', 'from', 'to', 'retired_origins', 'restored_origins'})
                or not isinstance(event['id'], str) or not re.fullmatch(r'[a-zA-Z0-9_-]+', event['id'])
                or event['id'] in event_ids or not isinstance(event['reason'], str) or not event['reason'].strip()):
            raise ValueError('source migration events need unique id and nonempty reason')
        event_ids.add(event['id'])
        consumed, produced = identities(event['from']), identities(event['to'])
        if not consumed and not produced:
            raise ValueError('empty source migration event')
        if any(state.get(directory) != item for directory, item in consumed.items()):
            raise ValueError(f'migration consumes a missing or changed identity: {event["id"]}')
        if any(directory in state and directory not in consumed for directory in produced):
            raise ValueError(f'migration overwrites an unconsumed identity: {event["id"]}')
        input_origins = {value for item in consumed.values() for value in item['origins']}
        output_origins = {value for item in produced.values() for value in item['origins']}
        introduced = output_origins - input_origins
        requested = event.get('restored_origins', [])
        if (not isinstance(requested, list) or any(not isinstance(value, str) for value in requested)
                or requested != sorted(set(requested)) or any(value not in origins for value in requested)):
            raise ValueError('restored origins must be unique sorted known original identities')
        restored = set(requested)
        if any(value not in retired for value in restored):
            raise ValueError('only explicitly retired original identities can be restored')
        if restored and introduced != restored:
            raise ValueError('restored origins must exactly match introduced lineage')
        for value in restored:
            original = origins[value]['upstream']
            archive = safe(root, original['archive'])
            if not archive.is_file() or digest(archive.read_bytes()) != original['archive_sha256']:
                raise ValueError(f'restored original archive is missing or changed: {value}')
        imported = introduced - restored
        if ((introduced & seen_origins) != restored or (imported and consumed)):
            raise ValueError('new origin lineage needs an explicit import event with empty from')
        before = {value for item in state.values() for value in item['origins']}
        state = {directory: item for directory, item in state.items() if directory not in consumed} | produced
        after = {value for item in state.values() for value in item['origins']}
        retirements = event['retired_origins']
        if not isinstance(retirements, list):
            raise ValueError('retired_origins must be a list')
        declared = {}
        for item in retirements:
            if (not isinstance(item, dict) or set(item) != {'id', 'reason'} or item['id'] in declared
                    or not isinstance(item['reason'], str) or not item['reason'].strip()):
                raise ValueError('retired origins need unique identity and explicit reason')
            declared[item['id']] = item['reason']
        if set(declared) != before - after:
            raise ValueError('migration silently loses lineage or declares an active origin retired')
        for value in restored:
            retired.pop(value)
        retired.update({name: {'event': event['id'], 'reason': reason} for name, reason in declared.items()})
        seen_origins.update(introduced)
        historical_directories.update(produced)
    if seen_origins != set(origins):
        raise ValueError('every original identity must enter migration history explicitly')
    return ledger, {'packages': state, 'retired_origins': dict(sorted(retired.items())),
                    'historical_directories': sorted(historical_directories)}


def source_packages(root, policy, documents, origins, migration_state=None):
    owned = {item['directory']: item for item in policy['owned_sources']}
    if len(owned) != len(policy['owned_sources']):
        raise ValueError('duplicate maintained source ownership')
    migrated = migration_state is not None
    if migrated:
        if 'maintained_sources' not in policy:
            raise ValueError('source migrations require explicit maintained_sources')
        for item in policy['maintained_sources']:
            path = item['directory']
            if path in owned or not path.startswith(policy['foundation'] + '/'):
                raise ValueError(f'ambiguous/non-foundation maintained source owner: {path}')
            owned[path] = item
        for item in owned.values():
            if 'include' in item:
                raise ValueError('migrated package receipts must cover complete source directories')
            for field in ('layer', 'purpose'):
                if not isinstance(item.get(field), str) or not item[field].strip():
                    raise ValueError(f'maintained package needs architectural {field}: {item["directory"]}')
            lineage = item.get('origins', [])
            if (not isinstance(lineage, list) or lineage != sorted(set(lineage))
                    or any(value not in origins for value in lineage)):
                raise ValueError('current source mappings need unique sorted original identities')
            if item in policy['maintained_sources'] and not lineage:
                raise ValueError('foundation sources must retain original lineage')
    else:
        if 'maintained_sources' in policy:
            raise ValueError('maintained_sources cannot bypass an explicit migration ledger')
        for record in origins.values():
            path = record['directory']
            if not path.startswith(policy['foundation'] + '/') or path in owned:
                raise ValueError(f'ambiguous/non-foundation registry source owner: {path}')
            owned[path] = {'directory': path, 'origin': record['id']}
    manifests = {str(Path(name).parent): name for name, doc in documents.items() if 'package' in doc}
    if set(manifests) != set(owned):
        raise ValueError(f'source ownership does not cover every active package: {sorted(set(manifests) ^ set(owned))}')
    result = {}
    workspace = documents['Cargo.toml']['workspace'].get('package', {})
    outputs = generated_roots(documents)
    for directory, spec in sorted(owned.items()):
        current = package_identity(root, documents[manifests[directory]], workspace)
        if not migrated and 'origin' in spec:
            origin = origins[spec['origin']]
            if (current['name'], current['version']) != (origin['name'], origin['version']):
                raise ValueError(f'current foundation package silently changed identity: {directory}')
        exclusions = tuple(other for other in owned if other != directory and other != '.'
                           and (directory == '.' or other.startswith(directory + '/')))
        if 'include' in spec:
            inventory = {}
            for item in spec['include']:
                global_path = item if directory == '.' else directory + '/' + item
                target = safe(root, global_path)
                if target.is_dir():
                    for name, value in files(root, global_path, excluded=(*outputs, *exclusions)).items():
                        inventory[item + '/' + name] = value
                elif target.is_file():
                    inventory[item] = digest(target.read_bytes())
                else:
                    raise ValueError(f'missing maintained package source input: {global_path}')
        else:
            inventory = files(root, directory, excluded=(*outputs, *exclusions))
        if not inventory or 'Cargo.toml' not in inventory:
            raise ValueError(f'package inventory is missing its Cargo manifest: {directory}')
        result[directory] = dict(directory=directory, **current, origin=spec.get('origin'),
                                 files_sha256=dict(sorted(inventory.items())))
        if migrated:
            lineage = spec.get('origins', [])
            direct = {str(Path(target).parent) for _, _, target in dependencies(root, manifests[directory], documents[manifests[directory]], documents['Cargo.toml']['workspace'].get('dependencies', {}))}
            result[directory].update(origins=lineage, origin=lineage[0] if len(lineage) == 1 else None,
                                     layer=spec['layer'], purpose=spec['purpose'], dependencies=sorted(direct))
    if migrated:
        current = {directory: identity(item) for directory, item in result.items()}
        if current != migration_state['packages']:
            raise ValueError('migration terminal identities differ from actual manifests/source mappings')
        # Every foundation file must belong to an active package, including
        # retained source fixtures. Layer directories cannot hide orphan code.
        for name in files(root, policy['foundation'], excluded=outputs):
            path = policy['foundation'] + '/' + name
            if not any(path.startswith(directory + '/') for directory in owned):
                raise ValueError(f'unowned foundation source file: {path}')
    return result


def cargo_metadata(root, target, cargo_home):
    if target not in TARGETS:
        raise ValueError(f'unsupported source graph target: {target}')
    environment = os.environ.copy()
    environment.update(CARGO_HOME=str(cargo_home), CARGO_NET_OFFLINE='true')
    process = subprocess.run(['cargo', 'metadata', '--locked', '--offline', '--format-version', '1',
                              '--manifest-path', str(root / 'Cargo.toml'), '--filter-platform', target],
                             cwd=root, env=environment, capture_output=True, text=True, encoding='utf-8')
    if process.returncode:
        raise ValueError(f'empty-Cargo-home offline metadata failed for {target}: {process.stderr.strip()}')
    metadata = json.loads(process.stdout)
    if Path(metadata['workspace_root']) != root:
        raise ValueError('Cargo resolved another workspace root')
    for package in metadata['packages']:
        name = relative(root, Path(package['manifest_path']))
        if package.get('source') is not None:
            raise ValueError(f'external resolved Cargo source: {package["name"]} {package["source"]}')
        safe(root, name)
        for item in package['targets']:
            relative(root, Path(item['src_path']))
    return metadata


def closure(metadata, manifests, role, *, include_dev=False, host_metadata=None):
    host_metadata = metadata if host_metadata is None else host_metadata
    if host_metadata['workspace_root'] != metadata['workspace_root']:
        raise ValueError('host and target metadata resolve different workspace roots')
    packages = {}
    for graph in (metadata, host_metadata):
        for item in graph['packages']:
            previous = packages.get(item['id'])
            if previous is not None and any(previous.get(field) != item.get(field)
                                            for field in ('manifest_path', 'name', 'version', 'source')):
                raise ValueError(f'host and target metadata package identity differs: {item["id"]}')
            packages[item['id']] = item
    nodes = {item['id']: item for item in metadata['resolve']['nodes']}
    host_nodes = {item['id']: item for item in host_metadata['resolve']['nodes']}
    workspace = set(metadata['workspace_members'])
    root = Path(metadata['workspace_root'])
    selected = {item['id'] for item in packages.values() if item['id'] in workspace
                and Path(item['manifest_path']).relative_to(root).as_posix() in manifests}
    if {Path(packages[item]['manifest_path']).relative_to(root).as_posix() for item in selected} != set(manifests):
        raise ValueError(f'profile is missing workspace roots: {manifests}')
    queue, used = deque((item, role) for item in selected), {}
    while queue:
        identity, current_role = queue.popleft()
        if any('proc-macro' in item['kind'] for item in packages[identity]['targets']):
            current_role = 'compile'
        if current_role in used.setdefault(identity, set()):
            continue
        used[identity].add(current_role)
        # Build scripts and procedural macros execute on the Linux host even
        # while the consuming library targets WASM. Their target-conditional
        # dependencies must therefore come from the host-filtered graph.
        current_nodes = host_nodes if current_role == 'compile' else nodes
        for edge in current_nodes[identity]['deps']:
            for kind in edge['dep_kinds']:
                # Cargo builds development dependencies for the selected test
                # roots, not for every library reached through those roots.
                if kind['kind'] == 'dev' and (not include_dev or identity not in selected):
                    continue
                queue.append((edge['pkg'], 'compile' if current_role == 'compile' or kind['kind'] == 'build' else 'runtime'))
    result = {}
    for identity, roles in used.items():
        path = Path(packages[identity]['manifest_path']).parent.relative_to(root).as_posix()
        features = {feature for role in roles
                    for feature in (host_nodes if role == 'compile' else nodes)[identity]['features']}
        result[path] = dict(name=packages[identity]['name'], version=packages[identity]['version'],
                            roles=sorted(roles), features=sorted(features))
    return dict(sorted(result.items()))


def resolutions(root, policy):
    cache, result = {}, []
    with tempfile.TemporaryDirectory(prefix='blog-empty-cargo-') as temporary:
        # Every supported target can reach a host build/proc-macro dependency.
        host_target = 'x86_64-unknown-linux-gnu'
        cache[host_target] = cargo_metadata(root, host_target, Path(temporary))
        for profile in policy['profiles']:
            target = profile['target']
            if target not in cache:
                cache[target] = cargo_metadata(root, target, Path(temporary))
            packages = closure(cache[target], profile['manifests'], profile['role'],
                               include_dev=profile.get('include_dev', False),
                               host_metadata=cache[host_target])
            result.append(dict(profile=profile['id'], target=target, packages=packages))
    return result


def table_row(values):
    return '| ' + ' | '.join(str(value).replace('|', '\\|').replace('\n', ' ') for value in values) + ' |'


def migrated_document(catalog, policy):
    lines = ['# 第三方 Rust 源码清单', '',
             '本表由当前源码映射生成，列出各包的原始名称、版本和许可证。内部 Cargo 包名和版本用于本项目，原始发布身份用于来源和安全审计。',
             '完整原始归档、逐文件摘要和许可文本保存在 `compliance/`，当前源码摘要保存在 `compliance/rust-sources.json`。', '',
             '阅读实现请从[依赖文档](dependencies/README.md)进入；构建入口和直接消费者见 [Cargo 依赖关系](RUST_SOURCE_ARCHITECTURE.md)，校验命令见[源码布局与校验](source-layout.md)。', '',
             '## 当前源码与原始来源', '',
             '| 当前用途路径 | 内部 Cargo 包 | 原始身份 | 原始许可 |', '| --- | --- | --- | --- |']
    for directory, item in sorted(catalog['packages'].items()):
        records = [catalog['origins'][value] for value in item['origins']]
        origin_ids = ', '.join(item['origins']) or '项目实现或已有独立来源记录'
        licenses = []
        for record in records:
            value = record['id'] + ': ' + (record.get('license') or '未声明')
            if not record.get('license_materials'):
                value += '；发布包无独立许可文本，待复核'
            licenses.append(value)
        lines.append(table_row([directory, item['name'], origin_ids, '; '.join(licenses) or '沿用组件独立法律材料']))
    return '\n'.join(lines) + '\n'


def architecture_document(catalog, policy):
    packages = catalog['packages']
    profiles = {directory: [] for directory in packages}
    for resolution in catalog['resolutions']:
        for directory in resolution['packages']:
            if directory in profiles:
                profiles[directory].append(resolution['profile'])
    consumers = {directory: [] for directory in packages}
    for directory, item in packages.items():
        for dependency in item['dependencies']:
            consumers[dependency].append(directory)
    layers = {}
    for directory, item in packages.items():
        layers.setdefault(item['layer'], []).append(directory)
    lines = ['# Cargo 依赖关系', '',
             '本表由 `workspaces.json` 的职责映射、当前 manifest 本地路径边和已验证的构建入口生成。设计说明见[架构](architecture.md)，各层用法和本地参考资料见[依赖文档](dependencies/README.md)。',
             '层名用于说明职责，不自动证明依赖方向或无环。依赖列包含目标条件、optional、dev 和 build 声明；入口列来自统一 workspace 特性图的保守闭包，不能当作精确链接结果。',
             '一个当前包可以整合多个原始来源；一个原始来源也可以由多个用途包分别实现。', '',
             '## 构建入口', '', '| 入口 | 目标 | 职责 | 测试根 dev 依赖 | 根包 |', '| --- | --- | --- | --- | --- |']
    for item in policy['profiles']:
        lines.append(table_row([item['id'], item['target'], item.get('purpose', item['role']),
                                '包含' if item.get('include_dev', False) else '不包含', ', '.join(item['manifests'])]))
    lines += ['', '## 职责层及实际声明关系', '', '| 职责层 | 包数 | 构建入口 | 依赖的其他层 |', '| --- | --- | --- | --- |']
    for layer, directories in sorted(layers.items()):
        selected = sorted({profile for directory in directories for profile in profiles[directory]})
        dependencies = sorted({packages[target]['layer'] for directory in directories for target in packages[directory]['dependencies'] if packages[target]['layer'] != layer})
        lines.append(table_row([layer, len(directories), ', '.join(selected) or '当前入口未选用', ', '.join(dependencies) or '无跨层声明']))
    for layer, directories in sorted(layers.items()):
        lines += ['', '## ' + layer.replace('\n', ' '), '', '| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |', '| --- | --- | --- | --- | --- |']
        for directory in directories:
            item = packages[directory]
            lines.append(table_row([item['name'] + ' (`' + directory + '`)', item['purpose'], ', '.join(profiles[directory]) or '当前入口未选用', ', '.join(sorted(consumers[directory])) or '入口或保留实现', ', '.join(item['origins']) or '项目/独立来源']))
    lines += ['', '原始档案和许可清单见 [第三方 Rust 来源](THIRD_PARTY_RUST_SOURCES.md)。内部版本可能省略并由 Cargo 记为 0.0.0；这不改变原始发布版本或安全审计身份。', '']
    return '\n'.join(lines)


def document(catalog, policy):
    if 'source_migrations' in catalog:
        return migrated_document(catalog, policy)
    lines = ['# Rust 第三方源码与离线构建', '',
             '活动 Rust 包属于同一根 Cargo workspace，只有根 Cargo.lock；依赖均使用仓库内 path。',
             '原始发布包、逐文件原始摘要和许可证在 compliance/provenance/origins.json 中登记。',
             '当前本地改造后的文件摘要、manifest、锁文件和目标闭包见 compliance/rust-sources.json。', '',
             '运行 `python3 tools/rust-sources/manage.py refresh` 显式登记本地变化并审查差异；',
             '`python3 tools/rust-sources/manage.py verify --metadata` 只核验，不接受新哈希。',
             '已有原始身份、归档和许可材料不得通过 refresh 改写或删除。当前版本不自动办理退役、改名或上游替换，需单独审查迁移工具。',
             '后续本地补丁记录于 compliance/provenance/local-modifications.json；refresh 只允许追加日志，不允许改写或删除历史项。', '',
             '闭包来自统一 workspace 特性图；runtime 是 normal 边可达候选，compile 包括 build/proc-macro 子图。',
             '跳过直接 dev 边不等于隔离全部开发特性，图仍是保守超集；不是精确链接证明或源码可删除证明。',
             '空 Cargo home 的 metadata 验证依赖解析不使用外部 Rust 源码，不替代编译、单元测试、浏览器或安装验收。',
             'Rust 工具链和 targets、系统 C/C++ 编译器及原生 SQLite/密码实现的构建材料仍需独立说明；源码内置不代表无漏洞或已发布。', '',
             '## 当前构建入口', '', '| 入口 | 目标 | 用途 |', '| --- | --- | --- |']
    for profile in policy['profiles']:
        lines.append(f'| {profile["id"]} | {profile["target"]} | {profile.get("purpose", profile["role"])} |')
    lines += ['', '## 原始来源与当前源码', '', '| 当前路径 | 原始身份 | 许可 | 本地变化 |', '| --- | --- | --- | --- |']
    for directory, item in sorted(catalog['packages'].items()):
        origin = catalog['origins'].get(item['origin']) if item['origin'] else None
        if origin:
            notes = [item['reason'] for item in (catalog.get('local_modifications') or {}).get('changes', []) if item['directory'] == directory]
            changes = '; '.join([*origin.get('changes', []), *notes]) or '当前逐文件摘要单独记录本地改造'
            license_value = origin.get('license') or '未声明'
            if not origin.get('license_materials'):
                license_value += '；发布包无独立许可文本，待复核'
            cells = [directory, origin['id'], license_value, changes]
        else:
            cells = [directory, '项目实现或已单独记录来源的整合代码', '沿用该组件的真实许可与既有来源记录', '当前逐文件摘要单独记录']
        lines.append('| ' + ' | '.join(str(value).replace('|', '\\|').replace('\n', ' ') for value in cells) + ' |')
    return '\n'.join(lines) + '\n'


def snapshot(root):
    policy = read_json(safe(root, POLICY))
    if policy.get('schema') != 1:
        raise ValueError('unsupported source policy schema')
    documents = static_structure(root, policy)
    origins, materials = original_sources(root)
    migrations, migration_state = source_migrations(root, origins)
    packages = source_packages(root, policy, documents, origins, migration_state)
    modifications = local_modifications(root, migration_state['historical_directories'] if migration_state is not None else packages)
    result = dict(schema=1, policy_sha256=digest(safe(root, POLICY).read_bytes()),
                config_sha256=digest(safe(root, '.cargo/config.toml').read_bytes()),
                lockfiles_sha256={'Cargo.lock': digest(safe(root, 'Cargo.lock').read_bytes())},
                manifests_sha256={name: digest(safe(root, name).read_bytes()) for name in sorted(documents)},
                origins=origins, origin_records_sha256={name: digest(encoded(record)) for name, record in origins.items()},
                immutable_materials_sha256=materials, packages=packages,
                local_modifications=modifications,
                local_modifications_sha256=digest(safe(root, LOCAL_MODIFICATIONS).read_bytes()) if modifications is not None else None)
    guard = policy['platform_guard']
    result['platform_guard_sha256'] = {guard: digest(safe(root, guard).read_bytes())}
    if migrations is not None:
        result.update(source_migrations=migrations, source_migrations_sha256=digest(safe(root, MIGRATIONS).read_bytes()),
                      retired_origins=migration_state['retired_origins'])
    return result, policy


def preserve(previous, current):
    before = previous.get('local_modifications')
    after = current.get('local_modifications')
    if before is not None and (after is None or after['changes'][:len(before['changes'])] != before['changes']):
        raise ValueError('local modification history changed or disappeared; append new records instead')
    for field in ('origin_records_sha256', 'immutable_materials_sha256'):
        for name, expected in previous.get(field, {}).items():
            if current[field].get(name) != expected:
                raise ValueError(f'immutable original identity/material changed or disappeared: {name}')
    before_migrations = previous.get('source_migrations')
    after_migrations = current.get('source_migrations')
    if before_migrations is not None:
        if (after_migrations is None or before_migrations['baseline'] != after_migrations['baseline']
                or after_migrations['events'][:len(before_migrations['events'])] != before_migrations['events']):
            raise ValueError('source migration history changed or disappeared; append events instead')
    elif after_migrations is not None:
        baseline = {item['directory']: item for item in after_migrations['baseline']}
        expected = {directory: identity(item) for directory, item in previous['packages'].items()}
        if baseline != expected:
            raise ValueError('migration baseline does not match the existing source catalog')
    if after_migrations is None:
        for directory, package in previous.get('packages', {}).items():
            following = current['packages'].get(directory)
            if following is None or any(following.get(key) != package.get(key) for key in ('name', 'version', 'origin')):
                raise ValueError(f'current source identity disappeared or changed without a reviewed migration: {directory}')


def atomic_write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, prefix='.' + path.name + '.', delete=False) as stream:
        temporary = Path(stream.name)
        try:
            stream.write(value)
            stream.flush()
            os.fsync(stream.fileno())
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    temporary.chmod(0o644)
    temporary.replace(path)


def refresh(root):
    current, policy = snapshot(root)
    if safe(root, CATALOG).exists():
        previous = read_json(safe(root, CATALOG))
        if previous.get('schema') != 1:
            raise ValueError('unsupported current source catalog schema')
        preserve(previous, current)
    current['resolutions'] = resolutions(root, policy)
    atomic_write(safe(root, CATALOG), (json.dumps(current, ensure_ascii=False, sort_keys=True, indent=2) + '\n').encode())
    atomic_write(safe(root, DOC), document(current, policy).encode())
    if 'source_migrations' in current:
        atomic_write(safe(root, ARCHITECTURE_DOC), architecture_document(current, policy).encode())
    print(f'Recorded {len(current["packages"])} local Rust packages, {len(current["origins"])} immutable upstream archives, one workspace and one lockfile.')


def verify(root, metadata=False):
    expected = read_json(safe(root, CATALOG))
    current, policy = snapshot(root)
    if expected.get('schema') != 1:
        raise ValueError('unsupported current source catalog schema')
    expected_profiles = [(item['id'], item['target']) for item in policy['profiles']]
    actual_profiles = [(item['profile'], item['target']) for item in expected.get('resolutions', [])]
    if actual_profiles != expected_profiles:
        raise ValueError('recorded source closure profile coverage differs')
    preserve(expected, current)
    for key, value in current.items():
        if expected.get(key) != value:
            raise ValueError(f'Rust source receipt drift: {key}; verify never accepts new hashes')
    if safe(root, DOC).read_text() != document(expected, policy):
        raise ValueError('generated source documentation is out of sync')
    if 'source_migrations' in expected and safe(root, ARCHITECTURE_DOC).read_text() != architecture_document(expected, policy):
        raise ValueError('generated purpose architecture documentation is out of sync')
    if metadata and resolutions(root, policy) != expected.get('resolutions'):
        raise ValueError('offline source closure or selected features changed')
    print(f'Verified {len(current["packages"])} local Rust packages and immutable originals' +
          ('; Linux/WASM empty-Cargo-home metadata passed.' if metadata else '.'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('refresh', 'verify'))
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--metadata', action='store_true', help='verify Linux and WASM metadata with an empty temporary Cargo home')
    args = parser.parse_args()
    try:
        root = args.root.resolve(strict=True)
        if args.command == 'refresh':
            refresh(root)
        else:
            verify(root, args.metadata)
    except (ValueError, OSError, KeyError, TypeError, tarfile.TarError, json.JSONDecodeError) as error:
        print(f'Rust source check failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
