"use client";

import Editor from "@monaco-editor/react";
import { useTheme } from "next-themes";

interface SchemaEditorProps {
  value: string;
  onChange: (value: string) => void;
}

export function SchemaEditor({ value, onChange }: SchemaEditorProps) {
  const { resolvedTheme } = useTheme();

  return (
    <Editor
      height="100%"
      language="json"
      theme={resolvedTheme === "dark" ? "vs-dark" : "vs"}
      value={value}
      onChange={(v) => onChange(v ?? "")}
      options={{
        minimap: { enabled: false },
        fontSize: 13,
        lineNumbers: "on",
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 2,
        wordWrap: "on",
        padding: { top: 12 },
      }}
    />
  );
}
