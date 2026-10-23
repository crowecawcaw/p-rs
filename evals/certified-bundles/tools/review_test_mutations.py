#!/usr/bin/env python3
"""Post-freeze diagnostic: exercise submitted tests against reference mutations."""
import argparse
import ast
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
RUNS = ROOT / 'runs'
OUT = ROOT / 'results' / 'v5-test-mutations.json'
REFERENCE = ROOT / 'private' / 'reference.py'


def known_mutations():
    tree = ast.parse((ROOT / 'tools' / 'calibrate.py').read_text())
    node = next(n.value for n in tree.body if isinstance(n, ast.Assign)
                and any(isinstance(t, ast.Name) and t.id == 'mutations' for t in n.targets))
    return ast.literal_eval(node)


def test_paths(work):
    return sorted(str(p.relative_to(work)) for p in work.rglob('test*.py')
                  if p.name != 'test_public.py' and '__pycache__' not in p.parts
                  and '.pytest_cache' not in p.parts)


def docker_test(run, work, tests, solution, timeout, binary):
    """Run the explicit agent-test paths in the frozen run's networkless image."""
    meta = json.loads((run / 'metadata.json').read_text())
    arm = meta['arm']
    args = ['docker', 'run', '--rm', '--network', 'none', '--cap-drop', 'ALL',
            '--name', f'snappea-test-mutation-{uuid.uuid4().hex[:12]}',
            '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}',
            '--memory', '768m', '--cpus', '1', '--pids-limit', '128', '--read-only',
            '--tmpfs', '/tmp:rw,size=128m', '--mount', f'type=bind,src={work},dst=/work',
            '-w', '/work']
    if arm == 'p':
        if binary is None:
            from run_sonnet import tool_paths
            binary = tool_paths()[0]
        binary = Path(binary).resolve()
        if not binary.is_file():
            raise RuntimeError(f'P checker binary not found: {binary}; pass --binary')
        args += ['--env', 'PATH=/opt:/usr/local/bin:/usr/local/sbin:/usr/sbin:/usr/bin:/sbin:/bin',
                 '--mount', f'type=bind,src={binary},dst=/opt/snappea,readonly']
    command = ['pytest', '-q', '-p', 'no:cacheprovider'] + tests
    try:
        proc = subprocess.run(args + [meta['image']] + command, capture_output=True,
                              text=True, timeout=timeout)
        return {'exit_code': proc.returncode, 'stdout': proc.stdout[-5000:],
                'stderr': proc.stderr[-3000:], 'solution': solution}
    except subprocess.TimeoutExpired as exc:
        name = args[args.index('--name') + 1]
        subprocess.run(['docker', 'kill', name], capture_output=True, timeout=5)
        return {'exit_code': 124, 'stdout': (exc.stdout or b'').decode(errors='replace')[-3000:],
                'stderr': f'suite timeout after {timeout} seconds', 'solution': solution}


def prepare(run, solution_text, temp):
    submission = run / 'submission'
    if not (run / 'submission-manifest.json').is_file() or not submission.is_dir():
        return None
    work = Path(temp) / 'work'
    work.mkdir()
    for name in ('contract.py', 'runtime.py'):
        shutil.copy2(ROOT / 'public' / name, work / name)
    for p in submission.rglob('*'):
        if not p.is_file() or '__pycache__' in p.parts or '.pytest_cache' in p.parts:
            continue
        rel = p.relative_to(submission)
        if rel.as_posix() == 'solution.py':
            continue
        target = work / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(p, target)
    (work / 'solution.py').write_text(solution_text)
    return work


def classify(result):
    text = (result.get('stdout', '') + result.get('stderr', '')).lower()
    if result['exit_code'] == 0:
        return 'pass'
    if any(s in text for s in ('missing emit', 'no events', 'event stream is empty',
                                'expected at least one event', 'no events were emitted',
                                'expected an event', 'trace is empty')):
        return 'instrumentation_mismatch'
    if result['exit_code'] == 124:
        return 'timeout'
    if 'assert' in text or 'failed' in text or 'specviolation' in text:
        return 'test_reject'
    return 'execution_error'


def one_case(run, tests, solution_text, label, temp, timeout, binary):
    work = prepare(run, solution_text, temp)
    if work is None:
        return {'status': 'not_frozen'}
    paths = test_paths(work)
    if not paths:
        return {'status': 'no_tests'}
    result = docker_test(run, work, paths, label, timeout, binary)
    return {'status': 'pass' if result['exit_code'] == 0 else 'fail',
            'classification': classify(result), 'result': result}


def process(run_id, mutations, timeout, binary):
    run = RUNS / run_id
    meta = json.loads((run / 'metadata.json').read_text())
    if meta.get('status') == 'running' or not (run / 'submission-manifest.json').is_file():
        return {'run': run_id, 'status': 'not_frozen'}
    tests = test_paths(run / 'submission')
    if not tests:
        return {'run': run_id, 'status': 'no_tests'}
    with tempfile.TemporaryDirectory(prefix=f'{run_id}-tests-') as td:
        own = one_case(run, tests, (run / 'submission' / 'solution.py').read_text(), 'own', td, timeout, binary)
    with tempfile.TemporaryDirectory(prefix=f'{run_id}-ref-') as td:
        baseline = one_case(run, tests, REFERENCE.read_text(), 'reference', td, timeout, binary)
    item = {'run': run_id, 'arm': meta['arm'], 'image': meta['image'], 'tests': tests,
            'own_solution': own, 'reference': baseline, 'mutants': {}}
    if baseline.get('status') != 'pass':
        item['status'] = 'baseline_incompatible'
        return item
    source = REFERENCE.read_text()
    for name, (old, new) in mutations.items():
        if old not in source:
            item['mutants'][name] = {'classification': 'mutation_source_mismatch'}
            continue
        mutant = source.replace(old, new)
        with tempfile.TemporaryDirectory(prefix=f'{run_id}-{name}-') as td:
            result = one_case(run, tests, mutant, name, td, timeout, binary)
        case_class = result.get('classification')
        cls = {'pass': 'survived', 'test_reject': 'killed',
               'instrumentation_mismatch': 'instrumentation_mismatch',
               'timeout': 'timed_out', 'execution_error': 'execution_error'}.get(
                   case_class, result.get('status', 'invalid'))
        item['mutants'][name] = {'classification': cls, 'result': result}
    item['status'] = 'complete'
    return item


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('run_ids', nargs='*')
    parser.add_argument('--workers', type=int, default=2)
    parser.add_argument('--output', type=Path, default=OUT)
    parser.add_argument('--timeout', type=int, default=30,
                        help='maximum seconds for each isolated test suite run (default: 30)')
    parser.add_argument('--binary', default=os.environ.get('SNAPPEA_EVAL_BINARY'),
                        help='P checker executable; otherwise use the eval runner binary path')
    args = parser.parse_args()
    if args.workers < 1 or args.workers > 2:
        parser.error('--workers must be 1 or 2')
    if args.timeout < 1:
        parser.error('--timeout must be positive')
    run_ids = args.run_ids or sorted(p.name for p in RUNS.glob('v5-*'))
    mutations = known_mutations()
    output = args.output
    previous = {}
    if output.is_file():
        try:
            previous = {row['run']: row for row in json.loads(output.read_text()).get('runs', [])}
        except (ValueError, KeyError, TypeError):
            previous = {}
    results = []
    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        futures = {pool.submit(process, rid, mutations, args.timeout, args.binary): rid for rid in run_ids}
        for future in as_completed(futures):
            item = future.result()
            previous[item['run']] = item
            print(json.dumps({'run': item['run'], 'status': item['status']}), flush=True)
    results = sorted(previous.values(), key=lambda x: x['run'])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({'diagnostic': 'agent tests against reference mutations',
                               'generated_at': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                               'execution': {'suite_timeout_seconds': args.timeout,
                                             'max_workers': args.workers,
                                             'p_checker': args.binary or 'eval runner default'},
                               'runs': results}, indent=2) + '\n')
    summary = []
    for item in results:
        if item.get('status') == 'complete':
            counts = {}
            for mutant in item['mutants'].values():
                key = mutant['classification']
                counts[key] = counts.get(key, 0) + 1
            summary.append({'run': item['run'], 'own': item['own_solution']['status'],
                            'baseline': item['reference']['status'], 'mutants': counts})
        else:
            summary.append({'run': item['run'], 'status': item['status']})
    print(json.dumps({'output': str(output), 'summary': summary}, indent=2))


if __name__ == '__main__':
    main()
