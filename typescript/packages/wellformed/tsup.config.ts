import { defineConfig } from "tsup";

export default defineConfig({
  entry: {
    index: "src/index.ts",
    "builder/index": "src/builder/index.ts",
    "forms/index": "src/forms/index.ts",
    "ir/index": "src/ir/index.ts",
    "runtime/index": "src/runtime/index.ts",
    serialize: "src/serialize.ts",
  },
  format: ["cjs", "esm"],
  dts: true,
  splitting: true,
  sourcemap: false,
  clean: true,
  treeshake: true,
  minify: true,
});
