import browserCollections from "collections/browser";
import { useFumadocsLoader } from "fumadocs-core/source/client";
import { DocsLayout } from "fumadocs-ui/layouts/docs";
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
  MarkdownCopyButton,
  ViewOptionsPopover,
} from "fumadocs-ui/layouts/docs/page";
import { baseOptions } from "@/lib/layout.shared";
import { absoluteUrl, getRequestOrigin } from "@/lib/seo";
import { gitConfig } from "@/lib/shared";
import { getPageMarkdownUrl, source } from "@/lib/source";
import { getMDXComponents } from "@/mdx-components";
import type { Route } from "./+types/docs";

export async function loader({ params, request }: Route.LoaderArgs) {
  const slugs = params["*"].split("/").filter((v) => v.length > 0);
  const page = source.getPage(slugs);
  if (!page) throw new Response("Not found", { status: 404 });

  return {
    origin: getRequestOrigin(request),
    path: page.path,
    url: page.url,
    markdownUrl: getPageMarkdownUrl(page),
    pageTree: await source.serializePageTree(source.getPageTree()),
  };
}

const clientLoader = browserCollections.docs.createClientLoader({
  component(
    { toc, frontmatter, default: Mdx },
    {
      markdownUrl,
      origin,
      path,
      url,
    }: { markdownUrl: string; origin: string; path: string; url: string },
  ) {
    const githubUrl = `https://github.com/${gitConfig.user}/${gitConfig.repo}/blob/${gitConfig.branch}/${gitConfig.contentDir}/${path}`;
    const title = `${frontmatter.title} | wellformed`;
    const description = frontmatter.description;
    const canonicalUrl = absoluteUrl(url, origin);
    const imageUrl = absoluteUrl("/og-image.png", origin);

    return (
      <DocsPage toc={toc}>
        <title>{title}</title>
        <meta name="description" content={description} />
        <link rel="canonical" href={canonicalUrl} />
        <meta name="robots" content="index, follow" />
        <meta property="og:type" content="article" />
        <meta property="og:site_name" content="wellformed" />
        <meta property="og:title" content={title} />
        <meta property="og:description" content={description} />
        <meta property="og:url" content={canonicalUrl} />
        <meta property="og:image" content={imageUrl} />
        <meta property="og:image:width" content="1200" />
        <meta property="og:image:height" content="630" />
        <meta
          property="og:image:alt"
          content="wellformed: validation logic, as portable data."
        />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content={title} />
        <meta name="twitter:description" content={description} />
        <meta name="twitter:image" content={imageUrl} />
        <DocsTitle>{frontmatter.title}</DocsTitle>
        <DocsDescription>{frontmatter.description}</DocsDescription>
        <div className="flex flex-row gap-2 items-center border-b -mt-4 pb-6">
          <MarkdownCopyButton markdownUrl={markdownUrl} />
          <ViewOptionsPopover markdownUrl={markdownUrl} githubUrl={githubUrl} />
        </div>
        <DocsBody>
          <Mdx components={getMDXComponents()} />
        </DocsBody>
      </DocsPage>
    );
  },
});

export default function Page({ loaderData }: Route.ComponentProps) {
  const { path, pageTree, markdownUrl, origin, url } =
    useFumadocsLoader(loaderData);

  return (
    <DocsLayout {...baseOptions()} tree={pageTree}>
      {clientLoader.useContent(path, { markdownUrl, origin, path, url })}
    </DocsLayout>
  );
}
