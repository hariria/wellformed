import { HomeLayout } from "fumadocs-ui/layouts/home";
import {
  AlertCircle,
  ArrowRight,
  Blocks,
  Bot,
  Boxes,
  Braces,
  Check,
  Cpu,
  FileJson,
  FileText,
  Fingerprint,
  GitBranch,
  Github,
  MonitorSmartphone,
  Play,
  Server,
  Wand2,
} from "lucide-react";
import { Link } from "react-router";
import { CodeShowcase, CopyButton } from "@/components/landing/code-showcase";
import { baseOptions } from "@/lib/layout.shared";
import { absoluteUrl, getRequestOrigin } from "@/lib/seo";
import { gitConfig, siteMetadata } from "@/lib/shared";
import type { Route } from "./+types/home";

const HOME_TITLE = "wellformed: Validation logic, as data";
const HOME_DESCRIPTION =
  "Author validation schemas in TypeScript, compile them to a portable JSON IR, and run the exact same rules in TypeScript and Rust. 60+ domain validators, cross-field rules, transforms, and full type inference.";

export function loader({ request }: Route.LoaderArgs) {
  return { origin: getRequestOrigin(request) };
}

const gridStyle: React.CSSProperties = {
  backgroundImage:
    "linear-gradient(to right, var(--color-border) 1px, transparent 1px), linear-gradient(to bottom, var(--color-border) 1px, transparent 1px)",
  backgroundSize: "56px 56px",
  maskImage:
    "radial-gradient(ellipse 80% 60% at 50% 0%, black, transparent 78%)",
  WebkitMaskImage:
    "radial-gradient(ellipse 80% 60% at 50% 0%, black, transparent 78%)",
};

const FEATURES = [
  {
    icon: Boxes,
    title: "Portable IR",
    detail: "schemaToJSON(schema)",
    body: "Author in TypeScript, ship as JSON. Store it, diff it in code review, version it like any other artifact.",
  },
  {
    icon: Cpu,
    title: "TypeScript + Rust",
    detail: "validate(&schema, &input)",
    body: "One schema, two first-class native runtimes. The same rules produce the same results on both sides of the wire.",
  },
  {
    icon: Fingerprint,
    title: "60+ domain validators",
    detail: "is_ssn · is_ein · is_cusip · is_icd10",
    body: "TINs, financial identifiers, healthcare and aviation codes, contact fields. Built in. No regex archaeology.",
  },
  {
    icon: GitBranch,
    title: "Cross-field rules",
    detail: '.when("type").equals("business")',
    body: "Conditional requirements, mutual exclusion, field comparisons. Real business logic, expressed as data.",
  },
  {
    icon: Wand2,
    title: "Transform pipeline",
    detail: "trim · digits_only · money→cents",
    body: "Normalize before you validate. Transforms travel inside the schema, so every runtime cleans data identically.",
  },
  {
    icon: Braces,
    title: "Full type inference",
    detail: "type T = Infer<typeof schema>",
    body: "Exact TypeScript types straight from the builder. Optional-aware, enum-aware, zero duplicate definitions.",
  },
];

const SURFACES = [
  {
    icon: Blocks,
    label: "Form builders",
    body: "Users and tenants define fields at runtime. The rules live with the form, not hardcoded in your app.",
  },
  {
    icon: MonitorSmartphone,
    label: "Web & mobile",
    body: "Inline field errors as the user types, straight from the stored schema.",
  },
  {
    icon: FileText,
    label: "PDFs & documents",
    body: "Validate a filled document's fields before you accept or render it.",
  },
  {
    icon: Server,
    label: "Any backend",
    body: "Validate submitted JSON in Node, Rust, or a worker before you trust the shape.",
  },
  {
    icon: Bot,
    label: "AI agents",
    body: "Generate and inspect rules as data, not opaque closures.",
  },
];

export default function Home({ loaderData }: Route.ComponentProps) {
  const canonicalUrl = absoluteUrl("/", loaderData.origin);
  const imageUrl = absoluteUrl(siteMetadata.ogImage, loaderData.origin);

  return (
    <HomeLayout {...baseOptions()}>
      <title>{HOME_TITLE}</title>
      <meta name="description" content={HOME_DESCRIPTION} />
      <link rel="canonical" href={canonicalUrl} />
      <meta name="robots" content="index, follow" />
      <meta property="og:type" content="website" />
      <meta property="og:site_name" content={siteMetadata.name} />
      <meta property="og:title" content={HOME_TITLE} />
      <meta property="og:description" content={HOME_DESCRIPTION} />
      <meta property="og:url" content={canonicalUrl} />
      <meta property="og:image" content={imageUrl} />
      <meta property="og:image:width" content="1200" />
      <meta property="og:image:height" content="630" />
      <meta property="og:image:alt" content={siteMetadata.ogImageAlt} />
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:title" content={HOME_TITLE} />
      <meta name="twitter:description" content={HOME_DESCRIPTION} />
      <meta name="twitter:image" content={imageUrl} />
      <div className="flex flex-1 flex-col">
        {/* ── Hero ─────────────────────────────────────────────── */}
        <section className="relative overflow-hidden border-b border-border">
          <div
            aria-hidden
            className="pointer-events-none absolute inset-0"
            style={gridStyle}
          />
          <div
            aria-hidden
            className="pointer-events-none absolute -top-32 right-[-10%] size-[520px] rounded-full bg-brand/20 blur-[130px]"
          />
          <div className="relative mx-auto grid max-w-6xl gap-12 px-6 py-20 lg:grid-cols-[1.05fr_1fr] lg:items-center lg:py-28">
            <div>
              <div className="wf-fade-up mb-5 inline-flex items-center gap-2 font-mono text-[11px] uppercase tracking-[0.2em] text-muted-foreground">
                <span className="size-1.5 rounded-full bg-brand" />
                TypeScript → JSON IR → Rust
              </div>

              <h1
                className="wf-fade-up font-display text-5xl leading-[1.02] tracking-tight text-foreground sm:text-6xl lg:text-7xl"
                style={{ animationDelay: "0.05s" }}
              >
                Validation logic, as{" "}
                <span className="italic text-brand">data</span>.
              </h1>

              <p
                className="wf-fade-up mt-6 max-w-xl text-lg leading-relaxed text-muted-foreground"
                style={{ animationDelay: "0.1s" }}
              >
                Write the schema once. It compiles to JSON, then validates{" "}
                <span className="text-foreground">
                  identically in JavaScript, TypeScript, and Rust
                </span>
                : same transforms, same rules, same error codes. One source of
                truth, no drift across runtimes.
              </p>

              <div
                className="wf-fade-up mt-8 flex flex-wrap items-center gap-3"
                style={{ animationDelay: "0.15s" }}
              >
                <Link
                  to="/playground"
                  className="group inline-flex items-center gap-2 rounded-full bg-brand px-5 py-2.5 text-sm font-medium text-brand-foreground transition-transform hover:-translate-y-0.5"
                >
                  <Play className="size-4" />
                  Open the Playground
                  <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
                </Link>
                <Link
                  to="/docs"
                  className="inline-flex items-center gap-2 rounded-full border border-border px-5 py-2.5 text-sm font-medium text-foreground transition-colors hover:bg-muted"
                >
                  Read the docs
                </Link>
              </div>

              <div
                className="wf-fade-up mt-7 flex flex-wrap items-center gap-3"
                style={{ animationDelay: "0.2s" }}
              >
                <div className="inline-flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-2 font-mono text-sm">
                  <span className="text-muted-foreground/50">$</span>
                  <span className="text-foreground">npm i wellformed-ts</span>
                  <CopyButton value="npm i wellformed-ts" className="ml-1" />
                </div>
                <p className="font-mono text-[11px] uppercase tracking-wider text-muted-foreground/70">
                  MIT · TS &amp; Rust · 60+ validators
                </p>
              </div>
            </div>

            <div
              className="wf-fade-up min-w-0"
              style={{ animationDelay: "0.18s" }}
            >
              <CodeShowcase />
            </div>
          </div>
        </section>

        {/* ── Surfaces (where the rules travel) ─────────────────── */}
        <section className="border-b border-border">
          <div className="mx-auto max-w-6xl px-6 py-20">
            <p className="font-mono text-[11px] uppercase tracking-[0.2em] text-brand">
              Where the rules travel
            </p>
            <h2 className="mt-3 max-w-3xl font-display text-3xl tracking-tight text-foreground sm:text-4xl">
              Your form isn't always a web page.
            </h2>
            <div className="mt-5 max-w-2xl space-y-4 leading-relaxed text-muted-foreground">
              <p>
                Often the form isn't fixed in your code. It's built in a form
                builder, configured per customer, or one of hundreds of forms
                that each carry their own rules. And whatever renders or
                receives it, the browser, the backend, a Rust service, needs
                those same rules.
              </p>
              <p>
                Most validation libraries write those rules as code, and{" "}
                <span className="text-foreground">code is not data</span>. You
                can't store a closure in a database, load the rules for one
                customer's form, or hand them to a Rust service. So every form
                gets hardcoded and every rule rewritten by hand, and the copies
                drift out of sync.
              </p>
              <p>
                wellformed compiles the rules to JSON. Store them with each
                form, load them at runtime, and run them unchanged in TypeScript
                and Rust.
              </p>
            </div>

            <div className="mt-12 grid gap-px overflow-hidden rounded-xl border border-border bg-border sm:grid-cols-2 lg:grid-cols-5">
              {SURFACES.map((s, i) => (
                <div key={s.label} className="relative bg-card p-5">
                  <s.icon className="size-5 text-brand" />
                  <h3 className="mt-3 text-sm font-medium text-foreground">
                    {s.label}
                  </h3>
                  <p className="mt-1.5 text-[13px] leading-relaxed text-muted-foreground">
                    {s.body}
                  </p>
                  {i < SURFACES.length - 1 ? (
                    <ArrowRight className="absolute right-[-9px] top-1/2 hidden size-4 -translate-y-1/2 text-muted-foreground/40 lg:block" />
                  ) : null}
                </div>
              ))}
            </div>

            <div className="mt-6 flex flex-wrap items-center gap-x-6 gap-y-2">
              <p className="font-mono text-[11px] uppercase tracking-wider text-muted-foreground/70">
                One schema · five surfaces · zero reimplementation
              </p>
              <Link
                to="/docs/why-wellformed"
                className="group inline-flex items-center gap-1.5 text-sm font-medium text-foreground hover:text-brand"
              >
                Why wellformed
                <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
              </Link>
            </div>
          </div>
        </section>

        {/* ── How it works ─────────────────────────────────────── */}
        <section className="border-b border-border bg-muted/20">
          <div className="mx-auto grid max-w-6xl gap-12 px-6 py-20 lg:grid-cols-2 lg:items-center lg:py-24">
            <div>
              <p className="font-mono text-[11px] uppercase tracking-[0.2em] text-brand">
                How it works
              </p>
              <h2 className="mt-3 font-display text-3xl tracking-tight text-foreground sm:text-4xl">
                Author once. Compile to data. Run anywhere.
              </h2>
              <p className="mt-5 leading-relaxed text-muted-foreground">
                Write the schema with a Zod-like builder. wellformed compiles it
                to a JSON IR: a complete program of predicates, transforms,
                error codes, and cross-field rules, with nothing hidden in
                closures.
              </p>
              <p className="mt-4 leading-relaxed text-foreground">
                Store that IR, send it over the wire, or load it by id. The{" "}
                <span className="text-brand">TypeScript and Rust runtimes</span>{" "}
                read the same file and validate identically.
              </p>
            </div>

            <BoundaryDiagram />
          </div>
        </section>

        {/* ── Playground card ──────────────────────────────────── */}
        <section className="border-b border-border">
          <div className="mx-auto max-w-6xl px-6 py-20">
            <div className="grid overflow-hidden rounded-2xl border border-border bg-card lg:grid-cols-2">
              <div className="p-8 lg:p-12">
                <p className="font-mono text-[11px] uppercase tracking-[0.2em] text-brand">
                  Try it live
                </p>
                <h2 className="mt-3 font-display text-3xl tracking-tight text-foreground sm:text-4xl">
                  See validation as data, live.
                </h2>
                <p className="mt-5 max-w-md leading-relaxed text-muted-foreground">
                  Edit a schema's IR on the left. A real form validates on the
                  right: predicates, transforms, cross-field rules, error codes,
                  all in your browser. Seeing the wire format do the work
                  converts skeptics faster than any pitch.
                </p>
                <Link
                  to="/playground"
                  className="group mt-8 inline-flex items-center gap-2 rounded-full bg-brand px-5 py-2.5 text-sm font-medium text-brand-foreground transition-transform hover:-translate-y-0.5"
                >
                  <Play className="size-4" />
                  Open the Playground
                  <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
                </Link>
              </div>

              <div className="border-t border-border bg-muted/30 p-8 lg:border-l lg:border-t-0 lg:p-12">
                <FormPreview />
              </div>
            </div>
          </div>
        </section>

        {/* ── Feature grid ─────────────────────────────────────── */}
        <section className="border-b border-border bg-muted/20">
          <div className="mx-auto max-w-6xl px-6 py-20">
            <p className="font-mono text-[11px] uppercase tracking-[0.2em] text-brand">
              Batteries included
            </p>
            <h2 className="mt-3 max-w-2xl font-display text-3xl tracking-tight text-foreground sm:text-4xl">
              Everything you'd otherwise re-implement on both sides.
            </h2>

            <div className="mt-12 grid gap-px overflow-hidden rounded-xl border border-border bg-border sm:grid-cols-2 lg:grid-cols-3">
              {FEATURES.map((f) => (
                <div key={f.title} className="bg-card p-6">
                  <f.icon className="size-5 text-brand" />
                  <h3 className="mt-4 font-medium text-foreground">
                    {f.title}
                  </h3>
                  <p className="mt-1 font-mono text-[11px] text-muted-foreground/80">
                    {f.detail}
                  </p>
                  <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
                    {f.body}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ── Performance ──────────────────────────────────────── */}
        <section className="border-b border-border bg-muted/20">
          <div className="mx-auto max-w-6xl px-6 py-20 text-center">
            <p className="font-mono text-[11px] uppercase tracking-[0.2em] text-brand">
              Performance
            </p>
            <h2 className="mt-3 font-display text-3xl tracking-tight text-foreground sm:text-4xl">
              Purpose-built beats regex.
            </h2>
            <p className="mx-auto mt-4 max-w-2xl leading-relaxed text-muted-foreground">
              Most validators lean on generic regex. wellformed's built-in
              predicates are purpose-built, so an email or SSN checks in tens of
              nanoseconds instead of microseconds. That is 10 to 40x faster than
              Zod in the same V8 runtime, and the Rust runtime goes further with
              SIMD-friendly byte scanners.
            </p>

            <PerfChart />

            <Link
              to="/docs/performance"
              className="group mt-8 inline-flex items-center gap-1.5 text-sm font-medium text-foreground hover:text-brand"
            >
              See the full benchmark
              <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
            </Link>
          </div>
        </section>

        {/* ── Final CTA ────────────────────────────────────────── */}
        <section className="relative overflow-hidden">
          <div
            aria-hidden
            className="pointer-events-none absolute inset-x-0 top-0 mx-auto size-[420px] rounded-full bg-brand/15 blur-[120px]"
          />
          <div className="relative mx-auto max-w-3xl px-6 py-24 text-center">
            <h2 className="font-display text-4xl tracking-tight text-foreground sm:text-5xl">
              Stop maintaining two copies of the truth.
            </h2>
            <p className="mx-auto mt-5 max-w-xl text-lg leading-relaxed text-muted-foreground">
              Define your validation once. Ship it as data. Run it everywhere.
            </p>
            <div className="mt-9 flex flex-wrap items-center justify-center gap-3">
              <Link
                to="/playground"
                className="group inline-flex items-center gap-2 rounded-full bg-brand px-5 py-2.5 text-sm font-medium text-brand-foreground transition-transform hover:-translate-y-0.5"
              >
                <Play className="size-4" />
                Open the Playground
                <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
              </Link>
              <Link
                to="/docs/getting-started"
                className="inline-flex items-center gap-2 rounded-full border border-border px-5 py-2.5 text-sm font-medium text-foreground transition-colors hover:bg-muted"
              >
                Get started
              </Link>
            </div>
            <div className="mt-7 inline-flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-2 font-mono text-sm">
              <span className="text-muted-foreground/50">$</span>
              <span className="text-foreground">npm i wellformed-ts</span>
              <CopyButton value="npm i wellformed-ts" className="ml-1" />
            </div>
          </div>
        </section>

        {/* ── Footer ───────────────────────────────────────────── */}
        <footer className="border-t border-border">
          <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 px-6 py-10 sm:flex-row">
            <span className="font-display text-xl text-foreground">
              wellformed
            </span>
            <nav className="flex items-center gap-6 text-sm text-muted-foreground">
              <Link to="/docs" className="hover:text-foreground">
                Docs
              </Link>
              <Link to="/playground" className="hover:text-foreground">
                Playground
              </Link>
              <a
                href={`https://github.com/${gitConfig.user}/${gitConfig.repo}`}
                className="inline-flex items-center gap-1.5 hover:text-foreground"
              >
                <Github className="size-4" />
                GitHub
              </a>
            </nav>
            <span className="font-mono text-xs text-muted-foreground/70">
              MIT © 2026
            </span>
          </div>
        </footer>
      </div>
    </HomeLayout>
  );
}

// Visual: one schema flowing across a runtime boundary into TS and Rust.
// Validation latency, wellformed-ts vs Zod, on a shared linear scale so the
// order-of-magnitude gap is visible at a glance. Numbers from /docs/performance.
function PerfChart() {
  const MAX_NS = 4410; // longest bar (Zod, URL)
  const rows = [
    {
      label: "Email",
      wf: 61,
      zod: 2730,
      wfL: "61 ns",
      zodL: "2.73 µs",
      mult: "45x",
    },
    {
      label: "URL",
      wf: 106,
      zod: 4410,
      wfL: "106 ns",
      zodL: "4.41 µs",
      mult: "42x",
    },
    {
      label: "SSN",
      wf: 85,
      zod: 2560,
      wfL: "85 ns",
      zodL: "2.56 µs",
      mult: "30x",
    },
    {
      label: "Object",
      wf: 411,
      zod: 4350,
      wfL: "411 ns",
      zodL: "4.35 µs",
      mult: "11x",
    },
  ];

  return (
    <div className="mx-auto mt-10 max-w-2xl text-left">
      <div className="mb-6 flex items-center justify-end gap-5 font-mono text-[11px] text-muted-foreground">
        <span className="flex items-center gap-1.5">
          <span className="size-2.5 rounded-sm bg-brand" />
          wellformed-ts
        </span>
        <span className="flex items-center gap-1.5">
          <span className="size-2.5 rounded-sm bg-muted-foreground/30" />
          Zod
        </span>
      </div>

      <div className="space-y-5">
        {rows.map((r) => (
          <div key={r.label}>
            <div className="mb-1.5 flex items-baseline justify-between">
              <span className="font-mono text-xs uppercase tracking-wider text-foreground">
                {r.label}
              </span>
              <span className="font-mono text-[11px] text-brand">
                {r.mult} faster
              </span>
            </div>
            <div className="space-y-1.5">
              <div className="flex items-center gap-2">
                <div
                  className="h-2.5 rounded-full bg-brand"
                  style={{ width: `max(0.5rem, ${(r.wf / MAX_NS) * 100}%)` }}
                />
                <span className="font-mono text-[11px] text-foreground">
                  {r.wfL}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <div
                  className="h-2.5 rounded-full bg-muted-foreground/30"
                  style={{ width: `${(r.zod / MAX_NS) * 100}%` }}
                />
                <span className="font-mono text-[11px] text-muted-foreground">
                  {r.zodL}
                </span>
              </div>
            </div>
          </div>
        ))}
      </div>

      <p className="mt-6 text-center font-mono text-[10px] text-muted-foreground/60">
        Validation latency, lower is faster. Same V8 runtime, Apple Silicon.
      </p>
    </div>
  );
}

function BoundaryDiagram() {
  return (
    <div className="rounded-2xl border border-border bg-card p-6 sm:p-8">
      <div className="flex items-center gap-3 rounded-lg border border-border bg-muted/40 px-4 py-3">
        <Braces className="size-4 text-muted-foreground" />
        <span className="font-mono text-xs text-muted-foreground">
          w.object(&#123; … &#125;).when(…).require(…)
        </span>
        <span className="ml-auto font-mono text-[10px] uppercase tracking-wider text-muted-foreground/60">
          author · ts
        </span>
      </div>

      <Connector />

      <div className="flex items-center gap-3 rounded-lg border border-brand/40 bg-brand/10 px-4 py-3">
        <FileJson className="size-4 text-brand" />
        <span className="font-mono text-xs text-foreground">
          account.schema.json
        </span>
        <span className="ml-auto font-mono text-[10px] uppercase tracking-wider text-brand">
          portable ir
        </span>
      </div>

      <div className="my-4 flex items-center gap-3">
        <div className="h-px flex-1 border-t border-dashed border-border" />
        <span className="font-mono text-[10px] uppercase tracking-[0.2em] text-muted-foreground/60">
          runtime boundary
        </span>
        <div className="h-px flex-1 border-t border-dashed border-border" />
      </div>

      <div className="grid grid-cols-2 gap-3">
        {[
          { label: "TypeScript", sub: "wellformed-ts" },
          { label: "Rust", sub: "wellformed" },
        ].map((rt) => (
          <div
            key={rt.label}
            className="rounded-lg border border-border bg-muted/30 px-4 py-3"
          >
            <div className="flex items-center gap-2">
              <Cpu className="size-4 text-muted-foreground" />
              <span className="text-sm font-medium text-foreground">
                {rt.label}
              </span>
            </div>
            <div className="mt-2 flex items-center gap-1.5 font-mono text-[11px] text-brand">
              <Check className="size-3.5" />
              valid
            </div>
            <p className="mt-1 font-mono text-[10px] text-muted-foreground/60">
              {rt.sub}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}

function Connector() {
  return (
    <div className="flex justify-center py-2">
      <div className="h-5 w-px bg-border" />
    </div>
  );
}

// Faux form preview hinting at the live playground behavior.
function FormPreview() {
  const rows = [
    { label: "Email", value: "ada@example.com", state: "ok" as const },
    { label: "SSN", value: "•••-••-1234", state: "ok" as const },
    {
      label: "EIN",
      value: "",
      state: "err" as const,
      note: "required when type is business",
    },
  ];
  return (
    <div className="rounded-xl border border-border bg-card p-5">
      <div className="mb-4 flex items-center justify-between">
        <span className="font-mono text-[11px] uppercase tracking-wider text-muted-foreground">
          live preview
        </span>
        <span className="inline-flex items-center gap-1.5 rounded-full bg-brand/15 px-2 py-0.5 font-mono text-[10px] text-brand">
          <span className="size-1.5 rounded-full bg-brand" />
          validating
        </span>
      </div>
      <div className="space-y-3">
        {rows.map((r) => (
          <div key={r.label}>
            <div className="mb-1 font-mono text-[10px] uppercase tracking-wider text-muted-foreground/70">
              {r.label}
            </div>
            <div
              className={`flex items-center gap-2 rounded-md border px-3 py-2 text-sm ${
                r.state === "ok"
                  ? "border-border bg-background"
                  : "border-destructive/40 bg-destructive/5"
              }`}
            >
              <span
                className={
                  r.value ? "text-foreground" : "text-muted-foreground/40"
                }
              >
                {r.value || "empty"}
              </span>
              {r.state === "ok" ? (
                <Check className="ml-auto size-4 text-brand" />
              ) : (
                <AlertCircle className="ml-auto size-4 text-destructive" />
              )}
            </div>
            {r.note ? (
              <p className="mt-1 font-mono text-[10px] text-destructive/80">
                {r.note}
              </p>
            ) : null}
          </div>
        ))}
      </div>
    </div>
  );
}
