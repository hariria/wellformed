"use client";

import { Button } from "@/components/ui/button";
import type { Preset } from "./presets";

interface PresetSelectorProps {
  presets: Preset[];
  activeIndex: number;
  onSelect: (index: number) => void;
}

export function PresetSelector({
  presets,
  activeIndex,
  onSelect,
}: PresetSelectorProps) {
  return (
    <div className="flex items-center gap-2 flex-wrap">
      {presets.map((preset, i) => (
        <Button
          key={preset.name}
          variant={i === activeIndex ? "default" : "outline"}
          size="sm"
          onClick={() => onSelect(i)}
        >
          {preset.name}
        </Button>
      ))}
    </div>
  );
}
