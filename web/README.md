# Website

The project site, built with Vite and React:

- `index.html`: the landing page (`src/site/home/`).
- `docs/*.html`: docs pages (`src/site/docs/`). Add a page by adding an HTML entry
  in `vite.config.ts` and a link in `src/site/docs/DocsLayout.tsx`.
- `playground.html`: the P Playground (`src/App.tsx`), for editing and
  model-checking P models in the browser. It runs the `snappea-wasm` build of the
  toolchain in a Web Worker; nothing is sent to a server.

The docs diagrams are React components driven by scripted data
(`src/site/docs/diagrams/`).

```sh
cd web
npm ci
npm run dev      # builds the wasm module (needs the wasm32-unknown-unknown target), then starts Vite
npm run build    # production build in dist/
```

- Demo models come straight from the repository (`tests/upstream/Tutorial`,
  `design/time/RetryBudget`, `web/demos`); see `src/demos.ts`.
- Projects are checked with `snappea check <every .p file> <test>`, so `.pproj` files are not used.
- `.github/workflows/pages.yml` publishes `main` to GitHub Pages and maintainer PRs to `pr/<number>/`.
