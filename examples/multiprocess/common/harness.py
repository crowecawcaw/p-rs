"""Copyable integration-test supervisor, independent of process launch method.

Backends start(name, database, broken, stderr), alive(name, process),
cleanup(name, process), and absent(name, process). Each worker is a Popen with
text stdin/stdout pipes. Cleanup must remove the underlying worker, even when
its launcher (such as `docker run`) has already exited.
"""
import json
import queue
import sqlite3
import subprocess
import threading
import time
from contextlib import ExitStack
from pathlib import Path

from inventory import initialize_database

TIMEOUT = 30
SPEC = Path(__file__).resolve().parents[1] / "specs" / "inventory.p"


def run_scenario(backend, prs_bin, artifacts, scenario):
    """Run `healthy` or assert detection/cancellation of the `oversell` fault.

    Returns a summary suitable for JSON serialization. An expected violation is
    a passing integration test only after checking its exact cause and proving
    detection happened before producer EOF and that cleanup removed workers.
    """
    if scenario not in ("healthy", "oversell"):
        raise ValueError("unknown scenario")
    artifacts = Path(artifacts).resolve()
    artifacts.mkdir(parents=True, exist_ok=False)
    database = artifacts / "inventory.sqlite"
    initialize_database(database)
    messages = queue.Queue()
    workers = {}
    observer = None
    reader_threads = []
    emitters = set()
    pending = {}
    next_seq = 1
    command_id = 0
    alive_at_failure = False
    summary = None
    cleanup_errors = []
    with ExitStack() as files:
        events_file = files.enter_context((artifacts / "events.jsonl").open("w"))
        output = files.enter_context((artifacts / "observer.stdout").open("w"))
        errors = files.enter_context((artifacts / "observer.stderr").open("w"))
        try:
            observer = subprocess.Popen([
                str(prs_bin), "observe", "--pfiles", str(SPEC), "--spec", "InventoryCorrect",
                "--stdin", "--fail-fast", "--coverage", "--json-report", str(artifacts / "report.json"),
                "--outputDir", str(artifacts / "observe"),
            ], stdin=subprocess.PIPE, stdout=output, stderr=errors, text=True)

            def watch_observer():
                messages.put(("observer", observer.wait()))

            threading.Thread(target=watch_observer, daemon=True).start()

            def read_worker(name, process, worker_output):
                try:
                    for line in process.stdout:
                        worker_output.write(line)
                        worker_output.flush()
                        messages.put((name, json.loads(line)))
                except BaseException as exc:
                    messages.put((name, {"kind": "reader_error", "error": str(exc)}))
                finally:
                    messages.put((name, {"kind": "eof"}))

            for name in ("worker-a", "worker-b"):
                stderr = files.enter_context((artifacts / (name + ".stderr")).open("w"))
                process = backend.start(name, database, scenario == "oversell", stderr)
                workers[name] = process
                worker_output = files.enter_context((artifacts / (name + ".stdout")).open("w"))
                thread = threading.Thread(target=read_worker, args=(name, process, worker_output), daemon=True)
                thread.start()
                reader_threads.append(thread)

            def wait_for(predicate, *, allow_observer=False, allow_eof=False):
                nonlocal next_seq
                deadline = time.monotonic() + TIMEOUT
                while True:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise TimeoutError("timed out waiting for workers/observer; see " + str(artifacts))
                    try:
                        name, message = messages.get(timeout=remaining)
                    except queue.Empty as exc:
                        raise TimeoutError("timed out waiting for workers/observer; see " + str(artifacts)) from exc
                    if name == "observer":
                        if allow_observer:
                            return message
                        raise AssertionError("observer exited early (%s); see %s" % (message, artifacts))
                    kind = message["kind"]
                    if kind == "event":
                        emitters.add(name)
                        event = message["event"]
                        seq = event["seq"]
                        if seq < next_seq or seq in pending:
                            raise AssertionError("duplicate outbox event")
                        pending[seq] = event
                        while next_seq in pending:
                            line = json.dumps(pending.pop(next_seq)) + "\n"
                            events_file.write(line)
                            events_file.flush()
                            observer.stdin.write(line)
                            observer.stdin.flush()
                            next_seq += 1
                    elif kind == "reader_error":
                        raise AssertionError(message)
                    elif kind == "eof" and not allow_eof:
                        raise AssertionError("worker exited before its stop command: " + name)
                    if predicate(name, message):
                        return message

            ready = set()
            while len(ready) < len(workers):
                msg = wait_for(lambda name, msg: msg["kind"] == "ready")
                ready.add(msg["worker"])

            def command(name, operation, **fields):
                nonlocal command_id
                command_id += 1
                workers[name].stdin.write(json.dumps(dict(fields, id=command_id, operation=operation)) + "\n")
                workers[name].stdin.flush()
                return command_id

            def call(name, operation, expected, **fields):
                ident = command(name, operation, **fields)
                reply = wait_for(lambda who, msg: who == name and msg["kind"] == "ack" and msg["id"] == ident)
                if reply.get("accepted") != expected:
                    raise AssertionError("unexpected application result: " + str(reply))

            call("worker-a", "stock", True, units=3)
            call("worker-a", "reserve", True, units=2, order="order-a")
            if scenario == "oversell":
                # Do not wait for its ack: observer exit may race that ack after
                # the committed event. Both producers remain blocked for commands.
                command("worker-b", "reserve", units=2, order="order-b")
                observer_code = wait_for(lambda *_: False, allow_observer=True)
                alive_at_failure = all(backend.alive(name, proc) for name, proc in workers.items())
                if not alive_at_failure:
                    raise AssertionError("failure must be observed while both workers are still alive")
            else:
                call("worker-b", "reserve", False, units=2, order="order-b")
                call("worker-a", "release", True, order="order-a")
                call("worker-b", "reserve", True, units=2, order="order-b")
                call("worker-b", "release", True, order="order-b")
                for name in workers:
                    ident = command(name, "stop")
                    wait_for(lambda who, msg: who == name and msg["kind"] == "ack" and msg["id"] == ident, allow_eof=True)
                for process in workers.values():
                    if process.wait(timeout=TIMEOUT) != 0:
                        raise AssertionError("worker failed; inspect stderr")
                if pending:
                    raise AssertionError("gap in commit sequence")
                observer.stdin.close()  # Only healthy runs finish by EOF.
                observer_code = wait_for(lambda *_: False, allow_observer=True, allow_eof=True)
            report = json.loads((artifacts / "report.json").read_text())
            spec = report["specs"][0]
            if len(report["specs"]) != 1 or spec["spec"] != "InventoryCorrect":
                raise AssertionError("unexpected spec report")
            coverage = spec["coverage"]
            if coverage["scope"] != "observed_monitor" or coverage["states"]["covered"] != 2:
                raise AssertionError("missing observed monitor coverage")
            if emitters != set(workers):
                raise AssertionError("both workers must emit committed events")
            if scenario == "healthy":
                if observer_code != 0 or report["exit_code"] != 0 or spec["errors"] or not report["input_complete"] or report["stop_reason"] != "eof":
                    raise AssertionError("healthy history failed: " + str(report))
                if coverage["handlers"]["covered"] != 3 or spec["events_read"] != 5:
                    raise AssertionError("healthy scenario missed operations")
            else:
                violations = spec["errors"]
                if observer_code != 1 or report["exit_code"] != 2 or report["input_complete"] or report["stop_reason"] != "error":
                    raise AssertionError("expected native prs failure before EOF: " + str(report))
                if len(violations) != 1 or violations[0]["kind"] != "spec_violation" or "reservation exceeds available stock" not in str(violations[0]) or violations[0]["key"] != "coffee":
                    raise AssertionError("unexpected violation: " + str(violations))
            with sqlite3.connect(database) as db:
                final_stock = db.execute("SELECT available FROM stock WHERE sku='coffee'").fetchone()[0]
            if final_stock != (-1 if scenario == "oversell" else 3):
                raise AssertionError("unexpected persisted inventory stock")
            summary = {"scenario": scenario, "final_stock": final_stock, "observer_process_exit": observer_code,
                       "report_exit_code": report["exit_code"], "input_complete": report["input_complete"],
                       "emitters": sorted(emitters), "workers_alive_at_failure": alive_at_failure,
                       "coverage": coverage, "artifacts": str(artifacts)}
        finally:
            for name, process in workers.items():
                try:
                    backend.cleanup(name, process)
                    if not backend.absent(name, process):
                        raise AssertionError("worker survived cleanup: " + name)
                except BaseException as exc:
                    cleanup_errors.append(str(exc))
            if observer is not None:
                if observer.poll() is None:
                    observer.kill()
                observer.wait(timeout=TIMEOUT)
                if observer.stdin and not observer.stdin.closed:
                    observer.stdin.close()
            for process in workers.values():
                if process.stdin and not process.stdin.closed:
                    process.stdin.close()
                process.stdout.close()
            for thread in reader_threads:
                thread.join(timeout=TIMEOUT)
                if thread.is_alive():
                    cleanup_errors.append("worker reader thread survived cleanup")
            if cleanup_errors:
                raise RuntimeError("cleanup failed: " + "; ".join(cleanup_errors))
    summary["workers_removed"] = True
    (artifacts / "result.json").write_text(json.dumps(summary, indent=2) + "\n")
    print((artifacts / "observer.stdout").read_text(), end="")
    print(json.dumps({k: v for k, v in summary.items() if k != "coverage"}, sort_keys=True))
    return summary
