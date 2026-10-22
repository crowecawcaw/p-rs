import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))

pytest_plugins = ["pytester"]

# Use a freshly built binary when testing from a checkout.
if "SNAPPEA_BIN" not in os.environ:
    for profile in ("release", "debug"):
        exe = os.path.join(ROOT, "target", profile, "snappea" + (".exe" if os.name == "nt" else ""))
        if os.path.isfile(exe):
            os.environ["SNAPPEA_BIN"] = exe
            break

sys.path.insert(0, os.path.join(HERE, "example"))
SPEC = os.path.join(HERE, "example", "specs", "lock.p")
