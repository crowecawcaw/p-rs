"""`[tool.prs]` in pyproject.toml."""

from __future__ import annotations

import glob
import os
from dataclasses import dataclass, field
from functools import lru_cache
from typing import List, Optional

try:
    import tomllib  # type: ignore[import-not-found]
except ImportError:  # Python < 3.11
    import tomli as tomllib  # type: ignore[no-redef]

MODES = ("auto", "marked", "off")
SPEC_FILES_ENV = "PRS_SPEC_FILES"


class ConfigError(Exception):
    pass


@dataclass
class Config:
    #: Glob patterns of the P files holding the specs, relative to `root`.
    spec_files: List[str] = field(default_factory=list)
    #: Specs not checked anywhere.
    exclude: List[str] = field(default_factory=list)
    #: "auto": every test; "marked": tests marked `@pytest.mark.prs`; "off".
    mode: str = "auto"
    #: Folder of the pyproject.toml the settings came from.
    root: str = "."
    #: The pyproject.toml, if one had a [tool.prs] table.
    path: Optional[str] = None
    #: Require relevant events for every selected spec in each checked test.
    strict: bool = False

    def files(self) -> List[str]:
        """The P files, expanded and sorted."""
        out: List[str] = []
        for pat in self.spec_files:
            full = pat if os.path.isabs(pat) else os.path.join(self.root, pat)
            matches = sorted(glob.glob(full, recursive=True))
            if not matches:
                raise ConfigError(f"[tool.prs] spec_files: {pat!r} matches no files (looked in {self.root})")
            for m in matches:
                if os.path.isdir(m):
                    matches_in = sorted(glob.glob(os.path.join(m, "**", "*.p"), recursive=True))
                    out.extend(x for x in matches_in if x not in out)
                elif m not in out:
                    out.append(m)
        return out


def parse(table: dict, root: str, path: Optional[str]) -> Config:
    def str_list(name: str) -> List[str]:
        v = table.get(name, [])
        if isinstance(v, str):
            v = [v]
        if not isinstance(v, list) or not all(isinstance(x, str) for x in v):
            raise ConfigError(f"[tool.prs] {name} must be a list of strings")
        return v

    unknown = set(table) - {"spec_files", "exclude", "mode", "strict"}
    if unknown:
        raise ConfigError(f"[tool.prs]: unknown setting(s) {', '.join(sorted(unknown))}")
    mode = table.get("mode", "auto")
    if mode not in MODES:
        raise ConfigError(f"[tool.prs] mode must be one of {', '.join(MODES)}, not {mode!r}")
    strict = table.get("strict", False)
    if not isinstance(strict, bool):
        raise ConfigError("[tool.prs] strict must be a boolean")
    return Config(str_list("spec_files"), str_list("exclude"), mode, root, path, strict)


def load(start: str) -> Config:
    """The settings of the nearest pyproject.toml with a [tool.prs] table, searching
    upwards from `start`. `PRS_SPEC_FILES` (paths separated by os.pathsep) overrides
    `spec_files`."""
    cfg = _load(os.path.abspath(start))
    env = os.environ.get(SPEC_FILES_ENV)
    if env:
        cfg = Config([p for p in env.split(os.pathsep) if p], cfg.exclude, cfg.mode, os.getcwd(), cfg.path, cfg.strict)
    return cfg


@lru_cache(maxsize=None)
def _load(start: str) -> Config:
    d = start
    while True:
        p = os.path.join(d, "pyproject.toml")
        if os.path.isfile(p):
            with open(p, "rb") as f:
                try:
                    data = tomllib.load(f)
                except tomllib.TOMLDecodeError as e:
                    raise ConfigError(f"{p}: {e}") from None
            table = data.get("tool", {}).get("prs")
            if table is not None:
                return parse(table, d, p)
        parent = os.path.dirname(d)
        if parent == d:
            return Config(root=start)
        d = parent
