// Docs shell: site header, a sidebar of pages, and the page content.
import type { ReactNode } from "react";
import { REPO, SiteHeader, links } from "../SiteHeader";

const BLOB = `${REPO}/blob/main`;

/** Docs pages on this site. Add new pages here. */
const PAGES = [
  { id: "overview", title: "Overview", href: (root: string) => links(root).overview },
  { id: "coming-from-p", title: "Coming from P", href: (root: string) => links(root).comingFromP },
];

/** Guides that still live in the repository. */
const GUIDES = [
  { title: "Python tests", href: `${BLOB}/docs/python.md` },
  { title: "Rust tests", href: `${BLOB}/examples/rust/README.md` },
  { title: "Java tests", href: `${BLOB}/java/README.md` },
  { title: "Go tests", href: `${BLOB}/go/README.md` },
  { title: "Checking logs (snappea observe)", href: `${BLOB}/docs/observe.md` },
  { title: "Testing and production", href: `${BLOB}/docs/production.md` },
  { title: "Rust library", href: `${BLOB}/docs/library.md` },
];

export function DocsLayout({ root, page, children }: { root: string; page: string; children: ReactNode }) {
  return (
    <div className="site docs">
      <SiteHeader root={root} current="docs" />
      <div className="docs-shell">
        <nav className="docs-side" aria-label="Docs">
          <p className="side-head">Docs</p>
          <ul>
            {PAGES.map((p) => (
              <li key={p.id}>
                <a href={p.href(root)} aria-current={p.id === page ? "page" : undefined}>{p.title}</a>
              </li>
            ))}
          </ul>
          <p className="side-head">Guides on GitHub</p>
          <ul>
            {GUIDES.map((g) => (
              <li key={g.title}>
                <a href={g.href} target="_blank" rel="noreferrer">{g.title} <span aria-hidden="true">↗</span></a>
              </li>
            ))}
          </ul>
        </nav>
        <main className="doc">{children}</main>
      </div>
    </div>
  );
}
