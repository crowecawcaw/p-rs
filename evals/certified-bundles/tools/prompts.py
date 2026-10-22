"""Short design-first task prompts for new cohorts; historical prompts stay frozen."""
VERSION = 5
COMMON = "Read TASK.md and contract.py.\n\n"
CONTROL = "Write a design. Then implement it in Python.\nAdd Python tests.\n"
P = "Write a design and a P/Snappea model. Check and refine the model until you're happy with it, then implement it in Python.\nUse pysnappea in Python tests.\n\nP/Snappea: /opt/snappea; docs: /opt/pdocs/; example: /opt/inventory/.\n"

def task_prompt(arm):
    if arm not in ('p', 'python'):
        raise ValueError(f'Unknown arm: {arm}')
    return COMMON + (P if arm == 'p' else CONTROL) + "\nDeliver solution.py and DESIGN.md.\n"
