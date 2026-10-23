#!/usr/bin/env python3
"""Quick schedules-to-all-observed-issues benchmark over every runnable repository model case."""
import argparse
import concurrent.futures
import datetime
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
STRATEGIES = ['random', 'feedback', 'dfs', 'auto']


def inventory(scope='all'):
    projects = []
    roots = [ROOT/'examples', ROOT/'tests/upstream/Tutorial', ROOT/'design/time']
    for root in roots:
        for path in sorted(root.rglob('*.pproj')):
            projects.append({'id': path.relative_to(ROOT).as_posix(), 'pproj': str(path)})
    # Standalone time examples and existing rare-prefix regression model.
    for path in sorted((ROOT/'design/time/examples').glob('*.p')) + sorted((ROOT/'crates/snappea/tests/models').glob('*.p')):
        projects.append({'id': path.relative_to(ROOT).as_posix(), 'pfiles': [str(path)]})
    if scope == 'all':
        base = ROOT/'tests/upstream/RegressionTests'
        roots.append(base)
        for suite in sorted(base.iterdir()):
            if not suite.is_dir():
                continue
            for category in ['Correct', 'DynamicError', 'StaticError']:
                folder = suite/category
                if not folder.is_dir():
                    continue
                for path in sorted(folder.iterdir()):
                    files = sorted(str(p) for p in path.rglob('*.p')) if path.is_dir() else []
                    if files:
                        item = {'id': path.relative_to(ROOT).as_posix(), 'pfiles': files, 'expected': category}
                        if category == 'StaticError':
                            item['skip'] = 'static-error test: compilation is expected to fail; no schedules'
                        projects.append(item)
    files = {}
    for root in roots + [ROOT/'crates/snappea/tests/models']:
        for path in root.rglob('*'):
            if path.is_file() and path.suffix in ['.p', '.pproj', '.cs', '.java'] and not {'PGenerated', 'PCheckerOutput'}.intersection(path.parts):
                files[str(path)] = path.read_text(errors='replace') if path.suffix in ['.p', '.pproj'] else ''
    return projects, files


def analyze(projects):
    """Use one common union catalog per case, preserving unknown and incomplete results."""
    cases = []
    for project in projects:
        runs = project.get('runs', [])
        names = sorted(set(project.get('cases', [])) | {r['case'] for r in runs})
        for name in names:
            rows = [dict(r) for r in runs if r['case'] == name]
            issues = sorted({issue for r in rows for issue in r.get('first_issue_schedules', {})})
            potential = sorted({issue for r in rows for issue in r.get('potential_liveness_schedules', {})})
            certified = any(r.get('exhaustive_complete') and not r.get('unsupported_issues') and
                            set(issues).issubset(r.get('first_issue_schedules', {})) for r in rows)
            for row in rows:
                first = row.get('first_issue_schedules', {})
                row['schedules_to_all_known'] = max((first[i] for i in issues), default=None) if issues and set(issues).issubset(first) else None
                row['known_issues_found'] = len(set(issues).intersection(first))
            cases.append({'project': project['id'], 'case': name, 'known_issues': issues,
                          'expected': project.get('expected'), 'catalog_complete': certified, 'potential_liveness_sites': potential, 'runs': rows})
    return cases


def run_project(index, project, manifest_path, opts):
    result = dict(project)
    if project.get('skip'):
        result.update(status='excluded', detail=project['skip'], runs=[])
        return result
    try:
        process = subprocess.run([str(ROOT/'target/release/examples/corpus_bench'), str(manifest_path), str(index)],
                                 cwd=ROOT, capture_output=True, text=True, timeout=opts.project_timeout)
        output, error, code = process.stdout, process.stderr, process.returncode
    except subprocess.TimeoutExpired as exc:
        output = exc.stdout or ''
        if isinstance(output, bytes):
            output = output.decode(errors='replace')
        error, code = f'project exceeded {opts.project_timeout}s', -1
    result['runs'] = []
    for line in output.splitlines():
        try:
            item = json.loads(line)
        except json.JSONDecodeError:
            continue  # Only an incomplete final line can be lost on process termination.
        if item.pop('kind') == 'project':
            result.update(item)
        else:
            # Missing foreign adapters are execution limitations, not discovered model bugs.
            first = item.get('first_issue_schedules', {})
            unsupported = {k: v for k, v in first.items() if 'has no implementation registered with the checker' in k}
            item['unsupported_issues'] = unsupported
            item['potential_liveness_schedules'] = {k: v for k, v in first.items() if 'detected potential liveness bug' in k}
            item['first_issue_schedules'] = {k: v for k, v in first.items() if k not in unsupported and k not in item['potential_liveness_schedules']}
            result['runs'].append(item)
    if code:
        result.update(status='process_error' if code != -1 else 'project_timeout', detail=error[-4000:])
    result.setdefault('status', 'no_output')
    existing = {(r['case'], r['strategy'], r['seed']) for r in result['runs']}
    for case in result.get('cases', []):
        for strategy in STRATEGIES:
            for seed in range(opts.seeds):
                if (case, strategy, seed) not in existing:
                    result['runs'].append({'case': case, 'strategy': strategy, 'seed': seed, 'status': 'not_run',
                                           'schedules_tested': 0, 'first_issue_schedules': {}})
    return result


def report(data):
    cases = data['cases']
    positive = [c for c in cases if c['known_issues']]
    lines = [f"Measured {len(cases)} cases across {len(data['projects'])} project inputs; {len(positive)} cases have observed issues.",
             f"Revision: `{data['commit']}`; working tree dirty: {data['dirty']}.",
             f"Budget: {data['budget']} schedules, {data['steps']} steps, {data['seeds']} seed(s), {data['seconds_per_run']}s safety cap per run.",
             f"Completed exhaustive catalogs: {sum(c['catalog_complete'] for c in cases)}; expected DynamicError cases with no common issue observed: {sum(c.get('expected') == 'DynamicError' and not c['known_issues'] for c in cases)}.",
             '', '**All issues means the union observed across every strategy and seed in this run.**',
             'Only a completed unbounded exhaustive search certifies the catalog. A blank catalog otherwise means unknown, not bug-free.',
             'Counts are first-discovery schedule indexes, computed after the shared catalog is formed; successful schedules count too.',
             'Temperature-based potential liveness warnings are recorded separately and excluded from the common issue catalog: unfair DFS does not perform that check.',
             '', '| Strategy | Runs finding all observed issues (issue-bearing cases) |', '|---|---:|']
    for strategy in STRATEGIES:
        rows = [r for c in positive for r in c['runs'] if r['strategy'] == strategy]
        lines.append(f"| {strategy} | {sum(r['schedules_to_all_known'] is not None for r in rows)}/{len(rows)} |")
    lines += ['', '| Case | Known issues | Potential liveness | Catalog complete | ' + ' | '.join(STRATEGIES) + ' |',
              '|---|---:|---:|---|' + '---:|'*len(STRATEGIES)]
    for case in cases:
        cells = []
        for strategy in STRATEGIES:
            rows = [r for r in case['runs'] if r['strategy'] == strategy]
            complete = [r['schedules_to_all_known'] for r in rows if r['schedules_to_all_known'] is not None]
            tested = sum(r.get('schedules_tested', 0) for r in rows)
            if not case['known_issues']:
                cell = f'— ({tested} tested)'
            elif complete:
                cell = f'{statistics.median(complete):g} ({len(complete)}/{len(rows)})'
            else:
                found = max((r.get('known_issues_found', 0) for r in rows), default=0)
                cell = f'miss {found}/{len(case["known_issues"])} ({tested} tested)'
            cells.append(cell)
        name = f"{case['project']}::{case['case']}".replace('|', '\\|')
        lines.append(f"| {name} | {len(case['known_issues'])} | {len(case.get('potential_liveness_sites', []))} | {'yes' if case['catalog_complete'] else 'no'} | " + ' | '.join(cells) + ' |')
    excluded = [p for p in data['projects'] if p['status'] != 'ready']
    lines += ['', f'{len(excluded)} project inputs excluded, unsupported, or interrupted:', '']
    for project in excluded:
        detail = project.get('detail', '').replace('\n', ' ')[:350]
        lines.append(f"- `{project['id']}`: {project['status']} — {detail}")
    lines += ['', 'Cells show median schedules among completed seeds plus completed/attempted runs. A miss retains the number of issues found and schedules actually tested.',
              'Time caps, engine errors, unrun rows, step-bound hits and missing foreign implementations are retained in the JSON. This quick pass does not establish that unseen issues do not exist.']
    return '\n'.join(lines)+'\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True)
    parser.add_argument('--report')
    parser.add_argument('--scope', choices=['all', 'examples'], default='all')
    parser.add_argument('--filter', default='')
    parser.add_argument('--budget', type=int, default=200)
    parser.add_argument('--steps', type=int, default=1000)
    parser.add_argument('--seeds', type=int, default=1)
    parser.add_argument('--seconds-per-run', type=float, default=0.1)
    parser.add_argument('--project-timeout', type=float, default=60)
    parser.add_argument('--jobs', type=int, default=2)
    opts = parser.parse_args()
    if min(opts.budget, opts.steps, opts.seeds, opts.seconds_per_run, opts.project_timeout, opts.jobs) <= 0:
        parser.error('budgets, bounds, seeds, timeouts and jobs must be positive')
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
    commit, dirty = git('rev-parse', 'HEAD'), bool(git('status', '--porcelain', '--untracked-files=no'))
    subprocess.run(['cargo', 'build', '--release', '--locked', '-p', 'snappea', '--example', 'corpus_bench'], cwd=ROOT, check=True)
    projects, files = inventory(opts.scope)
    projects = [p for p in projects if opts.filter in p['id']]
    settings = {k: getattr(opts, k) for k in ['budget', 'steps', 'seeds', 'seconds_per_run', 'scope']}
    manifest = dict(settings, root=str(ROOT), strategies=STRATEGIES, projects=projects, files=files)
    results = [None]*len(projects)
    with tempfile.TemporaryDirectory(prefix='snappea-corpus-') as temp:
        path = Path(temp)/'manifest.json'
        path.write_text(json.dumps(manifest))
        with concurrent.futures.ThreadPoolExecutor(max_workers=opts.jobs) as pool:
            futures = {pool.submit(run_project, i, p, path, opts): i for i, p in enumerate(projects)}
            for done, future in enumerate(concurrent.futures.as_completed(futures), 1):
                index = futures[future]
                results[index] = future.result()
                if done % 25 == 0 or done == len(projects):
                    print(f'{done}/{len(projects)} projects measured', flush=True)
    cases = analyze(results)
    catalog = [(c['project'], c['case'], c['known_issues']) for c in cases]
    data = dict(settings, suite_version=1, commit=commit, dirty=dirty, projects=results, cases=cases,
                catalog_sha256=hashlib.sha256(json.dumps(catalog, sort_keys=True).encode()).hexdigest(),
                timestamp=datetime.datetime.now(datetime.timezone.utc).isoformat())
    Path(opts.out).write_text(json.dumps(data, indent=2)+'\n')
    markdown = report(data)
    destination = Path(opts.report) if opts.report else Path(opts.out).with_suffix('.md')
    destination.write_text(markdown)
    print('\n'.join(markdown.splitlines()[:18]))
    print(f'Per-case report: {destination}; raw results: {opts.out}')

if __name__ == '__main__':
    main()
