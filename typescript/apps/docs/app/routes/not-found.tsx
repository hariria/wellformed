import { HomeLayout } from "fumadocs-ui/layouts/home";
import { DefaultNotFound } from "fumadocs-ui/layouts/home/not-found";
import { baseOptions } from "@/lib/layout.shared";
import { seoMeta } from "@/lib/seo";
import type { Route } from "./+types/not-found";

export function meta(_: Route.MetaArgs) {
  return seoMeta({
    title: "Not Found | wellformed",
    description: "The requested wellformed page could not be found.",
    noIndex: true,
  });
}

export default function NotFound() {
  return (
    <HomeLayout {...baseOptions()}>
      <DefaultNotFound />
    </HomeLayout>
  );
}
