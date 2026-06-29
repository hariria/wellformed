#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import * as v from "valibot";
import * as yup from "yup";
import * as z from "zod";

import { validate, w } from "../dist/index.js";

const require = createRequire(import.meta.url);
const packageDir = dirname(dirname(fileURLToPath(import.meta.url)));
const args = new Set(process.argv.slice(2));

const iterations = Number(process.env.BENCH_ITERS ?? "100000");
const warmupIterations = Number(process.env.BENCH_WARMUP ?? "10000");
const rounds = Number(process.env.BENCH_ROUNDS ?? "7");

const validators = ["wellformed-ts", "valibot", "zod", "yup"];

const emailInputs = [
  "ada@example.com",
  "user.name+tag@example.org",
  "a@b.co",
  "not-an-email",
  "@example.com",
  "bad@.com",
];

const urlInputs = [
  "https://example.com",
  "http://sub.example.com/path?q=1",
  "https://docs.example.org/benchmarks",
  "ftp://example.com",
  "not a url",
  "https://",
];

const uuidInputs = [
  "550e8400-e29b-41d4-a716-446655440000",
  "6fa459ea-ee8a-3ca4-894e-db77e160355e",
  "00000000-0000-0000-0000-000000000000",
  "550e8400-e29b-41d4-a716-44665544000",
  "not-a-uuid",
  "550e8400e29b41d4a716446655440000",
];

const ssnInputs = [
  "123-45-6789",
  "078-05-1120",
  "219099999",
  "000-12-3456",
  "666-12-3456",
  "987-65-4321",
];

const cardInputs = [
  "4111 1111 1111 1111",
  "5555-5555-5555-4444",
  "378282246310005",
  "4111 1111 1111 1112",
  "1234567890123456",
  "not-a-card",
];

const ibanInputs = [
  "GB29 NWBK 6016 1331 9268 19",
  "DE89370400440532013000",
  "FR7630006000011234567890189",
  "GB29NWBK60161331926818",
  "DE123",
  "not-an-iban",
];

const objectInputs = [
  {
    email: "ada@example.com",
    age: 37,
    website: "https://example.com",
    ssn: "123-45-6789",
  },
  {
    email: "grace@example.org",
    age: 58,
    website: "http://research.example.org/profile",
    ssn: "078-05-1120",
  },
  {
    email: "bad",
    age: 37,
    website: "https://example.com",
    ssn: "123-45-6789",
  },
  {
    email: "ada@example.com",
    age: 12,
    website: "https://example.com",
    ssn: "123-45-6789",
  },
  {
    email: "ada@example.com",
    age: 37,
    website: "ftp://example.com",
    ssn: "123-45-6789",
  },
  {
    email: "ada@example.com",
    age: 37,
    website: "https://example.com",
    ssn: "000-12-3456",
  },
];

const ibanCountryLengths = {
  AL: 28,
  AD: 24,
  AT: 20,
  AZ: 28,
  BH: 22,
  BY: 28,
  BE: 16,
  BA: 20,
  BR: 29,
  BG: 22,
  CR: 22,
  HR: 21,
  CY: 28,
  CZ: 24,
  DK: 18,
  DO: 28,
  EE: 20,
  FO: 18,
  FI: 18,
  FR: 27,
  GE: 22,
  DE: 22,
  GI: 23,
  GR: 27,
  GL: 18,
  GT: 28,
  HU: 28,
  IS: 26,
  IQ: 23,
  IE: 22,
  IL: 23,
  IT: 27,
  JO: 30,
  KZ: 20,
  XK: 20,
  KW: 30,
  LV: 21,
  LB: 28,
  LI: 21,
  LT: 20,
  LU: 20,
  MK: 19,
  MT: 31,
  MR: 27,
  MU: 30,
  MC: 27,
  MD: 24,
  ME: 22,
  NL: 18,
  NO: 15,
  PK: 24,
  PS: 29,
  PL: 28,
  PT: 25,
  QA: 29,
  RO: 24,
  LC: 32,
  SM: 27,
  SA: 24,
  RS: 22,
  SC: 31,
  SK: 24,
  SI: 19,
  ES: 24,
  SE: 24,
  CH: 21,
  TN: 24,
  TR: 26,
  AE: 23,
  GB: 22,
  VA: 22,
  VG: 24,
  UA: 29,
};

const wfSchemas = {
  email: w.string().email().toTypeSchema(),
  url: w.string().url().toTypeSchema(),
  uuid: w.string().uuid().toTypeSchema(),
  ssn: w.string().ssn().toTypeSchema(),
  creditCard: w.string().creditCard().toTypeSchema(),
  iban: w.string().iban().toTypeSchema(),
  object: w
    .object({
      email: w.string().email(),
      age: w.integer().min(18).max(99),
      website: w.string().url(),
      ssn: w.string().ssn(),
    })
    .toTypeSchema(),
};

const zSchemas = {
  email: z.string().email(),
  url: z.string().url().refine(isHttpUrl),
  uuid: z.string().uuid(),
  ssn: z.string().refine(isValidSsn),
  creditCard: z.string().refine(isValidCreditCard),
  iban: z.string().refine(isValidIban),
  object: z.object({
    email: z.string().email(),
    age: z.number().int().min(18).max(99),
    website: z.string().url().refine(isHttpUrl),
    ssn: z.string().refine(isValidSsn),
  }),
};

const vSchemas = {
  email: v.pipe(v.string(), v.email()),
  url: v.pipe(v.string(), v.url(), v.check(isHttpUrl, "Invalid URL")),
  uuid: v.pipe(v.string(), v.uuid()),
  ssn: v.pipe(v.string(), v.check(isValidSsn, "Invalid SSN")),
  creditCard: v.pipe(
    v.string(),
    v.check(isValidCreditCard, "Invalid credit card"),
  ),
  iban: v.pipe(v.string(), v.check(isValidIban, "Invalid IBAN")),
  object: v.object({
    email: v.pipe(v.string(), v.email()),
    age: v.pipe(v.number(), v.integer(), v.minValue(18), v.maxValue(99)),
    website: v.pipe(v.string(), v.url(), v.check(isHttpUrl, "Invalid URL")),
    ssn: v.pipe(v.string(), v.check(isValidSsn, "Invalid SSN")),
  }),
};

const yupSchemas = {
  email: yup.string().required().email(),
  url: yup.string().required().url().test("http-url", "Invalid URL", isHttpUrl),
  uuid: yup.string().required().uuid(),
  ssn: yup.string().required().test("ssn", "Invalid SSN", isValidSsn),
  creditCard: yup
    .string()
    .required()
    .test("credit-card", "Invalid credit card", isValidCreditCard),
  iban: yup.string().required().test("iban", "Invalid IBAN", isValidIban),
  object: yup
    .object({
      email: yup.string().required().email(),
      age: yup.number().required().integer().min(18).max(99),
      website: yup
        .string()
        .required()
        .url()
        .test("http-url", "Invalid URL", isHttpUrl),
      ssn: yup.string().required().test("ssn", "Invalid SSN", isValidSsn),
    })
    .required(),
};

const cases = [
  {
    id: "email",
    label: "Email",
    inputs: emailInputs,
    note: "Built-in email validator in each library.",
  },
  {
    id: "url",
    label: "HTTP URL",
    inputs: urlInputs,
    note: "Built-in URL validator plus an http/https policy.",
  },
  {
    id: "uuid",
    label: "UUID",
    inputs: uuidInputs,
    note: "Built-in UUID validator in each library.",
  },
  {
    id: "ssn",
    label: "SSN",
    inputs: ssnInputs,
    note: "wellformed built-in vs custom semantic predicate in other libraries.",
  },
  {
    id: "creditCard",
    label: "Credit card",
    inputs: cardInputs,
    note: "Luhn plus common network prefix/length checks.",
  },
  {
    id: "iban",
    label: "IBAN",
    inputs: ibanInputs,
    note: "Mod-97 checksum and country length checks.",
  },
  {
    id: "object",
    label: "Object",
    inputs: objectInputs,
    note: "Four-field object: email, age, URL, and SSN.",
  },
];

const runners = {
  "wellformed-ts": (id) => (input) => validate(wfSchemas[id], input).valid,
  valibot: (id) => (input) => v.safeParse(vSchemas[id], input).success,
  zod: (id) => (input) => zSchemas[id].safeParse(input).success,
  yup: (id) => (input) =>
    yupSchemas[id].isValidSync(input, { strict: true, abortEarly: false }),
};

const environment = {
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  iterations,
  warmupIterations,
  rounds,
  versions: {
    "wellformed-ts": readPackageVersion(join(packageDir, "package.json")),
    valibot: readInstalledVersion("valibot"),
    zod: readInstalledVersion("zod"),
    yup: readInstalledVersion("yup"),
  },
};

const results = [];
const warnings = [];

for (const benchCase of cases) {
  const caseResults = [];
  for (const validator of validators) {
    const fn = runners[validator](benchCase.id);
    const correctness = benchCase.inputs.map((input) => fn(input));
    const timing = measure((i) =>
      fn(benchCase.inputs[i % benchCase.inputs.length]),
    );
    caseResults.push({
      validator,
      ...timing,
      accepted: correctness.filter(Boolean).length,
      total: correctness.length,
      signature: correctness.map((ok) => (ok ? "1" : "0")).join(""),
    });
  }

  const signatures = new Set(caseResults.map((result) => result.signature));
  if (signatures.size > 1) {
    warnings.push({
      case: benchCase.id,
      signatures: Object.fromEntries(
        caseResults.map((result) => [result.validator, result.signature]),
      ),
    });
  }

  results.push({
    id: benchCase.id,
    label: benchCase.label,
    note: benchCase.note,
    results: caseResults,
  });
}

const payload = { environment, warnings, cases: results };

if (args.has("--json")) {
  console.log(JSON.stringify(payload, null, 2));
} else {
  printMarkdown(payload);
}

if (warnings.length > 0 && !args.has("--allow-divergence")) {
  process.exitCode = 1;
}

function measure(fn) {
  let sink = 0;

  for (let i = 0; i < warmupIterations; i++) {
    sink ^= fn(i) ? 1 : 0;
  }

  const samples = [];
  for (let round = 0; round < rounds; round++) {
    const start = process.hrtime.bigint();
    for (let i = 0; i < iterations; i++) {
      sink ^= fn(i) ? 1 : 0;
    }
    const elapsedNs = Number(process.hrtime.bigint() - start);
    samples.push(elapsedNs / iterations);
  }

  samples.sort((a, b) => a - b);
  const minNs = samples[0] ?? Number.NaN;
  const p50Ns = samples[Math.floor(samples.length / 2)] ?? Number.NaN;

  return {
    minNs,
    p50Ns,
    opsPerSec: 1_000_000_000 / minNs,
    sink,
  };
}

function printMarkdown(payload) {
  const { environment: env } = payload;

  console.log("# TypeScript validator benchmarks");
  console.log();
  console.log(
    `Node ${env.node} on ${env.platform}/${env.arch}; ${env.rounds} rounds x ${env.iterations.toLocaleString()} iterations, ${env.warmupIterations.toLocaleString()} warmup iterations.`,
  );
  console.log(
    `Versions: wellformed-ts ${env.versions["wellformed-ts"]}, Valibot ${env.versions.valibot}, Zod ${env.versions.zod}, Yup ${env.versions.yup}.`,
  );
  console.log();
  console.log("| Case | wellformed-ts | Valibot | Zod | Yup |");
  console.log("|---|---:|---:|---:|---:|");

  for (const benchCase of payload.cases) {
    const byValidator = Object.fromEntries(
      benchCase.results.map((result) => [result.validator, result]),
    );
    console.log(
      `| ${benchCase.label} | ${formatNs(byValidator["wellformed-ts"].minNs)} | ${formatNs(byValidator.valibot.minNs)} | ${formatNs(byValidator.zod.minNs)} | ${formatNs(byValidator.yup.minNs)} |`,
    );
  }

  console.log();
  console.log("## Notes");
  console.log();
  console.log(
    "- Before timing each case, the harness checks that all validators accept and reject the same sample inputs.",
  );
  for (const benchCase of payload.cases) {
    console.log(`- ${benchCase.label}: ${benchCase.note}`);
  }

  if (payload.warnings.length > 0) {
    console.log();
    console.log("## Correctness warnings");
    console.log();
    for (const warning of payload.warnings) {
      console.log(
        `- ${warning.case}: ${Object.entries(warning.signatures)
          .map(([name, signature]) => `${name}=${signature}`)
          .join(", ")}`,
      );
    }
  }
}

function formatNs(ns) {
  if (ns >= 1000) return `${(ns / 1000).toFixed(2)} us`;
  return `${ns.toFixed(1)} ns`;
}

function readInstalledVersion(packageName) {
  try {
    return readPackageVersion(require.resolve(`${packageName}/package.json`));
  } catch {
    try {
      let current = dirname(require.resolve(packageName));
      while (current !== dirname(current)) {
        const packageJson = join(current, "package.json");
        if (existsSync(packageJson)) return readPackageVersion(packageJson);
        current = dirname(current);
      }
    } catch {
      // Fall through to unknown.
    }
    return "unknown";
  }
}

function readPackageVersion(path) {
  return JSON.parse(readFileSync(path, "utf8")).version;
}

function isHttpUrl(value) {
  if (typeof value !== "string") return false;
  return value.startsWith("http://") || value.startsWith("https://");
}

function extractDigits(value) {
  if (typeof value !== "string") return "";
  let digits = "";
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (code >= 48 && code <= 57) digits += value.charAt(i);
  }
  return digits;
}

function isValidSsn(value) {
  const digits = extractDigits(value);
  if (digits.length !== 9) return false;
  const area =
    (digits.charCodeAt(0) - 48) * 100 +
    (digits.charCodeAt(1) - 48) * 10 +
    (digits.charCodeAt(2) - 48);
  const group = (digits.charCodeAt(3) - 48) * 10 + (digits.charCodeAt(4) - 48);
  const serial =
    (digits.charCodeAt(5) - 48) * 1000 +
    (digits.charCodeAt(6) - 48) * 100 +
    (digits.charCodeAt(7) - 48) * 10 +
    (digits.charCodeAt(8) - 48);
  return (
    area !== 0 && area !== 666 && area < 900 && group !== 0 && serial !== 0
  );
}

function isValidCreditCard(value) {
  const digits = extractDigits(value);
  if (!luhn(digits)) return false;
  return (
    isVisa(digits) ||
    isMastercard(digits) ||
    isAmex(digits) ||
    isDiscover(digits)
  );
}

function luhn(digits) {
  if (digits.length === 0) return false;
  let sum = 0;
  let double = false;
  for (let i = digits.length - 1; i >= 0; i--) {
    let digit = digits.charCodeAt(i) - 48;
    if (digit < 0 || digit > 9) return false;
    if (double) {
      digit *= 2;
      if (digit > 9) digit -= 9;
    }
    sum += digit;
    double = !double;
  }
  return sum % 10 === 0;
}

function isVisa(digits) {
  return digits.length === 16 && digits.startsWith("4");
}

function isMastercard(digits) {
  if (digits.length !== 16) return false;
  const prefix2 = Number.parseInt(digits.slice(0, 2), 10);
  if (prefix2 >= 51 && prefix2 <= 55) return true;
  const prefix4 = Number.parseInt(digits.slice(0, 4), 10);
  return prefix4 >= 2221 && prefix4 <= 2720;
}

function isAmex(digits) {
  return (
    digits.length === 15 && (digits.startsWith("34") || digits.startsWith("37"))
  );
}

function isDiscover(digits) {
  if (digits.length !== 16) return false;
  if (digits.startsWith("6011") || digits.startsWith("65")) return true;
  const prefix3 = Number.parseInt(digits.slice(0, 3), 10);
  return prefix3 >= 644 && prefix3 <= 649;
}

// biome-ignore lint/complexity/noExcessiveCognitiveComplexity: Keep the benchmark helper close to the production IBAN logic.
function isValidIban(value) {
  if (typeof value !== "string") return false;
  const codes = [];
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (code === 32 || (code >= 9 && code <= 13)) continue;
    const upper = code >= 97 && code <= 122 ? code - 32 : code;
    if (!isAsciiAlphanumeric(upper)) return false;
    codes.push(upper);
  }
  if (codes.length < 5 || codes.length > 34) return false;
  if (!isAsciiAlpha(codes[0]) || !isAsciiAlpha(codes[1])) return false;
  if (!isAsciiDigit(codes[2]) || !isAsciiDigit(codes[3])) return false;
  const country = String.fromCharCode(codes[0], codes[1]);
  const expectedLength = ibanCountryLengths[country];
  if (expectedLength !== undefined && codes.length !== expectedLength)
    return false;

  let remainder = 0;
  for (let i = 4; i < codes.length; i++) {
    remainder = ibanMod97Step(codes[i], remainder);
  }
  for (let i = 0; i < 4; i++) {
    remainder = ibanMod97Step(codes[i], remainder);
  }
  return remainder === 1;
}

function ibanMod97Step(code, remainder) {
  if (isAsciiDigit(code)) return (remainder * 10 + (code - 48)) % 97;
  return (remainder * 100 + (code - 55)) % 97;
}

function isAsciiDigit(code) {
  return code >= 48 && code <= 57;
}

function isAsciiAlpha(code) {
  return (code >= 65 && code <= 90) || (code >= 97 && code <= 122);
}

function isAsciiAlphanumeric(code) {
  return isAsciiAlpha(code) || isAsciiDigit(code);
}
