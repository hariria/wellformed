import { ArrowRight, Play } from "lucide-react";
import { Link } from "react-router";

// "Run in Playground" CTA for docs. Deep-links to a preset so a reader goes
// from reading to running in one click.
export function PlaygroundButton({
  preset,
  children,
}: {
  preset?: string;
  children?: React.ReactNode;
}) {
  const href = preset
    ? `/playground?preset=${encodeURIComponent(preset)}`
    : "/playground";

  return (
    <Link
      to={href}
      className="not-prose group my-2 inline-flex items-center gap-2 rounded-full bg-brand px-4 py-2 text-sm font-medium text-brand-foreground no-underline transition-transform hover:-translate-y-0.5"
    >
      <Play className="size-4" />
      {children ?? "Run in Playground"}
      <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
    </Link>
  );
}
