// Docs shell: site header, a sidebar of pages, and the page content.
import type { ReactNode } from "react";
import { REPO, SiteHeader, links } from "../SiteHeader";

const BLOB = `${REPO}/blob/main`;

type Page = { id: string; title: string; href: (root: string) => string };

/** Docs pages on this site. Add new pages here. */
const PAGES: Page[] = [
  { id: "overview", title: "Overview", href: (root) => links(root).overview },
  { id: "coming-from-p", title: "Coming from P", href: (root) => links(root).comingFromP },
  { id: "cli", title: "Command line", href: (root) => links(root).cli },
  { id: "github-actions", title: "GitHub Actions", href: (root) => links(root).githubActions },
];

/** One page per language helper; each follows docs/language-support.md#docs-page. */
const LANGUAGES: Page[] = [
  { id: "python", title: "Python", href: (root) => links(root).python },
  { id: "rust", title: "Rust", href: (root) => links(root).rust },
  { id: "go", title: "Go", href: (root) => links(root).go },
  { id: "java", title: "Java", href: (root) => links(root).java },
];

/** Guides that still live in the repository. */
const GUIDES = [
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
          <PageList pages={PAGES} root={root} page={page} />
          <p className="side-head">Languages</p>
          <PageList pages={LANGUAGES} root={root} page={page} />
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

function PageList({ pages, root, page }: { pages: Page[]; root: string; page: string }) {
  return (
    <ul>
      {pages.map((p) => (
        <li key={p.id}>
          <a href={p.href(root)} aria-current={p.id === page ? "page" : undefined}>{p.title}</a>
        </li>
      ))}
    </ul>
  );
}
