// Share links: the project is deflated and base64url-encoded into the URL hash.
import type { Files } from "./protocol";

export interface Shared {
  demo?: string;
  files: Files;
  open?: string;
  testCase?: string;
  schedules?: number;
  extraArgs?: string;
}

const PREFIX = "#code=";

async function pipe(bytes: Uint8Array<ArrayBuffer>, stream: CompressionStream | DecompressionStream): Promise<Uint8Array> {
  const out = new Blob([bytes]).stream().pipeThrough(stream);
  return new Uint8Array(await new Response(out).arrayBuffer());
}

function toBase64Url(bytes: Uint8Array): string {
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function fromBase64Url(s: string): Uint8Array<ArrayBuffer> {
  const b = atob(s.replace(/-/g, "+").replace(/_/g, "/"));
  return Uint8Array.from(b, (c) => c.charCodeAt(0));
}

export async function encodeShare(s: Shared): Promise<string> {
  const bytes = await pipe(new TextEncoder().encode(JSON.stringify(s)), new CompressionStream("deflate-raw"));
  return PREFIX + toBase64Url(bytes);
}

export async function decodeShare(hash: string): Promise<Shared | undefined> {
  if (!hash.startsWith(PREFIX)) return undefined;
  try {
    const bytes = await pipe(fromBase64Url(hash.slice(PREFIX.length)), new DecompressionStream("deflate-raw"));
    const s = JSON.parse(new TextDecoder().decode(bytes)) as Shared;
    return s && typeof s.files === "object" ? s : undefined;
  } catch {
    return undefined;
  }
}
