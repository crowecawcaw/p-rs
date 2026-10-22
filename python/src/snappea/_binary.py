"""Finds the `snappea` executable: `SNAPPEA_BIN`, else the one installed with this package, else
one on PATH."""

from __future__ import annotations

import os
import shutil
import sys
import sysconfig
from functools import lru_cache

from ._errors import SetupError, warn

BINARY_ENV = "SNAPPEA_BIN"
#: Deprecated spelling of `SNAPPEA_BIN`.
OLD_BINARY_ENV = "SNAPPEA_BINARY"


class SnappeaNotFound(SetupError, RuntimeError):
    """No `snappea` binary: a :class:`SetupError` of kind ``binary_missing``."""

    def __init__(self, message: str):
        super().__init__(message, "binary_missing")


@lru_cache(maxsize=None)
def find_snappea() -> str:
    """Path of the `snappea` binary: `SNAPPEA_BIN`, the copy installed with this package, or
    `snappea` on PATH."""
    name, override = BINARY_ENV, os.environ.get(BINARY_ENV)
    if not override and os.environ.get(OLD_BINARY_ENV):
        name, override = OLD_BINARY_ENV, os.environ[OLD_BINARY_ENV]
        warn(f"{OLD_BINARY_ENV} is deprecated; set {BINARY_ENV} instead", DeprecationWarning)
    if override:
        if not os.path.isfile(override):
            raise SnappeaNotFound(f"{name}={override} is not a file")
        return os.path.abspath(override)
    exe = "snappea" + (sysconfig.get_config_var("EXE") or "")
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
    found = shutil.which("snappea")
    if found:
        return found
    raise SnappeaNotFound(
        "Could not find the snappea binary. Install the snappea wheel for your platform "
        f"(pip install pysnappea), put snappea on PATH (cargo install snappea), or set {BINARY_ENV}."
    )
