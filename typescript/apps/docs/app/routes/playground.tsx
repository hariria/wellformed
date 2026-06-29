import { useEffect, useState } from "react";
import { Link } from "react-router";
import { PlaygroundClient } from "@/components/playground/playground-client";
import { absoluteUrl, getRequestOrigin } from "@/lib/seo";
import { siteMetadata } from "@/lib/shared";
import type { Route } from "./+types/playground";

const PLAYGROUND_TITLE = "Playground | wellformed";
const PLAYGROUND_DESCRIPTION =
  "Interactive playground for the wellformed validation schema. Edit JSON IR schemas and see live form previews with real-time validation.";

export function loader({ request }: Route.LoaderArgs) {
  return { origin: getRequestOrigin(request) };
}

export default function PlaygroundPage({ loaderData }: Route.ComponentProps) {
  // The editor (Monaco) is client-only; render it after hydration so the
  // prerendered shell stays static.
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  const canonicalUrl = absoluteUrl("/playground", loaderData.origin);
  const imageUrl = absoluteUrl(siteMetadata.ogImage, loaderData.origin);

  return (
    <div className="flex flex-col h-screen overflow-hidden">
      <title>{PLAYGROUND_TITLE}</title>
      <meta name="description" content={PLAYGROUND_DESCRIPTION} />
      <link rel="canonical" href={canonicalUrl} />
      <meta name="robots" content="index, follow" />
      <meta property="og:type" content="website" />
      <meta property="og:site_name" content={siteMetadata.name} />
      <meta property="og:title" content={PLAYGROUND_TITLE} />
      <meta property="og:description" content={PLAYGROUND_DESCRIPTION} />
      <meta property="og:url" content={canonicalUrl} />
      <meta property="og:image" content={imageUrl} />
      <meta property="og:image:width" content="1200" />
      <meta property="og:image:height" content="630" />
      <meta property="og:image:alt" content={siteMetadata.ogImageAlt} />
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:title" content={PLAYGROUND_TITLE} />
      <meta name="twitter:description" content={PLAYGROUND_DESCRIPTION} />
      <meta name="twitter:image" content={imageUrl} />
      <header className="shrink-0 flex items-center gap-4 border-b px-4 h-12">
        <Link
          to="/"
          className="inline-flex items-center gap-2 font-bold text-lg tracking-tight"
        >
          <img
            src="/favicon.svg"
            alt=""
            aria-hidden="true"
            className="size-6 rounded-md"
          />
          <span>wellformed</span>
        </Link>
        <div className="flex-1" />
        <nav className="flex items-center gap-4">
          <Link
            to="/docs"
            className="text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            Docs
          </Link>
          <Link
            to="/playground"
            className="text-sm font-medium hover:text-foreground transition-colors"
          >
            Playground
          </Link>
        </nav>
      </header>
      <main className="flex-1 min-h-0">
        {mounted ? <PlaygroundClient /> : null}
      </main>
    </div>
  );
}
