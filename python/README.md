# p-rs for Python

Check [P](https://p-org.github.io/P/) specs against events your code emits during its
ordinary tests. The package brings the `prs` binary, so `pip install p-rs` is the whole
setup.

```python
from prs import emit

def grant(self, client, lock):
    ...
    emit("eEnter", key=lock, client=client)
```

```toml
# pyproject.toml
[tool.prs]
spec_files = ["specs/*.p"]
```

Every pytest test now also checks every spec in `specs/`. See
[docs/python.md](https://github.com/crowecawcaw/p-rs/blob/main/docs/python.md).
