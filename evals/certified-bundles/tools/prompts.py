"""Short design-first task prompts for new cohorts; historical prompts stay frozen."""
VERSION = 2
COMMON = "Read TASK.md and contract.py. Write solution.py and DESIGN.md.\n\n"
P = "Design the certified change bundle service and model it with P/Snappea. Then implement it in Python.\n\nP/Snappea: /opt/snappea; docs: /opt/pdocs/; example: /opt/inventory/.\n"
CONTROL = "Design the certified change bundle service. Then implement it in Python. Do not use P/Snappea.\n"

def task_prompt(arm):
    if arm not in ('p', 'python'):
        raise ValueError(f'Unknown arm: {arm}')
    return COMMON + (P if arm == 'p' else CONTROL)
