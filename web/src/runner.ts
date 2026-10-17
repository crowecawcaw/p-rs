// Talks to p.worker.ts. A check that runs too long can be cancelled, which
// replaces the worker.
import type { CheckRequest, CheckResult, CompileResult, Files, Request, WorkerReply } from "./protocol";

type Pending = { resolve: (r: never) => void; reject: (e: Error) => void };

export class Runner {
  private worker!: Worker;
  private nextId = 1;
  private pending = new Map<number, Pending>();

  constructor() {
    this.start();
  }

  private start() {
    this.worker = new Worker(new URL("./p.worker.ts", import.meta.url), { type: "module" });
    this.worker.onmessage = (e: MessageEvent<WorkerReply>) => {
      const p = this.pending.get(e.data.id);
      if (!p) return;
      this.pending.delete(e.data.id);
      if (e.data.error) p.reject(new Error(e.data.error));
      else p.resolve(e.data.result as never);
    };
  }

  private send<T>(req: Request): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (r: never) => void, reject });
      this.worker.postMessage({ id, req });
    });
  }

  compile(files: Files): Promise<CompileResult> {
    return this.send({ kind: "compile", files });
  }

  check(req: Omit<CheckRequest, "kind">): Promise<CheckResult> {
    return this.send({ kind: "check", ...req });
  }

  /** Stops whatever is running and rejects every pending request. */
  cancel() {
    this.worker.terminate();
    for (const p of this.pending.values()) p.reject(new Error("cancelled"));
    this.pending.clear();
    this.start();
  }
}
