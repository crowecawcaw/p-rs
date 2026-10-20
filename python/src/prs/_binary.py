"""Finds the `prs` executable: `PRS_BIN`, else the one installed with this package, else
one on PATH."""

from __future__ import annotations

import os
import shutil
import sys
import sysconfig
from functools import lru_cache

from ._errors import SetupError

BINARY_ENV = "PRS_BIN"


@lru_cache(maxsize=None)
def find_prs() -> str:
    """Path of the `prs` binary: `PRS_BIN`, the copy installed with this package, or
    `prs` on PATH."""
    override = os.environ.get(BINARY_ENV)
    if override:
        if not os.path.isfile(override):
            raise SetupError(f"{BINARY_ENV}={override} is not a file", "binary_missing")
        return os.path.abspath(override)
    exe = "prs" + (sysconfig.get_config_var("EXE") or "")
    # wheels install the binary next to the interpreter's scripts
    candidates = [sysconfig.get_path("scripts")]
    user_scheme = {"nt": "nt_user", "posix": "posix_user"}.get(os.name)
    if sys.platform == "darwin" and sys._framework:  # type: ignore[attr-defined]
        user_scheme = "osx_framework_user"
    if user_scheme:
        try:
            candidates.append(sysconfig.get_path("scripts", scheme=user_scheme))
        except KeyError:
            pass
    candidates.append(os.path.dirname(sys.executable))
    for d in candidates:
        p = os.path.join(d, exe)
        if os.path.isfile(p):
            return p
    found = shutil.which("prs")
    if found:
        return found
    raise SetupError(
        "Could not find the prs binary. Install the p-rs wheel for your platform "
        f"(pip install p-rs), put prs on PATH (cargo install p-rs), or set {BINARY_ENV}.",
        "binary_missing",
    )
