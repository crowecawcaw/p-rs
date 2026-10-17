#!/bin/bash
# Prints, per generated class in upstream's C# (`T.cs`), the sorted action names its state
# attributes reference; compare with `cargo run -p prs-sema --example csnames -- <files>`.
awk '
  /internal partial class [A-Za-z0-9_]+ : (StateMachine|Monitor)/ { if (cls != "") print cls; cls = $4 ":"; next }
  { while (match($0, /\[(OnEntry|OnExit)\(nameof\([A-Za-z0-9_]+\)\)\]|nameof\([A-Za-z0-9_]+\)\)\]/)) {
      s = substr($0, RSTART, RLENGTH); sub(/.*nameof\(/, "", s); sub(/\).*/, "", s)
      cls = cls " " s; $0 = substr($0, RSTART + RLENGTH) } }
  END { if (cls != "") print cls }' "$1" |
while read -r c rest; do echo "$c $(echo $rest | tr ' ' '\n' | grep . | sort | tr '\n' ' ' | sed 's/ $//')"; done | sed 's/ $//' | sort
