"""Independent, synchronous abstract specification and history checker."""
from copy import deepcopy
from contract import digest

class Oracle:
    def __init__(self, heads):
        self.heads = dict(heads)
        self.bundles = {}
        self.submits = {}
        self.outcomes = {}

    def call(self, method, args):
        if method == 'read_heads': return {b:self.heads[b] for b in args[0]}
        if method == 'get_status': return deepcopy(self.bundles[args[0]])
        if method == 'submit':
            rid, updates = args
            if rid not in self.submits:
                self.submits[rid] = dict(bundle=rid, revision=1, digest=digest(updates))
                self.bundles[rid] = dict(revision=1, updates=deepcopy(updates), digest=digest(updates),
                                        approvals=[], ci=False, published=False)
            return deepcopy(self.submits[rid])
        b = self.bundles[args[0]]
        revision = args[1]
        if method == 'revise':
            if revision != b['revision']: return 'stale'
            if b['published']: return 'published'
            b['revision'] += 1
            b['updates'] = deepcopy(args[2])
            b['digest'] = digest(args[2])
            b['approvals'] = []
            b['ci'] = False
            return 'updated'
        if method == 'approve':
            if revision != b['revision'] or b['published'] or args[2] not in ['alice','bob','carol']: return False
            if args[2] not in b['approvals']: b['approvals'].append(args[2])
            b['approvals'].sort()
            return True
        if method == 'record_ci':
            if revision != b['revision'] or b['published'] or args[2] != b['digest']: return False
            b['ci'] = bool(args[3])
            return True
        if method != 'publish': raise ValueError(method)
        rid = args[2]
        if rid in self.outcomes: return self.outcomes[rid]
        if revision != b['revision']: answer = 'stale'
        elif b['published']: answer = 'published'
        elif not b['ci'] or len(b['approvals']) < 2: answer = 'not_ready'
        else:
            answer = 'published'
            for branch, change in b['updates'].items():
                if self.heads[branch] != change['expected']: answer = 'conflict'
            if answer == 'published':
                for branch, change in b['updates'].items(): self.heads[branch] = change['new']
                b['published'] = True
        self.outcomes[rid] = answer
        return answer


def linearizable(initial, history):
    """Explore only legal real-time orders; compare all public return values."""
    def visit(model, remaining):
        if not remaining: return True
        for op in remaining:
            # A completed operation preceding this invocation must be ordered first.
            if any(other['end'] < op['start'] for other in remaining if other is not op): continue
            trial = deepcopy(model)
            if trial.call(op['method'], op['args']) != op['result']: continue
            if visit(trial, [x for x in remaining if x is not op]): return True
        return False
    return visit(deepcopy(initial), history)
