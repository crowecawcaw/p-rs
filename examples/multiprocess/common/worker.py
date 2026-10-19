"""Long-lived inventory worker: control commands in, committed events out."""
import argparse
import json
import sys

from inventory import Inventory


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", required=True)
    parser.add_argument("--name", required=True)
    parser.add_argument("--allow-oversell", action="store_true")
    args = parser.parse_args()

    def send(message):
        print(json.dumps(message), flush=True)

    def emit(event):
        send({"kind": "event", "worker": args.name, "event": event})

    inventory = Inventory(args.db, emit, allow_oversell=args.allow_oversell)
    try:
        send({"kind": "ready", "worker": args.name})
        for line in sys.stdin:
            command = json.loads(line)
            if command["operation"] == "stop":
                send({"kind": "ack", "id": command["id"], "stopped": True})
                return
            accepted = inventory.apply(**{k: v for k, v in command.items() if k != "id"})
            send({"kind": "ack", "id": command["id"], "accepted": accepted})
    finally:
        inventory.close()


if __name__ == "__main__":
    main()
