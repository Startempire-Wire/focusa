#!/usr/bin/env python3
"""Restore tag-bound legacy trust by replaying the existing canonical tag signer.

Compatibility names are byte-identical aliases required by the immutable old
asset contract; signing remains in release.yml at its original tag identity.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def gh(*args):
    return subprocess.check_output(['gh', *args], text=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--tag', required=True)
    parser.add_argument('--sha', required=True)
    parser.add_argument('--run', required=True)
    parser.add_argument('--job', required=True)
    args = parser.parse_args()
    repo = os.environ['GITHUB_REPOSITORY']
    run = json.loads(gh('api', f'repos/{repo}/actions/runs/{args.run}'))
    job = json.loads(gh('api', f'repos/{repo}/actions/jobs/{args.job}'))
    if (run['event'] != 'push' or run['head_sha'] != args.sha
            or run['head_branch'] != args.tag or run['path'] != '.github/workflows/release.yml'
            or job['run_id'] != int(args.run) or job['name'] != 'Publish SHA256SUMS'):
        raise ValueError('exact original tag-bound release signer identity required')
    with tempfile.TemporaryDirectory(prefix='focusa-tag-trust-', dir=os.environ['RUNNER_TEMP']) as tmp:
        folder = Path(tmp)
        subprocess.run(['gh', 'release', 'download', args.tag, '--repo', repo,
                        '--pattern', 'release-manifest.json', '--dir', tmp], check=True)
        manifest = json.loads((folder / 'release-manifest.json').read_text())
        if manifest['tag'] != args.tag or manifest['commit'] != args.sha:
            raise ValueError('published candidate manifest identity mismatch')
        aliases = []
        for target, old_arch in [('aarch64-apple-darwin', 'aarch64'), ('x86_64-apple-darwin', 'x64')]:
            name = f'Focusa-{args.tag}-{target}.dmg'
            subprocess.run(['gh', 'release', 'download', args.tag, '--repo', repo,
                            '--pattern', name, '--dir', tmp], check=True)
            payload = folder / name
            if hashlib.sha256(payload.read_bytes()).hexdigest() != manifest['assets'][name]['sha256']:
                raise ValueError('immutable provider image digest mismatch: ' + name)
            alias = folder / f'Focusa_{args.tag.removeprefix("v")}_{old_arch}.dmg'
            # Same digest, two supported naming generations; never another build.
            alias.hardlink_to(payload)
            aliases.append(alias)
        subprocess.run(['gh', 'release', 'upload', args.tag, '--repo', repo,
                        *map(str, aliases), '--clobber'], check=True)
    subprocess.run(['gh', 'run', 'rerun', args.run, '--repo', repo, '--job', args.job], check=True)
    print('tag_signer_replay_requested tag=' + args.tag + ' source_sha=' + args.sha)


if __name__ == '__main__':
    main()
