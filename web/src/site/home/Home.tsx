// Landing page: what Snap Pea is, in one screen.
import { REPO, SiteHeader, links } from "../SiteHeader";
import { SpeedBars } from "./SpeedBars";

const BLOB = `${REPO}/blob/main`;
const P_DOCS = "https://p-org.github.io/P/";
const l = links("./");

const FEATURES = [
  {
    title: "One file, runs anywhere",
    text: "One 4 MB binary parses, type-checks and model-checks P directly: no generated C#, no .NET SDK, no Java for PObserve. Drop it into CI, a container or an agent's sandbox.",
    link: { label: "Install", href: "#install" },
  },
  {
    title: "Model checking",
    text: "Explore message orderings, failures and timing with random, PCT, POS or exhaustive search, on every CPU core. Replay any bug from its schedule file.",
    link: { label: "Overview", href: l.overview },
  },
  {
    title: "Application integration",
    text: "Record events from ordinary tests in Python, Rust, Java and Go, and check them against P specs. snappea observe checks logs from running systems.",
    link: { label: "Integration examples", href: `${BLOB}/examples/README.md` },
  },
  {
    title: "Compatible with P",
    text: "snappea-compat is a drop-in replacement for the p command. snappea passes upstream's 424 compiler tests, 249 checker tests and its tutorials.",
    link: { label: "Coming from P", href: l.comingFromP },
  },
];

export function Home() {
  return (
    <div className="site home">
      <SiteHeader root="./" current="home" />
      <main>
        <section className="home-hero">
          <p className="eyebrow">The P language, in Rust</p>
          <h1>Fast formal models that also check your code</h1>
          <p className="home-lede">
            Snap Pea is a Rust implementation of <a href="#what-is-p">P</a>: a parser, type checker and model checker in
            one binary, with no compile step. It's built for speed, and for checking your application's real events
            against the same specs that checked its design.
          </p>
          <div className="home-ctas">
            <a className="button primary" href={l.overview}>Read the overview</a>
            <a className="button" href={l.playground}>Try the playground</a>
            <a className="button ghost" href={REPO} target="_blank" rel="noreferrer">GitHub</a>
          </div>
        </section>

        <section className="home-band">
          <div className="home-band-text">
            <h2>A fast edit-check loop</h2>
            <p>
              Snap Pea interprets models directly instead of compiling them, and runs schedules in parallel, so short
              checks finish almost instantly. That makes it easy to check a model on every edit, in CI and inside
              application test suites.
            </p>
            <p>Checking the Raft tutorial, 120 schedules:</p>
            <p className="muted small">
              Warmed up, snappea checks 290 schedules per second, and P checks 4.{" "}
              <a href={`${BLOB}/docs/development.md#benchmarks`}>Benchmarks</a>
            </p>
          </div>
          <SpeedBars />
        </section>

        <section className="home-features">
          {FEATURES.map((f) => (
            <article key={f.title} className="feature">
              <h3>{f.title}</h3>
              <p>{f.text}</p>
              <a href={f.link.href}>{f.link.label} →</a>
            </article>
          ))}
        </section>

        <section className="home-p" id="what-is-p">
          <div className="home-p-text">
            <h2>What is P?</h2>
            <p>
              <a href="https://github.com/p-org/P" target="_blank" rel="noreferrer">P</a> is a language for modeling
              distributed systems as communicating state machines, and for checking those models against specs. It
              catches the design bugs that hide in rare orderings of messages, failures and retries, before they reach
              code.
            </p>
            <p>
              It works at scale: P is used in industry to check the designs of large production systems, and its
              tutorials and case studies show how to model everything from two-phase commit to Paxos and Raft.
            </p>
            <p>
              Snap Pea aims to make P easier to adopt in more projects. It keeps P's language and results, and is tested
              against P's own test suites and tutorials.
            </p>
            <p className="home-p-links">
              <a href={P_DOCS} target="_blank" rel="noreferrer">P documentation</a>
              <a href={`${P_DOCS}tutsoutline/`} target="_blank" rel="noreferrer">Tutorials</a>
              <a href={`${P_DOCS}casestudies/`} target="_blank" rel="noreferrer">Case studies</a>
            </p>
          </div>
        </section>

        <section className="home-install" id="install">
          <h2>Install</h2>
          <p>The checker, from crates.io or PyPI:</p>
          <pre className="terminal">
            <code>
              <span className="prompt">$ </span>cargo install snappea{"   "}<span className="cmt"># snappea and snappea-compat, with a Rust toolchain</span>{"\n"}
              <span className="prompt">$ </span>pip install pysnappea{"     "}<span className="cmt"># prebuilt snappea, plus the Python helper</span>{"\n"}
              <span className="prompt">$ </span>snappea check{"            "}<span className="cmt"># check every test case in the .pproj</span>
            </code>
          </pre>
          <p>
            Each <a href={`${REPO}/releases`}>GitHub release</a> also has prebuilt <code>snappea</code> binaries for Linux,
            macOS and Windows. The application helpers are one <code>snappea</code> package per language:
          </p>
          <pre className="terminal">
            <code>
              <span className="prompt">$ </span>pip install pysnappea{"                       "}<span className="cmt"># Python</span>{"\n"}
              <span className="prompt">$ </span>cargo add --dev snappea{"                   "}<span className="cmt"># Rust</span>{"\n"}
              <span className="prompt">$ </span>go get github.com/crowecawcaw/snappea{"     "}<span className="cmt"># Go</span>
            </code>
          </pre>
          <p>
            Java: <code>io.github.crowecawcaw:snappea</code> on Maven Central, plus a natives jar with the checker (
            <a href={`${BLOB}/java/README.md#install`}>Maven and Gradle snippets</a>).
          </p>
        </section>
      </main>
      <footer className="site-footer">
        <span>MIT or Apache-2.0</span>
        <a href={REPO} target="_blank" rel="noreferrer">GitHub</a>
        <a href="https://github.com/p-org/P" target="_blank" rel="noreferrer">Upstream P</a>
      </footer>
    </div>
  );
}
