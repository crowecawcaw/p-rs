#!/usr/bin/env bash
# Compares `p check` of the upstream P tool and snappea-compat on the same project, test case,
# schedule count and seed: console output (minus paths and timings) and the trace files.
#
#   UPSTREAM_P="dotnet /path/to/P/Bld/Drops/Release/Binaries/net8.0/p.dll" \
#   SNAPPEA_COMPAT=target/release/snappea-compat \
#   scripts/compare-upstream.sh path/to/project tcName 1000 42 [extra p check options]
#
# The project's parent directory is copied twice (projects may include siblings such as
# ../Common) so the tools do not share outputs.
set -euo pipefail
proj=$1; tc=$2; n=$3; seed=$4; shift 4
UPSTREAM_P=${UPSTREAM_P:?set UPSTREAM_P to the upstream p command}
SNAPPEA_COMPAT=$(realpath "${SNAPPEA_COMPAT:-target/release/snappea-compat}")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
proj=$(realpath "$proj"); name=$(basename "$proj")
mkdir -p "$work/up" "$work/snappea"
cp -r "$(dirname "$proj")/." "$work/up/"; cp -r "$(dirname "$proj")/." "$work/snappea/"

# Source discovery is logged differently: upstream finds the compiled .dll, snappea-compat
# loads the .pproj itself. Those lines are dropped before comparing.
normalize() {
  sed -E -e '/^\.\. (Searching for a P compiled file|Found a P compiled file|Checking )/d' \
         -e '/^==== Loading project file|^\.+ includes (p|foreign) file|^-{40}$/d' \
         -e "s#$work/(up|snappea)#<dir>#g" \
         -e 's#Elapsed [0-9.E-]+ sec( and used [0-9.]+ GB)?\.#Elapsed <t>#' \
         -e "s#task '[0-9]+'#task '<n>'#g" -e "s#task \\\\u0027[0-9]+\\\\u0027#task <n>#g"
}

(cd "$work/up/$name"; $UPSTREAM_P compile >compile.txt 2>&1; $UPSTREAM_P check -tc "$tc" -s "$n" --seed "$seed" "$@" >out.txt 2>&1 || true)
(cd "$work/snappea/$name"; "$SNAPPEA_COMPAT" check -tc "$tc" -s "$n" --seed "$seed" "$@" >out.txt 2>&1 || true)
work_up="$work/up/$name"; work_snappea="$work/snappea/$name"

status=0
echo "== console"
diff <(normalize <"$work_up/out.txt") <(normalize <"$work_snappea/out.txt") || status=1
for f in $(cd "$work_up" && find PCheckerOutput -type f \( -name '*.txt' -o -name '*.json' -o -name '*.schedule' \) 2>/dev/null | sort); do
  case "$f" in *pchecker_summary.txt) continue;; esac
  echo "== $f"
  if [ -f "$work_snappea/$f" ]; then
    diff <(sed '1s/^\xEF\xBB\xBF//' "$work_up/$f" | normalize) <(normalize <"$work_snappea/$f") || status=1
  else
    echo "missing in snappea"; status=1
  fi
done
exit $status
