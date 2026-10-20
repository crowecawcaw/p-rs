#!/bin/sh
# Runs cmd/inventory the production way, with the same snappeaemit.Emit calls its tests
# check, and checks the logs it writes with `snappea observe`. See docs/production.md.
#
#   SNAPPEA_BIN=/path/to/snappea sh production.sh
set -eu
cd "$(dirname "$0")"
snappea=${SNAPPEA_BIN:-snappea}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
go build -o "$work/inventory" ./cmd/inventory

# Two hosts, one per product, each appending to its own log. Each key (product) has one
# writer, so clock skew between hosts cannot reorder a key's events.
printf 'stock coffee 3\nreserve coffee order-1 2\nreserve coffee order-2 2\nrelease coffee order-1\n' |
	"$work/inventory" -events "$work/log/app/events/host-a.jsonl"
printf 'stock tea 2\nreserve tea order-1 2\n' |
	"$work/inventory" -events "$work/log/app/events/host-b.jsonl"

echo "== batch: every log, after the fact"
"$snappea" observe specs/inventory.p "$work/log/app/events/*.jsonl" --spec InventoryCorrect --strict \
	--json-report "$work/batch.json" >/dev/null
cat "$work/batch.json"

echo "== live: one stream, checked as it arrives"
cat "$work"/log/app/events/*.jsonl |
	"$snappea" observe specs/inventory.p --spec InventoryCorrect --stdin \
		--json-report "$work/live.json" >/dev/null
cat "$work/live.json"

echo "== negative control: a forged oversell must be a violation (exit 1)"
echo '{"key":"coffee","timestamp":"2100-01-01T00:00:00Z","event":"eReserved","payload":{"order":"forged","units":5}}' \
	>"$work/log/app/events/host-c.jsonl"
status=0
"$snappea" observe specs/inventory.p "$work/log/app/events/*.jsonl" --spec InventoryCorrect \
	--json-report "$work/bad.json" >/dev/null || status=$?
if [ "$status" -ne 1 ] || ! grep -q '"spec_violation"' "$work/bad.json"; then
	echo "expected a spec violation (exit 1), got exit $status" >&2
	cat "$work/bad.json" >&2
	exit 1
fi
grep -o '"message": *"[^"]*"' "$work/bad.json"
echo "ok"
