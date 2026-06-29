import { useEffect, useState } from "react";
import { Link } from "react-router";
import { PlaygroundClient } from "@/components/playground/playground-client";
import type { Route } from "./+types/playground";

export function meta(_: Route.MetaArgs) {
  return [
    { title: "Playground | wellformed" },
    {
      name: "description",
      content:
        "Interactive playground for the wellformed validation schema. Edit JSON IR schemas and see live form previews with real-time validation.",
    },
  ];
}

export default function PlaygroundPage() {
  // The editor (Monaco) is client-only; render it after hydration so the
  // prerendered shell stays static.
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);

  return (
    <div className="flex flex-col h-screen overflow-hidden">
      <header className="shrink-0 flex items-center gap-4 border-b px-4 h-12">
        <Link to="/" className="font-bold text-lg tracking-tight">
          wellformed
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
