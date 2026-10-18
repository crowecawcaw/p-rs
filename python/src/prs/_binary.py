"""Finds the `prs` executable: the one installed with this package, else one on PATH."""

from __future__ import annotations

import os
import shutil
import sys
import sysconfig
from functools import lru_cache

BINARY_ENV = "PRS_BINARY"


class PrsNotFound(RuntimeError):
    pass


@lru_cache(maxsize=None)
def find_prs() -> str:
    """Path of the `prs` binary. `PRS_BINARY` overrides the search."""
    override = os.environ.get(BINARY_ENV)
    if override:
        if not os.path.isfile(override):
            raise PrsNotFound(f"{BINARY_ENV}={override} is not a file")
        return override
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
    raise PrsNotFound(
        "Could not find the prs binary. Install the p-rs wheel for your platform, "
        f"put prs on PATH, or set {BINARY_ENV}."
    )
