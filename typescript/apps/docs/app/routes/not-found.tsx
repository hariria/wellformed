import { HomeLayout } from "fumadocs-ui/layouts/home";
import { DefaultNotFound } from "fumadocs-ui/layouts/home/not-found";
import { baseOptions } from "@/lib/layout.shared";
import type { Route } from "./+types/not-found";

export function meta(_: Route.MetaArgs) {
  return [{ title: "Not Found | wellformed" }];
}

export default function NotFound() {
  return (
    <HomeLayout {...baseOptions()}>
      <DefaultNotFound />
    </HomeLayout>
  );
}
