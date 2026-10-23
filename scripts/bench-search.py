#!/usr/bin/env python3
"""Run the search benchmark and record revision, raw discoveries, and censored results."""
import argparse
import datetime
import json
import os
from pathlib import Path
import statistics
import subprocess

ROOT = Path(__file__).resolve().parent.parent

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--out', required=True)
    p.add_argument('--seeds', type=int, default=20)
    p.add_argument('--seed-start', type=int, default=0)
    p.add_argument('--budget', type=int, default=10000)
    p.add_argument('--plain-feedback', action='store_true', help='also measure feedback without progress checkpoints (on by default)')
    args = p.parse_args()
    if args.seeds < 1 or args.budget < 1 or args.seed_start < 0:
        p.error('seeds and budget must be positive; seed-start must be nonnegative')
    subprocess.run(['cargo', 'build', '--release', '-p', 'snappea', '--example', 'search_bench', '--locked'], cwd=ROOT, check=True)
    env = dict(os.environ, SEARCH_SEEDS=str(args.seeds), SEARCH_SEED_START=str(args.seed_start), SEARCH_BUDGET=str(args.budget),
               SEARCH_PLAIN_FEEDBACK=str(int(args.plain_feedback)))
    data = json.loads(subprocess.check_output([str(ROOT/'target/release/examples/search_bench')], cwd=ROOT, env=env, text=True))
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
    data.update(commit=git('rev-parse', 'HEAD'), dirty=bool(git('status', '--porcelain', '--untracked-files=no')),
                timestamp=datetime.datetime.now(datetime.timezone.utc).isoformat())
    print('| Workload | Strategy | All bugs found | Median schedules (completed) | Penalized mean¹ |')
    print('|---|---|---:|---:|---:|')
    for workload in sorted({r['workload'] for r in data['rows']}):
        for strategy in sorted({r['strategy'] for r in data['rows']}):
            rows = [r for r in data['rows'] if r['workload'] == workload and r['strategy'] == strategy]
            complete = [r['schedules_to_all'] for r in rows if r['schedules_to_all'] is not None]
            penalized = [r['schedules_to_all'] if r['schedules_to_all'] is not None else 2*args.budget for r in rows]
            median = statistics.median(complete) if complete else '—'
            print(f'| {workload} | {strategy} | {len(complete)}/{len(rows)} | {median} | {statistics.mean(penalized):.1f} |')
    print('\n¹ Unfinished runs cost twice the schedule budget. Compare completion rates first; completed-only medians are biased by censoring.')
    Path(args.out).write_text(json.dumps(data, indent=2)+'\n')

if __name__ == '__main__':
    main()
