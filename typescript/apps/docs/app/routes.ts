import { index, type RouteConfig, route } from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),
  route("docs/*", "routes/docs.tsx"),
  route("playground", "routes/playground.tsx"),

  // Static search index (Orama, computed in-browser).
  route("api/search", "routes/search.ts"),

  // AI / LLM integration.
  route("llms-full.txt", "llms/full.ts"),
  // Generates per-page markdown; build relocates output to `/docs/<slug>.md`.
  route("_docs-md/*", "llms/mdx.ts"),

  route("*", "routes/not-found.tsx"),
] satisfies RouteConfig;
