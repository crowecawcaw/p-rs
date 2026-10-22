#!/usr/bin/env python3
"""Post-freeze validation only; never feed reviewer checks back to solver agents."""
from transcripts import read_transcript
import json
import argparse
from pathlib import Path
import re
import shutil
import subprocess
ROOT=Path(__file__).resolve().parents[1]
BIN=ROOT/'vendor/bin/snappea-v0.1.0-x86_64-unknown-linux-gnu/snappea'
parser=argparse.ArgumentParser()
parser.add_argument('--prefix',default='v2-p-')
parser.add_argument('--output',default='results/v2-model-review.json')
options=parser.parse_args()
results=[]
for run in sorted((ROOT/'runs').glob(options.prefix+'*')):
    submission=run/'submission'
    files=[p for p in submission.rglob('*.p') if 'PCheckerOutput' not in p.relative_to(submission).parts]
    rows=[json.loads(l) for l in read_transcript(run).splitlines()]
    commands=[line for row in rows for line in row['command'].splitlines() if re.match(r'\s*(?:/opt/)?snappea\s+check\b',line) and '--help' not in line]
    result={'run':run.name,'p_source_files':[str(p.relative_to(submission)) for p in files],
        'solver_checker_invoked':bool(commands),'solver_check_commands':commands,'reviewer_only':True}
    if files:
        dest=ROOT/'results/v2-model-review'/run.name;dest.mkdir(parents=True,exist_ok=True)
        for f in files:
            target=dest/f.relative_to(submission);target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,target)
        args=[str(BIN),'check',*[str(p.relative_to(submission)) for p in files],'--exhaustive']
        check=subprocess.run(args,cwd=dest,capture_output=True,text=True,timeout=30)
        output=check.stdout+check.stderr;(dest/'reviewer-check.log').write_text(output)
        result.update(reviewer_exit=check.returncode,reviewer_output=output)
        if run.name=='v2-p-06' and check.returncode==0:
            p=dest/'CertifiedBundles.p';source=p.read_text()
            mutant=source.replace('memoized = true','memoized = false')
            assert mutant!=source
            p.write_text(mutant)
            probe=subprocess.run(args,cwd=dest,capture_output=True,text=True,timeout=30)
            (dest/'memo-probe.log').write_text(probe.stdout+probe.stderr)
            p.write_text(source)
            result.update(memo_mutation_exit=probe.returncode,memo_mutation_output=probe.stdout+probe.stderr)
    results.append(result)
    print(run.name,'source files',len(files),'solver checked',bool(commands),'reviewer exit',result.get('reviewer_exit'))
(ROOT/options.output).write_text(json.dumps(results,indent=2))
