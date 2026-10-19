"""Long-lived inventory worker: commands on stdin, replies on stdout, events to --events."""
import argparse
import json
import sys

from inventory import Inventory


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", required=True)
    parser.add_argument("--name", required=True)
    parser.add_argument("--events", required=True, help="file or pipe for committed events (JSON lines)")
    parser.add_argument("--allow-oversell", action="store_true")
    args = parser.parse_args()

    def reply(message):
        print(json.dumps(message), flush=True)

    with open(args.events, "w", buffering=1) as events:
        def emit(event):
            # `src` only helps people reading the record; the observer ignores it.
            events.write(json.dumps(dict(event, src=args.name)) + "\n")  # one line per write

        inventory = Inventory(args.db, emit, allow_oversell=args.allow_oversell)
        try:
            reply({"kind": "ready", "worker": args.name})
            for line in sys.stdin:
                command = json.loads(line)
                if command["operation"] == "stop":
                    reply({"kind": "ack", "id": command["id"], "stopped": True})
                    return
                accepted = inventory.apply(**{k: v for k, v in command.items() if k != "id"})
                reply({"kind": "ack", "id": command["id"], "accepted": accepted})
        finally:
            inventory.close()


if __name__ == "__main__":
    main()
