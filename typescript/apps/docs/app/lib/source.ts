import { docs } from "collections/server";
import { loader } from "fumadocs-core/source";
import { docsRoute } from "./shared";

export const source = loader({
  baseUrl: docsRoute,
  source: docs.toFumadocsSource(),
});

/** Public URL of a page's markdown twin: `/docs/<slug>.md` (index -> `/docs/index.md`). */
export function getPageMarkdownUrl(
  page: (typeof source)["$inferPage"],
): string {
  const slug = page.slugs.join("/");
  return slug ? `/docs/${slug}.md` : "/docs/index.md";
}

/**
 * Resolve a page from the splat of the markdown route or the `.md` dev URL,
 * normalizing the index page (`index` -> root slug `[]`).
 */
export function getMarkdownPage(splat: string) {
  let slugs = splat.split("/").filter((v) => v.length > 0);
  if (slugs.length === 1 && slugs[0] === "index") slugs = [];
  return source.getPage(slugs);
}

/** Render a single page as LLM-friendly markdown with a title heading. */
export async function getLLMText(
  page: (typeof source)["$inferPage"],
): Promise<string> {
  const processed = await page.data.getText("processed");

  return `# ${page.data.title} (${page.url})

${processed}`;
}
