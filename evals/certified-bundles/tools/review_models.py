#!/usr/bin/env python3
"""Reproduce post-freeze P property/reachability probes; never modify submissions."""
import json
from pathlib import Path
import shutil
import subprocess
ROOT=Path(__file__).resolve().parents[1]
BIN=ROOT/'vendor/bin/snappea-v0.1.0-x86_64-unknown-linux-gnu/snappea'
settings=[
('luna-p-01',['ChangeBundle.pproj'],'ChangeBundle.p','else if (head != "s0")','else if (false)','parent_conflict'),
('luna-p-02',['bundle_model.p'],'bundle_model.p','if (head == r.expected)','if (true)','parent_conflict'),
('luna-p-03',['Bundle.p'],'Bundle.p','else if (head != 0)','else if (false)','parent_conflict'),
('luna-p-04',['model/AtomicJournal.p'],'model/AtomicJournal.p','right = req.value;','right = req.value + 1;','torn_publication'),
('luna-p-04',['model/AtomicJournal.p'],'model/AtomicJournal.p','won = req.expected == version;','won = true;','stale_cas_acceptance'),
('luna-p-05',['bundle_model.p'],'bundle_model.p','assert staged[r.request],','assert false,','commit_reachability'),
('luna-p-06',['BundleModel.p','TwoPublishers.p'],'BundleModel.p','right = "r1";','right = "BROKEN";','torn_publication'),
]
results=[]
for rid,entry,source,old,new,kind in settings:
    dest=ROOT/'results/model-review'/f'{rid}-{kind}'
    dest.mkdir(parents=True,exist_ok=True)
    submission=ROOT/'runs'/rid/'submission'
    for f in submission.rglob('*'):
        if f.is_file() and f.suffix in ['.p','.pproj']:
            rel=f.relative_to(submission)
            if 'PCheckerOutput' in rel.parts: continue
            out=dest/rel;out.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,out)
    args=[str(BIN),'check',*entry,'--exhaustive']
    baseline=subprocess.run(args,cwd=dest,capture_output=True,text=True,timeout=30)
    (dest/'baseline.log').write_text(baseline.stdout+baseline.stderr)
    p=dest/source;s=p.read_text();assert old in s,(rid,kind)
    p.write_text(s.replace(old,new))
    probe=subprocess.run(args,cwd=dest,capture_output=True,text=True,timeout=30)
    (dest/'probe.log').write_text(probe.stdout+probe.stderr)
    p.write_text(s)
    item={'run':rid,'probe':kind,'baseline_exit':baseline.returncode,'probe_exit':probe.returncode,
          'baseline_output':baseline.stdout+baseline.stderr,'probe_output':probe.stdout+probe.stderr,
          'property_triggered':baseline.returncode==0 and probe.returncode==1}
    results.append(item);print(rid,kind,'baseline',baseline.returncode,'probe',probe.returncode,flush=True)
(ROOT/'results/model-probes.json').write_text(json.dumps(results,indent=2))
