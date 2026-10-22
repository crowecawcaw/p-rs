"""Exercise the real API-loop plumbing with a scripted response, never a model trial."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('runner',ROOT/'tools/run_sonnet.py')
runner=importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)
turn=0

def scripted(model,messages,max_tokens,timeout):
    global turn
    turn+=1
    if turn==1:
        source=(ROOT/'private/reference.py').read_text()
        return {'id':'scripted-turn-1','model':'scripted-not-a-model','usage':{'input_tokens':0,'output_tokens':0},
            'content':[{'type':'tool_use','id':'tool1','name':'shell','input':{'command':
                "cat > solution.py <<'SCRIPTED_REFERENCE'\n"+source+"\nSCRIPTED_REFERENCE\npython -m unittest -v test_public"}}]}
    assert messages[-1]['role']=='user'
    result=messages[-1]['content'][0]
    assert result['type']=='tool_result' and result['tool_use_id']=='tool1'
    assert json.loads(result['content'])['exit_code']==0
    return {'id':'scripted-turn-2','model':'scripted-not-a-model','usage':{'input_tokens':0,'output_tokens':0},
        'content':[{'type':'text','text':'Scripted protocol smoke complete, not an agent result.'}]}

run_id='protocol-smoke'
if (ROOT/'runs'/run_id).exists():
    raise SystemExit('Protocol smoke already exists; remove it explicitly to repeat.')
args=argparse.Namespace(run_id=run_id,arm='python',model='scripted-not-a-model',seconds=90,
    generated_tokens=1000,tool_calls=4,seeds=12,smoke=False,protocol_test=True)
with patch.dict(os.environ,{'ANTHROPIC_API_KEY':'scripted-placeholder-no-network'}),patch.object(runner,'request',scripted):
    runner.main(args)
meta=json.loads((ROOT/'runs'/run_id/'metadata.json').read_text())
assert meta['smoke'] is True and meta['tool_calls']==1 and meta['status']=='agent_finished'
print('API message/tool-result protocol, freezing, and isolated hidden grading passed (scripted).')
