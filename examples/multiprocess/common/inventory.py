"""A database-backed service with a transactional event outbox.

The supplied sink publishes committed events; it is independent of the application
business rules. A real publisher can retry outbox rows after a crash.
"""
import json
import sqlite3


def initialize_database(path):
    with sqlite3.connect(path) as db:
        db.executescript("""
            CREATE TABLE stock (sku TEXT PRIMARY KEY, available INTEGER NOT NULL);
            CREATE TABLE reservations (order_id TEXT PRIMARY KEY, sku TEXT, units INTEGER);
            CREATE TABLE outbox (seq INTEGER PRIMARY KEY AUTOINCREMENT, body TEXT NOT NULL);
        """)


class Inventory:
    def __init__(self, path, emit, *, allow_oversell=False):
        self.db = sqlite3.connect(path, timeout=10)
        self.emit = emit
        # Demonstration fault in a business guard, never in the event publisher.
        self.allow_oversell = allow_oversell

    def apply(self, operation, *, sku="coffee", units=0, order=""):
        self.db.execute("BEGIN IMMEDIATE")
        event = None
        try:
            if operation == "stock":
                if units < 0:
                    raise ValueError("negative stock")
                self.db.execute("INSERT INTO stock VALUES (?, ?)", (sku, units))
                event = ("eStocked", {"units": units})
            elif operation == "reserve":
                row = self.db.execute("SELECT available FROM stock WHERE sku=?", (sku,)).fetchone()
                duplicate = self.db.execute("SELECT 1 FROM reservations WHERE order_id=?", (order,)).fetchone()
                if row is not None and units > 0 and not duplicate and (self.allow_oversell or row[0] >= units):
                    self.db.execute("UPDATE stock SET available=available-? WHERE sku=?", (units, sku))
                    self.db.execute("INSERT INTO reservations VALUES (?, ?, ?)", (order, sku, units))
                    event = ("eReserved", {"order": order, "units": units})
            elif operation == "release":
                row = self.db.execute("SELECT units FROM reservations WHERE order_id=? AND sku=?", (order, sku)).fetchone()
                if row is not None:
                    self.db.execute("UPDATE stock SET available=available+? WHERE sku=?", (row[0], sku))
                    self.db.execute("DELETE FROM reservations WHERE order_id=?", (order,))
                    event = ("eReleased", {"order": order, "units": row[0]})
            else:
                raise ValueError("unknown operation: " + operation)
            published = None
            if event:
                body = {"event": event[0], "key": sku, "payload": event[1]}
                seq = self.db.execute("INSERT INTO outbox(body) VALUES (?)", (json.dumps(body),)).lastrowid
                # This is a logical clock reflecting the database's serialization,
                # suitable for this safety spec (which makes no elapsed-time claims).
                published = dict(body, timestamp=seq, seq=seq)
            self.db.commit()
        except BaseException:
            self.db.rollback()
            raise
        if published:
            self.emit(published)  # Never publish a transaction that later rolls back.
        return bool(event)

    def close(self):
        self.db.close()
