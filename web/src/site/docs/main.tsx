// One entry for every docs page; each page's HTML names it with data-page.
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DocsLayout } from "./DocsLayout";
import { ComingFromP } from "./ComingFromP";
import { Overview } from "./Overview";
import "../../styles.css";
import "../site.css";
import "./docs.css";

const ROOT = "../";
const PAGES = { overview: Overview, "coming-from-p": ComingFromP };

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
