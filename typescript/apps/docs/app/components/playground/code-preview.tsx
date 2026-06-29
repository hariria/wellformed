"use client";

import Editor from "@monaco-editor/react";
import { useTheme } from "next-themes";
import { useMemo, useState } from "react";
import type { Schema } from "wellformed-ts/ir";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { generateCode, generateRustCode } from "./generate-code";

interface CodePreviewProps {
  schema: Schema | null;
  parseError: string | null;
}

export function CodePreview({ schema, parseError }: CodePreviewProps) {
  const { resolvedTheme } = useTheme();
  const [language, setLanguage] = useState<"typescript" | "rust">("typescript");

  const typescriptCode = useMemo(() => {
    if (!schema) return "";
    return generateCode(schema);
  }, [schema]);
  const rustCode = useMemo(() => {
    if (!schema) return "";
    return generateRustCode(schema);
  }, [schema]);

  const code = language === "typescript" ? typescriptCode : rustCode;

  if (parseError) {
    return (
      <Card className="border-destructive/50">
        <CardHeader>
          <CardTitle className="text-destructive">Schema Error</CardTitle>
          <CardDescription>
            Fix the JSON to see the generated code
          </CardDescription>
        </CardHeader>
        <CardContent>
          <pre className="text-sm text-destructive whitespace-pre-wrap break-words font-mono">
            {parseError}
          </pre>
        </CardContent>
      </Card>
    );
  }

  if (!schema) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>No Schema</CardTitle>
          <CardDescription>
            Select a preset or enter a valid schema JSON to see generated code.
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  return (
    <div className="h-full flex flex-col min-h-0">
      <div className="shrink-0 border-b px-4 py-2 bg-muted/50 flex items-center gap-1">
        <Button
          variant={language === "typescript" ? "default" : "ghost"}
          size="sm"
          onClick={() => setLanguage("typescript")}
        >
          TypeScript
        </Button>
        <Button
          variant={language === "rust" ? "default" : "ghost"}
          size="sm"
          onClick={() => setLanguage("rust")}
        >
          Rust
        </Button>
      </div>
      <div className="flex-1 min-h-0">
        <Editor
          height="100%"
          language={language === "typescript" ? "typescript" : "rust"}
          theme={resolvedTheme === "dark" ? "vs-dark" : "vs"}
          value={code}
          options={{
            readOnly: true,
            minimap: { enabled: false },
            fontSize: 13,
            lineNumbers: "on",
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 2,
            wordWrap: "on",
            padding: { top: 12 },
            domReadOnly: true,
          }}
        />
      </div>
    </div>
  );
}
