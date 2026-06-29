import { siteMetadata } from "./shared";

type SeoMetaInput = {
  title: string;
  description: string;
  path?: string;
  image?: string;
  origin?: string;
  type?: "website" | "article";
  noIndex?: boolean;
};

export function getRequestOrigin(request: Request): string {
  if (import.meta.env.PROD) return siteMetadata.url;

  const url = new URL(request.url);
  if (
    url.hostname === "localhost" ||
    url.hostname === "127.0.0.1" ||
    url.hostname === "::1"
  ) {
    return url.origin;
  }

  return siteMetadata.url;
}

export function absoluteUrl(path = "/", origin = siteMetadata.url): string {
  return new URL(path, origin).toString();
}

export function seoMeta({
  title,
  description,
  path = "/",
  image = siteMetadata.ogImage,
  origin,
  type = "website",
  noIndex = false,
}: SeoMetaInput) {
  const url = absoluteUrl(path, origin);
  const imageUrl = absoluteUrl(image, origin);

  return [
    { title },
    { name: "description", content: description },
    { name: "robots", content: noIndex ? "noindex, nofollow" : "index, follow" },
    { property: "og:type", content: type },
    { property: "og:site_name", content: siteMetadata.name },
    { property: "og:title", content: title },
    { property: "og:description", content: description },
    { property: "og:url", content: url },
    { property: "og:image", content: imageUrl },
    { property: "og:image:width", content: "1200" },
    { property: "og:image:height", content: "630" },
    { property: "og:image:alt", content: siteMetadata.ogImageAlt },
    { name: "twitter:card", content: "summary_large_image" },
    { name: "twitter:title", content: title },
    { name: "twitter:description", content: description },
    { name: "twitter:image", content: imageUrl },
  ];
}
