#!/usr/bin/env python3
"""NuGet restore and assembly inventory for CSharpX (not a CLR invocation bridge)."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import xml.etree.ElementTree as ET


def project_refs(project):
    root = ET.parse(project).getroot()
    def tag(e): return e.tag.rsplit('}', 1)[-1]
    packages, dlls = [], []
    for e in root.iter():
        if tag(e) == 'PackageReference':
            name = e.get('Include') or e.get('Update')
            version = e.get('Version') or next((c.text for c in e if tag(c) == 'Version'), None)
            if not name or not version: raise ValueError('PackageReference requires Include and Version')
            packages.append((name, version))
        elif tag(e) == 'Reference':
            hint = next((c.text for c in e if tag(c) == 'HintPath'), None)
            if hint:
                dlls.append(str((Path(project).parent / hint).resolve()))
    return packages, dlls


def make_restore_project(packages, framework, out):
    ET.register_namespace('', 'http://schemas.microsoft.com/developer/msbuild/2003')
    root = ET.Element('Project', {'Sdk': 'Microsoft.NET.Sdk'})
    props = ET.SubElement(root, 'PropertyGroup')
    ET.SubElement(props, 'TargetFramework').text = framework
    ET.SubElement(props, 'EnableDefaultCompileItems').text = 'false'
    ET.SubElement(props, 'OutputType').text = 'Library'
    items = ET.SubElement(root, 'ItemGroup')
    for name, version in packages:
        ET.SubElement(items, 'PackageReference', {'Include': name, 'Version': version})
    out.parent.mkdir(parents=True, exist_ok=True)
    ET.ElementTree(root).write(out, encoding='utf-8', xml_declaration=True)


def inventory(assets, dlls, project, framework):
    data = json.loads(Path(assets).read_text(encoding='utf-8'))
    targets = data.get('targets', {})
    target_key = next((k for k in targets if k == framework or k.startswith(framework + '/')), None)
    if target_key is None: raise ValueError('NuGet assets missing target ' + framework)
    folders = data.get('packageFolders', {})
    libraries = data.get('libraries', {})
    assemblies = []
    for lib, entry in targets[target_key].items():
        info = libraries.get(lib, {})
        package_path = info.get('path', lib.lower())
        for kind in ('compile', 'runtime'):
            for relative in entry.get(kind, {}):
                if not relative.lower().endswith('.dll') or relative.startswith('_'):
                    continue
                paths = [str(Path(folder) / package_path / relative) for folder in folders]
                assemblies.append({'package': lib, 'kind': kind, 'asset': relative,
                                   'paths': paths})
    for path in dlls:
        assemblies.append({'package': None, 'kind': 'direct', 'asset': Path(path).name, 'paths': [path]})
    return {'schemaVersion': 1, 'project': str(Path(project).resolve()),
            'framework': framework, 'assemblies': assemblies,
            'interopStatus': 'metadata-inventory-only',
            'warning': 'Managed DLLs cannot be executed by the native CSharpX Rust backend without a CLR host or compatible managed runtime.'}


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('project', type=Path)
    p.add_argument('--framework', default='net8.0')
    p.add_argument('--output', type=Path)
    p.add_argument('--offline-assets', type=Path, help='Use existing project.assets.json; do not invoke dotnet')
    args = p.parse_args(argv)
    packages, dlls = project_refs(args.project)
    output = args.output or args.project.parent / 'obj' / 'csharpx-dotnet-references.json'
    if args.offline_assets:
        assets = args.offline_assets
    elif packages:
        restore_dir = args.project.parent / 'obj' / 'csharpx-nuget'
        restore_project = restore_dir / 'CSharpX.NuGetRestore.csproj'
        make_restore_project(packages, args.framework, restore_project)
        subprocess.run(['dotnet', 'restore', str(restore_project)], check=True)
        assets = restore_dir / 'obj' / 'project.assets.json'
    else:
        assets = None
    result = inventory(assets, dlls, args.project, args.framework) if assets else {
        'schemaVersion': 1, 'project': str(args.project.resolve()), 'framework': args.framework,
        'assemblies': [{'package': None, 'kind': 'direct', 'asset': Path(x).name, 'paths': [x]} for x in dlls],
        'interopStatus': 'metadata-inventory-only',
        'warning': 'No CLR execution bridge is implemented.'}
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(f'Wrote {output} ({len(result["assemblies"])} assembly entries)')
    return 0

if __name__ == '__main__':
    try: sys.exit(main())
    except (ValueError, OSError, subprocess.CalledProcessError) as exc:
        print(f'CSX-DOTNET: {exc}', file=sys.stderr)
        sys.exit(1)
