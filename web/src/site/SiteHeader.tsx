// Header shared by the landing page and the docs. The playground has its own.
export const REPO = "https://github.com/crowecawcaw/p-rs";

export type Section = "home" | "docs" | "playground";

/** Links between pages. `root` is the relative path to the site root ("./" or "../"). */
export const links = (root: string) => ({
  home: root,
  overview: `${root}docs/overview.html`,
  comingFromP: `${root}docs/coming-from-p.html`,
  playground: `${root}playground.html`,
});

export function SiteHeader({ root, current }: { root: string; current: Section }) {
  const l = links(root);
  return (
    <header className="top site-top">
      <a className="brand" href={l.home} aria-label="p-rs home">
        <span className="logo">p-rs</span>
      </a>
      <nav className="site-nav">
        <a href={l.overview} aria-current={current === "docs" ? "page" : undefined}>Docs</a>
        <a href={l.playground} aria-current={current === "playground" ? "page" : undefined}>Playground</a>
        <a href={REPO} target="_blank" rel="noreferrer">GitHub</a>
      </nav>
    </header>
  );
}
