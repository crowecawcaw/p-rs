// One entry for every docs page; each page's HTML names it with data-page.
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DocsLayout } from "./DocsLayout";
import { Cli } from "./Cli";
import { ComingFromP } from "./ComingFromP";
import { GitHubActions } from "./GitHubActions";
import { Overview } from "./Overview";
import { Go } from "./languages/Go";
import { Java } from "./languages/Java";
import { Python } from "./languages/Python";
import { Rust } from "./languages/Rust";
import "../../styles.css";
import "../site.css";
import "./docs.css";

const ROOT = "../";
const PAGES = { overview: Overview, "coming-from-p": ComingFromP, cli: Cli, "github-actions": GitHubActions, python: Python, rust: Rust, go: Go, java: Java };

const el = document.getElementById("root")!;
const id = (el.dataset.page ?? "overview") as keyof typeof PAGES;
const Page = PAGES[id] ?? Overview;

createRoot(el).render(
  <StrictMode>
    <DocsLayout root={ROOT} page={id}>
      <Page root={ROOT} />
    </DocsLayout>
  </StrictMode>,
);
