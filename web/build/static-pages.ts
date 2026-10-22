import { createServer, type Plugin, type ViteDevServer } from "vite";
import react from "@vitejs/plugin-react";

/** Render prose at build time (and on dev requests), preserving Vite's asset URLs. */
export function staticPages(): Plugin {
  let renderer: ViteDevServer;
  return {
    name: "static-pages",
    async buildStart() {
      // A separate module runner keeps rendering code out of the browser bundles.
      renderer = await createServer({
        configFile: false,
        plugins: [react()],
        cacheDir: "node_modules/.vite-render",
        optimizeDeps: { noDiscovery: true, include: [] },
        server: { middlewareMode: true, hmr: false, ws: false },
        appType: "custom",
      });
    },
    transformIndexHtml: {
      order: "pre",
      async handler(html) {
        const root = /<div id="root"(?: data-page="([^"]+)")?><\/div>/;
        const match = html.match(root);
        // The playground stays a client application.
        if (!match || (!match[1] && !html.includes("/src/site/home/main.tsx"))) return html;
        const { render } = await renderer.ssrLoadModule("/src/site/render.tsx");
        const content = render(match[1] ?? "home");
        return html.replace(root, () => match[0].replace("</div>", () => `${content}</div>`));
      },
    },
    async closeBundle() {
      await renderer?.close();
    },
  };
}
