import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { BookOpen, Github, Play } from "lucide-react";
import { gitConfig } from "./shared";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span className="inline-flex items-center gap-2 font-bold text-xl tracking-tight">
          <img
            src="/favicon.svg"
            alt=""
            aria-hidden="true"
            className="size-6 rounded-md"
          />
          <span>wellformed</span>
        </span>
      ),
    },
    links: [
      {
        text: "Docs",
        url: "/docs",
        icon: <BookOpen className="size-4" />,
      },
      {
        text: "Playground",
        url: "/playground",
        icon: <Play className="size-4" />,
      },
      {
        type: "icon",
        url: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
        text: "GitHub",
        icon: <Github className="size-4" />,
        external: true,
      },
    ],
  };
}
