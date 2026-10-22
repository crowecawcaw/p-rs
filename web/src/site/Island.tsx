import type { ReactNode } from "react";

/** Keep a diagram's initial content in HTML; only this subtree needs React. */
export function Island({ name, children }: { name: string; children: ReactNode }) {
  return <div data-island={name} style={{ display: "contents" }}>{children}</div>;
}
