import { readdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { Config } from "@react-router/dev/config";
import { createGetUrl, getSlugs } from "fumadocs-core/source";

const getDocsUrl = createGetUrl("/docs");
const appDir = dirname(fileURLToPath(import.meta.url));
const docsContentDir = join(appDir, "content/docs");

async function* walkMdxFiles(dir: string, prefix = ""): AsyncGenerator<string> {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const relativePath = prefix ? `${prefix}/${entry.name}` : entry.name;
    const absolutePath = join(dir, entry.name);

    if (entry.isDirectory()) {
      yield* walkMdxFiles(absolutePath, relativePath);
      continue;
    }

    if (entry.isFile() && entry.name.endsWith(".mdx")) {
      yield relativePath;
    }
  }
}

export default {
  // Static SPA build: emits prerendered HTML + assets to build/client,
  // ready to serve directly from Cloudflare Pages (no server runtime).
  ssr: false,
  async prerender({ getStaticPaths }) {
    // Static, non-parameterized routes.
    const paths = new Set<string>([
      "/",
      "/playground",
      "/docs",
      "/llms-full.txt",
      "/api/search",
    ]);
    for (const path of getStaticPaths()) paths.add(path);

    // One entry per docs page: the rendered HTML page and its markdown twin.
    // Twins are generated under /_docs-md/* and relocated to /docs/<slug>.md
    // after the build (scripts/relocate-md.mjs).
    for await (const entry of walkMdxFiles(docsContentDir)) {
      const slugs = getSlugs(entry);
      paths.add(getDocsUrl(slugs));
      paths.add(`/_docs-md/${slugs.length > 0 ? slugs.join("/") : "index"}`);
    }

    return [...paths];
  },
} satisfies Config;
