import { getLLMText, getMarkdownPage } from "@/lib/source";
import type { Route } from "./+types/mdx";

// Renders a single page as raw markdown. Registered at `_docs-md/*`; the build
// relocates the prerendered output to `/docs/<slug>.md`.
export async function loader({ params }: Route.LoaderArgs) {
  const page = getMarkdownPage(params["*"]);
  if (!page) {
    return new Response("not found", { status: 404 });
  }

  return new Response(await getLLMText(page), {
    headers: {
      "Content-Type": "text/markdown; charset=utf-8",
    },
  });
}
