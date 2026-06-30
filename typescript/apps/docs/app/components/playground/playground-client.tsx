"use client";

import { useCallback, useMemo, useState } from "react";
import type { Schema } from "wellformed-ts/ir";
import { Button } from "@/components/ui/button";
import { CodePreview } from "./code-preview";
import { FormPreview } from "./form-preview";
import { PresetSelector } from "./preset-selector";
import { presets } from "./presets";
import { SchemaEditor } from "./schema-editor";

function parseSchema(json: string): {
  schema: Schema | null;
  error: string | null;
} {
  try {
    const parsed = JSON.parse(json);
    if (!parsed.version || !parsed.root) {
      return {
        schema: null,
        error: "Schema must have 'version' and 'root' fields",
      };
    }
    return { schema: parsed as Schema, error: null };
  } catch (e) {
    return { schema: null, error: (e as Error).message };
  }
}

// Resolve the starting preset from a `?preset=<id>` deep link (used by the
// "Run in Playground" buttons in the docs), falling back to the first preset.
function initialPresetIndex(): number {
  if (typeof window === "undefined") return 0;
  const id = new URLSearchParams(window.location.search).get("preset");
  if (!id) return 0;
  const idx = presets.findIndex((p) => p.schema.id === id);
  return idx >= 0 ? idx : 0;
}

export function PlaygroundClient() {
  const start = initialPresetIndex();
  const [activePreset, setActivePreset] = useState(start);
  const [activeTab, setActiveTab] = useState<"preview" | "code">("preview");
  const [jsonText, setJsonText] = useState(() =>
    JSON.stringify(presets[start]?.schema ?? null, null, 2),
  );

  const { schema, error: parseError } = useMemo(
    () => parseSchema(jsonText),
    [jsonText],
  );

  const handlePresetSelect = useCallback((index: number) => {
    const preset = presets[index];
    if (!preset) return;

    setActivePreset(index);
    setJsonText(JSON.stringify(preset.schema, null, 2));
  }, []);

  const handleEditorChange = useCallback((value: string) => {
    setJsonText(value);
    setActivePreset(-1);
  }, []);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Preset selector bar */}
      <div className="shrink-0 border-b px-4 py-3 bg-background">
        <PresetSelector
          presets={presets}
          activeIndex={activePreset}
          onSelect={handlePresetSelect}
        />
        {activePreset >= 0 && presets[activePreset] && (
          <p className="text-xs text-muted-foreground mt-1.5">
            {presets[activePreset].description}
          </p>
        )}
      </div>

      {/* Two-panel layout (stacks on mobile) */}
      <div className="flex flex-col md:flex-row flex-1 min-h-0">
        {/* Left panel: Schema editor */}
        <div className="h-[40vh] md:h-auto md:w-1/2 border-b md:border-b-0 md:border-r flex flex-col min-h-0">
          <div className="shrink-0 px-4 py-2 border-b bg-muted/50">
            <h2 className="text-sm font-medium">Schema (JSON IR)</h2>
          </div>
          <div className="flex-1 min-h-0">
            <SchemaEditor value={jsonText} onChange={handleEditorChange} />
          </div>
        </div>

        {/* Right panel: Preview / Code tabs */}
        <div className="flex-1 md:w-1/2 flex flex-col min-h-0">
          <div className="shrink-0 px-4 py-2 border-b bg-muted/50 flex items-center gap-1">
            <Button
              variant={activeTab === "preview" ? "default" : "ghost"}
              size="sm"
              onClick={() => setActiveTab("preview")}
            >
              Preview
            </Button>
            <Button
              variant={activeTab === "code" ? "default" : "ghost"}
              size="sm"
              onClick={() => setActiveTab("code")}
            >
              Code
            </Button>
          </div>
          <div className="flex-1 min-h-0 overflow-hidden">
            {activeTab === "preview" ? (
              <div className="h-full overflow-y-auto p-4">
                <FormPreview schema={schema} parseError={parseError} />
              </div>
            ) : (
              <CodePreview schema={schema} parseError={parseError} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
