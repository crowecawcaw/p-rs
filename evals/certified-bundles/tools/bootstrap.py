#!/usr/bin/env python3
"""Stage the pinned checker and checkout docs without committing binaries or a source copy."""
import argparse
import hashlib
import io
from pathlib import Path
import shutil
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
NAME = 'snappea-v0.1.0-x86_64-unknown-linux-gnu'
SHA = '2169c917466275af47ef1db1aa9f49aeff5cb07b7a64d1d889eba7c9d8b70393'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, help='Use a locally built checker instead of v0.1.0')
    parser.add_argument('--source', type=Path, default=ROOT.parents[1], help='Source checkout providing docs/ and examples/model/')
    args = parser.parse_args()
    source = args.source.resolve()
    for directory in ['docs', 'examples/model']:
        if not (source / directory).is_dir():
            parser.error(f'{source} has no {directory}')
    binary = ROOT / 'vendor/bin' / NAME / 'snappea'
    binary.parent.mkdir(parents=True, exist_ok=True)
    if args.binary:
        shutil.copy2(args.binary.resolve(), binary)
    elif not binary.exists():
        url = f'https://github.com/crowecawcaw/snappea/releases/download/v0.1.0/{NAME}.tar.gz'
        with urllib.request.urlopen(url, timeout=60) as response:
            data = response.read()
        if hashlib.sha256(data).hexdigest() != SHA:
            raise SystemExit('Release checksum mismatch')
        with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
            with archive.extractfile(f'{NAME}/snappea') as stream:
                binary.write_bytes(stream.read())
        binary.chmod(0o755)
    link = ROOT / 'vendor/source/snappea-main'
    link.parent.mkdir(parents=True, exist_ok=True)
    if link.is_symlink():
        link.unlink()
    if link.exists():
        raise SystemExit(f'Refusing to replace existing directory: {link}')
    link.symlink_to(source, target_is_directory=True)
    print(f'Checker: {binary}\nSHA256: {hashlib.sha256(binary.read_bytes()).hexdigest()}\nDocs/examples: {source}')

if __name__ == '__main__':
    main()
