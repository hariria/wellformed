import { isAbsolute, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { reactRouter } from "@react-router/dev/vite";
import tailwindcss from "@tailwindcss/vite";
import mdx from "fumadocs-mdx/vite";
import { defineConfig, type Plugin } from "vite";

const docsContentDir = fileURLToPath(
  new URL("./content/docs", import.meta.url),
);

function isDocsMdxFile(file: string): boolean {
  const relativePath = relative(docsContentDir, file);
  return (
    relativePath.length > 0 &&
    !relativePath.startsWith("..") &&
    !isAbsolute(relativePath) &&
    relativePath.endsWith(".mdx")
  );
}

// React Router builds its prerender/data endpoint table when the dev server
// starts. If a new docs page is added later, the page route can render while
// the corresponding `/docs/<slug>.data` endpoint still 404s until restart.
function restartOnDocsRouteChanges(): Plugin {
  return {
    name: "docs-route-restart",
    apply: "serve",
    configureServer(server) {
      let timeout: ReturnType<typeof setTimeout> | undefined;

      const restart = (file: string) => {
        if (!isDocsMdxFile(file)) return;
        if (timeout) clearTimeout(timeout);
        timeout = setTimeout(() => {
          server.config.logger.info(
            "[docs] docs page list changed; restarting dev server",
          );
          void server.restart();
        }, 100);
      };

      server.watcher.on("add", restart);
      server.watcher.on("unlink", restart);

      return () => {
        if (timeout) clearTimeout(timeout);
        server.watcher.off("add", restart);
        server.watcher.off("unlink", restart);
      };
    },
  };
}

// Serve `/docs/<slug>.md` during `pnpm dev`. In production these are static
// files emitted by the build (scripts/relocate-md.mjs); React Router's `docs/*`
// splat would otherwise render them as HTML.
function docsMarkdownDevServer(): Plugin {
  return {
    name: "docs-markdown-dev",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        const pathname = (req.url ?? "").split("?")[0];
        if (pathname === "/llms.txt") {
          res.statusCode = 404;
          res.end();
          return;
        }

        const match = pathname.match(/^\/docs\/(.+)\.md$/);
        if (!match) return next();
        try {
          const { getMarkdownPage, getLLMText } =
            await server.ssrLoadModule("/app/lib/source.ts");
          const page = getMarkdownPage(match[1]);
          if (!page) {
            res.statusCode = 404;
            res.end("not found");
            return;
          }
          res.setHeader("Content-Type", "text/markdown; charset=utf-8");
          res.end(await getLLMText(page));
        } catch (err) {
          next(err as Error);
        }
      });
    },
  };
}

export default defineConfig({
  plugins: [
    restartOnDocsRouteChanges(),
    docsMarkdownDevServer(),
    mdx(),
    tailwindcss(),
    reactRouter(),
  ],
  optimizeDeps: {
    include: [
      "@monaco-editor/react",
      "@radix-ui/react-checkbox",
      "@radix-ui/react-label",
      "@radix-ui/react-select",
      "@radix-ui/react-separator",
      "@radix-ui/react-slot",
      "class-variance-authority",
      "clsx",
      "lucide-react",
      "next-themes",
      "react",
      "react-dom",
      "react-dom/client",
      "react-hook-form",
      "react-router",
      "react-router/dom",
      "react/jsx-dev-runtime",
      "react/jsx-runtime",
      "tailwind-merge",
    ],
  },
  resolve: {
    tsconfigPaths: true,
  },
});
