import type { ComponentType } from "react";

export function hydrateIslands(loaders: Record<string, () => Promise<ComponentType>>) {
  for (const el of document.querySelectorAll<HTMLElement>("[data-island]")) {
    const load = loaders[el.dataset.island!];
    if (!load) throw new Error(`Unknown island: ${el.dataset.island}`);
    // Pages without islands don't need to download the React runtime.
    void Promise.all([load(), import("react"), import("react-dom/client")]).then(
      ([Component, { createElement }, { hydrateRoot }]) => hydrateRoot(el, createElement(Component)),
    );
  }
}
