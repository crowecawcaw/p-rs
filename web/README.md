# P Playground

A web page for editing and model-checking P models in the browser. It runs the
`pnext-wasm` build of the toolchain in a Web Worker; nothing is sent to a server.

```sh
cd web
npm ci
npm run dev      # builds the wasm module (needs the wasm32-unknown-unknown target), then starts Vite
npm run build    # production build in dist/
```

- Demo models come straight from the repository (`tests/upstream/Tutorial`,
  `design/time/RetryBudget`, `web/demos`); see `src/demos.ts`.
- Projects are compiled with `p compile -pf <every .p file>`, so `.pproj` files are not used.
- Share links hold the whole project, deflated, in the URL hash.
- `.github/workflows/pages.yml` publishes `main` to GitHub Pages and maintainer PRs to `pr/<number>/`.
