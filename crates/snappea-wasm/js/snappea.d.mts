// Types for snappea.mjs.

export interface PResult {
  exitCode: number;
  stdout: string;
  stderr: string;
  /** The whole in-memory file system after the command ({absolutePath: text}). */
  files: Record<string, string>;
}

export interface PTool {
  /** Runs `p <args>` over an in-memory file system ({absolutePath: text}). */
  run(args: string[], files?: Record<string, string>, opts?: { cwd?: string; seed?: number }): PResult;
}

export function loadP(source: BufferSource | Response | PromiseLike<Response>): Promise<PTool>;
