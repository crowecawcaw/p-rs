"""Short design-first task prompts for new cohorts; historical prompts stay frozen."""
import os
VERSION = 7
COMMON = "Read TASK.md and contract.py.\n\n"
CONTROL = "Write a design. Then implement it in Python.\nAdd Python tests.\n"
VARIANTS = {
    'design_process': "Make a P/Snappea model as part of the design process. Once it passes, begin the Python implementation. Use pysnappea in Python tests.",
    'before_file': "Design the service in P/Snappea and get the model passing before writing solution.py. Then implement it in Python and use pysnappea in the tests.",
    'passing_first': "First write and check a P/Snappea model of the design. Only start the Python implementation after the model passes. Use pysnappea in Python tests.",
    'implementation_events': "First write and check a P/Snappea model of the design. Only start the Python implementation after the model passes. Use pysnappea to check events from the Python implementation during tests.",
}
DEFAULT_VARIANT = 'implementation_events'
P = VARIANTS[DEFAULT_VARIANT] + "\n\nP/Snappea: /opt/snappea; docs: /opt/pdocs/; example: /opt/inventory/.\n"

def prompt_version():
    return VERSION

def prompt_variant(arm):
    return os.environ.get('SNAPPEA_EVAL_PROMPT_VARIANT') or (DEFAULT_VARIANT if arm == 'p' else None)

def task_prompt(arm):
    if arm not in ('p', 'python'):
        raise ValueError(f'Unknown arm: {arm}')
    variant = os.environ.get('SNAPPEA_EVAL_PROMPT_VARIANT')
    if variant:
        if arm != 'p' or variant not in VARIANTS:
            raise ValueError(f'Unknown P prompt variant: {variant}')
        body = VARIANTS[variant] + "\n\nP/Snappea: /opt/snappea; docs: /opt/pdocs/; example: /opt/inventory/.\n"
    else:
        body = P if arm == 'p' else CONTROL
    return COMMON + body + "\nDeliver solution.py and DESIGN.md.\n"
