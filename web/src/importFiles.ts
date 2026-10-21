// Reads files dropped on (or picked in) the playground: .p files, folders and .zip
// archives. Returns project-relative paths with a shared top folder removed, so
// `Paxos.zip` holding `Paxos/PSrc/x.p` opens as `PSrc/x.p`.
import { unzipSync, strFromU8 } from "fflate";
import type { Files } from "./protocol";

const TEXT = /\.(p|toml|json|txt|md)$/i;
const SKIP = /(^|\/)(\.[^/]*|__MACOSX|PCheckerOutput|PGenerated|node_modules|target)(\/|$)/;
const MAX_FILES = 500;
const MAX_BYTES = 2_000_000;

type Entry = { path: string; read: () => Promise<Uint8Array> };

function keep(path: string) {
  return TEXT.test(path) && !SKIP.test(path);
}

async function fromZip(bytes: Uint8Array, prefix: string, out: Entry[]) {
  const files = unzipSync(bytes, { filter: (f) => keep(f.name) && f.originalSize <= MAX_BYTES });
  for (const [name, data] of Object.entries(files)) out.push({ path: prefix + name, read: async () => data });
}

async function fromFile(file: File, path: string, out: Entry[]) {
  if (/\.zip$/i.test(file.name)) {
    await fromZip(new Uint8Array(await file.arrayBuffer()), path.replace(/[^/]*$/, ""), out);
  } else if (keep(path) && file.size <= MAX_BYTES) {
    out.push({ path, read: async () => new Uint8Array(await file.arrayBuffer()) });
  }
}

function readAll(dir: FileSystemDirectoryEntry): Promise<FileSystemEntry[]> {
  const reader = dir.createReader();
  const all: FileSystemEntry[] = [];
  return new Promise((resolve, reject) => {
    const next = () =>
      reader.readEntries((batch) => {
        if (!batch.length) return resolve(all);
        all.push(...batch);
        next();
      }, reject);
    next();
  });
}

async function fromEntry(entry: FileSystemEntry, out: Entry[]) {
  const path = entry.fullPath.replace(/^\//, "");
  if (entry.isDirectory) {
    if (SKIP.test(path)) return;
    for (const child of await readAll(entry as FileSystemDirectoryEntry)) await fromEntry(child, out);
  } else {
    const file = await new Promise<File>((res, rej) => (entry as FileSystemFileEntry).file(res, rej));
    await fromFile(file, path, out);
  }
}

/** Removes top folders every path shares. */
function stripCommonRoot(paths: string[]): (p: string) => string {
  let drop = 0;
  for (;;) {
    const first = paths[0].slice(drop).split("/");
    if (first.length < 2) break;
    const dir = first[0] + "/";
    if (!paths.every((p) => p.slice(drop).startsWith(dir))) break;
    drop += dir.length;
  }
  return (p) => p.slice(drop);
}

async function toFiles(entries: Entry[]): Promise<Files> {
  if (entries.length > MAX_FILES) throw new Error(`Too many files (${entries.length}); the limit is ${MAX_FILES}.`);
  if (!entries.some((e) => /\.p$/i.test(e.path))) throw new Error("No .p files found.");
  const strip = stripCommonRoot(entries.map((e) => e.path));
  const files: Files = {};
  for (const e of entries) files[strip(e.path)] = strFromU8(await e.read());
  return files;
}

/** Files from a drop: files, folders (where the browser supports it) and zips. */
export async function filesFromDrop(dt: DataTransfer): Promise<Files> {
  // The browser empties the DataTransfer once the drop handler returns, so take what it holds first.
  const out: Entry[] = [];
  const entries = [...dt.items].map((i) => i.webkitGetAsEntry?.()).filter((e): e is FileSystemEntry => !!e);
  const files = [...dt.files];
  if (entries.length) for (const e of entries) await fromEntry(e, out);
  else for (const f of files) await fromFile(f, f.name, out);
  return toFiles(out);
}

/** Files from an `<input type="file">`. */
export async function filesFromList(list: FileList): Promise<Files> {
  const out: Entry[] = [];
  for (const f of [...list]) await fromFile(f, f.webkitRelativePath || f.name, out);
  return toFiles(out);
}
