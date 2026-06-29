import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
const nodeCommand = process.execPath;
const binSuffix = process.platform === "win32" ? ".cmd" : "";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(scriptDir, "..");
const workspaceDir = path.resolve(packageDir, "../..");
const tempDir = mkdtempSync(path.join(tmpdir(), "wellformed-pack-smoke-"));
const consumerDir = path.join(tempDir, "consumer");

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
  });

  if (result.status !== 0) {
    const suffix = options.capture
      ? `\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`
      : "";
    throw new Error(`Command failed: ${command} ${args.join(" ")}${suffix}`);
  }

  return result.stdout;
}

function parseNpmPackJson(output) {
  const start = output.indexOf("[");
  const end = output.lastIndexOf("]");
  if (start === -1 || end === -1 || end < start) {
    throw new Error(`Could not parse npm pack JSON:\n${output}`);
  }
  const parsed = JSON.parse(output.slice(start, end + 1));
  const first = parsed[0];
  if (!first?.filename) {
    throw new Error(`npm pack did not return a filename:\n${output}`);
  }
  return first.filename;
}

function resolveTscBin() {
  const candidates = [
    path.join(packageDir, "node_modules", ".bin", `tsc${binSuffix}`),
    path.join(workspaceDir, "node_modules", ".bin", `tsc${binSuffix}`),
  ];

  const tscBin = candidates.find((candidate) => existsSync(candidate));
  if (!tscBin) {
    throw new Error(
      `TypeScript compiler not found. Checked:\n${candidates.join("\n")}`,
    );
  }
  return tscBin;
}

try {
  mkdirSync(consumerDir, { recursive: true });

  const packOutput = run(
    npmCommand,
    ["pack", packageDir, "--json", "--dry-run=false"],
    {
      cwd: tempDir,
      capture: true,
    },
  );
  const tarball = path.join(tempDir, parseNpmPackJson(packOutput));

  writeFileSync(
    path.join(consumerDir, "package.json"),
    JSON.stringify(
      {
        private: true,
        type: "module",
      },
      null,
      2,
    ),
  );

  run(
    npmCommand,
    [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--dry-run=false",
      tarball,
    ],
    {
      cwd: consumerDir,
    },
  );

  const installedPackageDir = path.join(
    consumerDir,
    "node_modules",
    "wellformed-ts",
  );
  const expectedPackageFiles = [
    "README.md",
    "LICENSE",
    "skills.md",
    path.join("dist", "index.js"),
    path.join("dist", "runtime", "index.js"),
  ];
  for (const relativePath of expectedPackageFiles) {
    const installedPath = path.join(installedPackageDir, relativePath);
    if (!existsSync(installedPath)) {
      throw new Error(`Packed package is missing ${relativePath}`);
    }
  }

  writeFileSync(
    path.join(consumerDir, "esm.mjs"),
    `
import { validate, w, schemaToJSON } from "wellformed-ts";
import { w as builderW } from "wellformed-ts/builder";
import { validate as validateRuntime } from "wellformed-ts/runtime";
import { parseSchema } from "wellformed-ts/serialize";
import * as forms from "wellformed-ts/forms";
import * as ir from "wellformed-ts/ir";

const builder = w.object({
  name: w.string().trim().minLen(1),
  age: w.integer().min(0).optional(),
});
const schema = builder.toSchema("1.0");
const parsed = parseSchema(schemaToJSON(schema));
const result = validate(parsed, { name: "  Ada  ", age: 36 });
if (!result.valid) throw new Error("ESM validation failed");
if (result.value.name !== "Ada") throw new Error("ESM transform failed");
if (!validateRuntime(schema, { name: "Grace" }).valid) {
  throw new Error("ESM runtime subpath failed");
}
const refSchema = parseSchema(JSON.stringify({
  version: "1.0",
  definitions: {
    Name: {
      type: "string",
      constraints: [{
        pred: { type: "min_len", len: 1 },
        error: { code: "REQUIRED", message: "Name is required" }
      }]
    }
  },
  root: { type: "ref", $ref: "Name" }
}));
if (!validate(refSchema, "Ada").valid) {
  throw new Error("ESM ref schema validation failed");
}
const invalidRef = validate(refSchema, "");
if (invalidRef.valid || invalidRef.errors[0]?.code !== "REQUIRED") {
  throw new Error("ESM ref schema error propagation failed");
}
if (typeof builderW.string !== "function") {
  throw new Error("ESM builder subpath failed");
}
if (typeof forms !== "object" || typeof ir !== "object") {
  throw new Error("ESM forms/ir subpath failed");
}
`,
  );
  run(nodeCommand, ["esm.mjs"], { cwd: consumerDir });

  writeFileSync(
    path.join(consumerDir, "cjs.cjs"),
    `
const wellformed = require("wellformed-ts");
const builder = require("wellformed-ts/builder");
const runtime = require("wellformed-ts/runtime");
const serialize = require("wellformed-ts/serialize");
const forms = require("wellformed-ts/forms");
const ir = require("wellformed-ts/ir");

const schema = wellformed.w.object({
  name: wellformed.w.string().trim().minLen(1),
}).toSchema("1.0");
const parsed = serialize.parseSchema(serialize.schemaToJSON(schema));
const result = runtime.validate(parsed, { name: "  Ada  " });
if (!result.valid) throw new Error("CJS validation failed");
if (result.value.name !== "Ada") throw new Error("CJS transform failed");
const refSchema = serialize.parseSchema(JSON.stringify({
  version: "1.0",
  definitions: {
    Name: {
      type: "string",
      constraints: [{
        pred: { type: "min_len", len: 1 },
        error: { code: "REQUIRED", message: "Name is required" }
      }]
    }
  },
  root: { type: "ref", $ref: "Name" }
}));
if (!runtime.validate(refSchema, "Ada").valid) {
  throw new Error("CJS ref schema validation failed");
}
const invalidRef = runtime.validate(refSchema, "");
if (invalidRef.valid || invalidRef.errors[0]?.code !== "REQUIRED") {
  throw new Error("CJS ref schema error propagation failed");
}
if (typeof builder.w.string !== "function") {
  throw new Error("CJS builder subpath failed");
}
if (typeof forms !== "object" || typeof ir !== "object") {
  throw new Error("CJS forms/ir subpath failed");
}
`,
  );
  run(nodeCommand, ["cjs.cjs"], { cwd: consumerDir });

  writeFileSync(
    path.join(consumerDir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          target: "ES2022",
          module: "NodeNext",
          moduleResolution: "NodeNext",
          strict: true,
          noEmit: true,
          skipLibCheck: false,
        },
        include: ["consumer.ts"],
      },
      null,
      2,
    ),
  );
  writeFileSync(
    path.join(consumerDir, "consumer.ts"),
    `
import { parseSchema, validate, w, type Infer, type Schema } from "wellformed-ts";
import { w as builderW } from "wellformed-ts/builder";
import type { TypeSchema } from "wellformed-ts/ir";
import { validate as validateRuntime, type ValidationResult } from "wellformed-ts/runtime";
import "wellformed-ts/forms";
import "wellformed-ts/serialize";

const schemaBuilder = w.object({
  name: w.string().trim().minLen(1),
  age: w.integer().min(0).optional(),
  active: builderW.boolean(),
});

type User = Infer<typeof schemaBuilder>;
const value: User = { name: "Ada", active: true };
const schema: Schema = schemaBuilder.toSchema("1.0");
const root: TypeSchema = schema.root;
const parsed = parseSchema(JSON.stringify(schema));
const result: ValidationResult = validate(parsed, value);
validateRuntime({ version: "1.0", root }, value);
const refSchema: Schema = parseSchema(JSON.stringify({
  version: "1.0",
  definitions: {
    Name: {
      type: "string",
      constraints: [{
        pred: { type: "min_len", len: 1 },
        error: { code: "REQUIRED", message: "Name is required" }
      }]
    }
  },
  root: { type: "ref", $ref: "Name" }
}));
const refResult: ValidationResult = validate(refSchema, "Ada");

if (!result.valid || !refResult.valid) {
  throw new Error("type smoke validation failed");
}
`,
  );

  run(resolveTscBin(), ["--project", "tsconfig.json"], { cwd: consumerDir });
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}
