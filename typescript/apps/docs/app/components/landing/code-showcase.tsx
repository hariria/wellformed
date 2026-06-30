"use client";

import { ArrowRight, Check, Copy } from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/utils";

// --- Minimal, dependency-free tokenizer -------------------------------------
// We control every snippet shown, so a small ordered-rule tokenizer gives
// precise, on-brand highlighting without pulling in a syntax engine.

type Rule = [token: string, re: RegExp];

const RULES: Record<string, Rule[]> = {
  ts: [
    ["comment", /\/\/[^\n]*/y],
    ["string", /"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`/y],
    ["number", /\b\d[\d_.]*\b/y],
    ["keyword", /\b(?:import|from|export|const|let|type|return|as|enum)\b/y],
    [
      "brand",
      /\b(?:w|validate|Infer|toSchema|when|equals|require|email|ssn|ein|phone|min|max|minLen|trim|optional|strict|integer|string|object)\b/y,
    ],
    ["ident", /[A-Za-z_$][\w$]*/y],
    ["space", /\s+/y],
  ],
  json: [
    ["key", /"(?:[^"\\]|\\.)*"(?=\s*:)/y],
    ["string", /"(?:[^"\\]|\\.)*"/y],
    ["number", /-?\b\d[\d_.]*\b/y],
    ["brand", /\b(?:true|false|null)\b/y],
    ["space", /\s+/y],
  ],
  rust: [
    ["comment", /\/\/[^\n]*/y],
    ["string", /r#"[\s\S]*?"#|"(?:[^"\\]|\\.)*"/y],
    ["macro", /\b[a-z_]+!/y],
    ["keyword", /\b(?:use|let|fn|mut|match|assert|pub|struct|impl)\b/y],
    ["brand", /\b(?:validate|Schema|is_valid|from_str|wellformed)\b/y],
    ["number", /\b\d[\d_.]*\b/y],
    ["ident", /[A-Za-z_][\w]*/y],
    ["space", /\s+/y],
  ],
};

const TOKEN_CLASS: Record<string, string> = {
  comment: "text-muted-foreground/60 italic",
  string: "text-muted-foreground",
  key: "text-foreground",
  number: "text-foreground/80",
  keyword: "text-foreground font-medium",
  brand: "text-brand",
  macro: "text-brand",
  ident: "text-foreground/70",
  space: "",
  punct: "text-muted-foreground/80",
};

function tokenize(code: string, lang: string) {
  const rules = RULES[lang] ?? RULES.ts;
  const out: { t: string; v: string }[] = [];
  let i = 0;
  while (i < code.length) {
    let matched = false;
    for (const [t, re] of rules) {
      re.lastIndex = i;
      const m = re.exec(code);
      if (m && m.index === i && m[0].length > 0) {
        out.push({ t, v: m[0] });
        i += m[0].length;
        matched = true;
        break;
      }
    }
    if (!matched) {
      out.push({ t: "punct", v: code[i] });
      i += 1;
    }
  }
  return out;
}

function Highlighted({ code, lang }: { code: string; lang: string }) {
  return (
    <code className="font-mono">
      {tokenize(code, lang).map((tok, idx) => (
        <span
          // biome-ignore lint/suspicious/noArrayIndexKey: static, never reordered
          key={idx}
          className={TOKEN_CLASS[tok.t]}
        >
          {tok.v}
        </span>
      ))}
    </code>
  );
}

// --- Copy button ------------------------------------------------------------

export function CopyButton({
  value,
  className,
  label,
}: {
  value: string;
  className?: string;
  label?: string;
}) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      onClick={() => {
        navigator.clipboard?.writeText(value).then(() => {
          setCopied(true);
          setTimeout(() => setCopied(false), 1600);
        });
      }}
      aria-label={label ?? "Copy to clipboard"}
      className={cn(
        "inline-flex items-center gap-1.5 text-muted-foreground transition-colors hover:text-foreground",
        className,
      )}
    >
      {copied ? (
        <Check className="size-3.5 text-brand" />
      ) : (
        <Copy className="size-3.5" />
      )}
      {label ? <span>{copied ? "Copied" : label}</span> : null}
    </button>
  );
}

// --- The runtime-portability showcase ---------------------------------------

const TABS = [
  {
    id: "ts",
    label: "TypeScript",
    sub: "author",
    code: `import { w, validate, type Infer } from "wellformed-ts";

// Domain validators + cross-field rules, all built in.
const Account = w.object({
  email: w.string().email(),
  ssn:   w.string().ssn().optional(),
  ein:   w.string().ein().optional(),
  type:  w.enum(["individual", "business"] as const),
})
  .when("type").equals("individual").require("ssn")
  .when("type").equals("business").require("ein");

type Account = Infer<typeof Account>;

// Compile to portable IR, then validate.
const result = validate(Account.toSchema("1.0"), input);`,
  },
  {
    id: "json",
    label: "JSON IR",
    sub: "ship",
    code: `{
  "version": "1.0",
  "root": {
    "type": "object",
    "unknown_keys": "strict",
    "properties": {
      "email": {
        "type": "string",
        "constraints": [
          { "pred": { "type": "call", "name": "is_email" } }
        ]
      },
      "ssn": {
        "type": "string",
        "optional": true,
        "constraints": [
          { "pred": { "type": "call", "name": "is_ssn" } }
        ]
      }
    },
    "rules": [
      { "when": { "field": "type", "eq": "individual" }, "require": "ssn" }
    ]
  }
}`,
  },
  {
    id: "rust",
    label: "Rust",
    sub: "validate",
    code: `use wellformed::{validate, Schema};

// Deserialize the SAME JSON IR your TypeScript authored.
let schema: Schema = serde_json::from_str(ir_json)?;

let result = validate(&schema, &input);
assert!(result.is_valid());`,
  },
] as const;

export function CodeShowcase() {
  const [active, setActive] = useState<(typeof TABS)[number]["id"]>("ts");
  const tab = TABS.find((t) => t.id === active) ?? TABS[0];

  return (
    <div className="w-full min-w-0 overflow-hidden rounded-xl border border-border bg-card shadow-2xl shadow-black/[0.07] ring-1 ring-black/[0.02]">
      <div className="flex items-center justify-between gap-3 border-b border-border bg-muted/40 px-3 py-2">
        <div className="inline-flex rounded-lg bg-background/60 p-0.5 font-mono text-xs">
          {TABS.map((t) => (
            <button
              key={t.id}
              type="button"
              onClick={() => setActive(t.id)}
              className={cn(
                "rounded-md px-2.5 py-1 transition-colors",
                active === t.id
                  ? "bg-foreground text-background"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              {t.label}
            </button>
          ))}
        </div>
        <CopyButton
          value={tab.code}
          label="Copy"
          className="font-mono text-xs"
        />
      </div>

      <pre className="h-[22rem] w-full min-w-0 overflow-auto px-5 py-4 text-[12.5px] leading-[1.7] [tab-size:2] sm:h-[24rem]">
        <Highlighted code={tab.code} lang={tab.id} />
      </pre>

      <div className="flex items-center gap-2 border-t border-border bg-muted/30 px-5 py-2.5 font-mono text-[11px] text-muted-foreground">
        {TABS.map((t, i) => (
          <span key={t.id} className="flex items-center gap-2">
            {i > 0 ? (
              <ArrowRight className="size-3 text-muted-foreground/50" />
            ) : null}
            <span
              className={cn(
                "transition-colors",
                active === t.id && "text-brand",
              )}
            >
              {t.sub}
            </span>
          </span>
        ))}
        <span className="ml-auto text-muted-foreground/70">
          one schema · two runtimes
        </span>
      </div>
    </div>
  );
}
