"""Short design-first task prompts for new cohorts; historical prompts stay frozen."""
VERSION = 3
COMMON = "Read TASK.md and contract.py. Write solution.py and DESIGN.md.\n\n"
CONTROL = "Design and implement the service in Python.\n"
P = CONTROL + "First, model it with P/Snappea and check what could go wrong.\n\nP/Snappea: /opt/snappea; docs: /opt/pdocs/; example: /opt/inventory/.\n"

def task_prompt(arm):
    if arm not in ('p', 'python'):
        raise ValueError(f'Unknown arm: {arm}')
    return COMMON + (P if arm == 'p' else CONTROL)
