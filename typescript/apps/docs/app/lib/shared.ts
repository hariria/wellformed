export const docsRoute = "/docs";

// Internal route that renders a page as raw markdown. At build time the
// prerendered output is relocated to `/docs/<slug>.md` (see scripts/relocate-md.mjs)
// and a dev-server middleware serves the same URLs during `pnpm dev`.
export const markdownGenRoute = "/_docs-md";

export const siteUrl = "https://wellformed.net";

export const gitConfig = {
  user: "hariria",
  repo: "wellformed",
  branch: "main",
  // Path to the docs content within the repo, for "edit on GitHub" links.
  contentDir: "typescript/apps/docs/content/docs",
};
