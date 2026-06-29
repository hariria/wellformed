import { access, readdir, readFile } from "node:fs/promises";
import { extname, join, posix, relative, sep } from "node:path";

const ROOT = "build/client";
const ORIGIN = "https://wellformed.local";

const DOCUMENT_EXTENSIONS = new Set([".html", ".md", ".txt"]);
const IGNORE_PROTOCOLS = new Set([
  "data:",
  "javascript:",
  "mailto:",
  "tel:",
]);

async function walk(dir) {
  const out = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...(await walk(full)));
    else out.push(full);
  }
  return out;
}

function toPublicPath(file) {
  const rel = relative(ROOT, file).split(sep).join(posix.sep);
  if (rel === "index.html") return "/";
  if (rel.endsWith("/index.html")) {
    return `/${rel.slice(0, -"index.html".length)}`;
  }
  return `/${rel}`;
}

function extractLinks(file, text) {
  const links = [];
  const htmlPatterns = [
    /\b(?:href|src)=["']([^"']+)["']/gi,
    /\b(?:href|src)=&quot;([^&]+)&quot;/gi,
  ];
  const markdownPatterns = [
    /!?\[[^\]]*]\(([^)\s]+)(?:\s+"[^"]*")?\)/g,
    /\[[^\]]+]:\s*(\S+)/g,
  ];
  const patterns = file.endsWith(".html")
    ? htmlPatterns
    : [...markdownPatterns, ...htmlPatterns];

  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(text))) {
      links.push(match[1]);
    }
  }
  return links;
}

function normalizeLink(raw, fromPath) {
  const href = raw.trim();
  if (!href || href.startsWith("#") || href.startsWith("{")) return null;
  if (href.startsWith("//")) return null;

  const explicitProtocol = href.match(/^([a-z][a-z0-9+.-]*):/i)?.[1];
  if (explicitProtocol) {
    const protocol = `${explicitProtocol.toLowerCase()}:`;
    if (IGNORE_PROTOCOLS.has(protocol)) return null;
  }

  let url;
  try {
    url = new URL(href, `${ORIGIN}${fromPath}`);
  } catch {
    return null;
  }

  if (url.origin !== ORIGIN) return null;
  return decodeURI(url.pathname);
}

async function exists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function hasBuiltTarget(pathname) {
  const clean = posix.normalize(pathname).replace(/^\/+/, "");
  if (clean === "" || clean === ".") {
    return exists(join(ROOT, "index.html"));
  }

  const direct = join(ROOT, clean);
  if (await exists(direct)) return true;

  if (pathname.endsWith("/")) {
    return exists(join(ROOT, clean, "index.html"));
  }

  if (extname(clean)) {
    return false;
  }

  return exists(join(ROOT, clean, "index.html"));
}

const files = await walk(ROOT);
const documents = files.filter((file) => DOCUMENT_EXTENSIONS.has(extname(file)));
const failures = [];
let checkedLinks = 0;

for (const file of documents) {
  const text = await readFile(file, "utf8");
  const fromPath = toPublicPath(file);
  for (const raw of extractLinks(file, text)) {
    const pathname = normalizeLink(raw, fromPath);
    if (!pathname) continue;
    checkedLinks++;
    if (!(await hasBuiltTarget(pathname))) {
      failures.push({ from: fromPath, href: raw, resolved: pathname });
    }
  }
}

if (failures.length > 0) {
  console.error(`[check-links] ${failures.length} broken internal link(s):`);
  for (const failure of failures) {
    console.error(
      `  ${failure.from} -> ${failure.href} (resolved ${failure.resolved})`,
    );
  }
  process.exit(1);
}

console.log(
  `[check-links] checked ${checkedLinks} internal links across ${documents.length} built documents`,
);
