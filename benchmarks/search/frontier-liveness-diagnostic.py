#!/usr/bin/env python3
"""Separate explicit-monitor diagnostic; original vendored files are never changed."""
import gzip,importlib.util,json,re,subprocess,tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('bench',root/'scripts/bench-corpus.py');b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
projects,files=b.inventory()
previous=json.load(gzip.open(root/'benchmarks/search/choice-corpus.json.gz','rt'))
missing={c['project'] for c in previous['cases'] if c.get('expected')=='DynamicError' and not c['known_issues']}
projects=[p for p in projects if p['id'] in missing]
manifest=dict(root=str(root),projects=projects,files=files,budget=20,steps=1000,seeds=1,strategies=['feedback'],seconds_per_run=1)
rows=[]
with tempfile.TemporaryDirectory() as tmp:
 path=Path(tmp)/'manifest.json'
 for project in projects:
  original='\n'.join(files[p] for p in project['pfiles'])
  clean=re.sub(r'/\*.*?\*/|//[^\n]*','',original,flags=re.S)
  machines=re.findall(r'\bmachine\s+(\w+)',clean);monitors=re.findall(r'\bspec\s+(\w+)',clean)
  for attach in [False,True]:
   item=dict(project)
   if attach:
    if re.search(r'\btest\s+',clean): continue
    generated=str(root/'scratch_diagnostic_monitors.p')
    files[generated]='test tcDiagnostic [main=Main]: assert '+', '.join(monitors)+' in { '+', '.join(machines)+' };\n'
    item['pfiles']=item['pfiles']+[generated]
   manifest['projects']=[item];path.write_text(json.dumps(manifest))
   run=subprocess.run([str(root/'target/release/examples/corpus_bench'),str(path),'0'],capture_output=True,text=True,check=True)
   events=[json.loads(l) for l in run.stdout.splitlines()]
   rows.append(dict(project=project['id'],attached_explicitly=attach,generated_test=files.get(generated) if attach else None,results=events))
   print(project['id'].split('/')[-1],attach,[(r.get('case_monitors'),r.get('first_issue_schedules')) for r in events],flush=True)
(root/'benchmarks/search/frontier-liveness-diagnostic.json').write_text(json.dumps(dict(description='Separate harness diagnostic; explicit monitor test changes the checked property and is not part of the unchanged-corpus comparison.',rows=rows),indent=2)+'\n')
