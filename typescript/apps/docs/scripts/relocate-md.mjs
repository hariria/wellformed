// After `react-router build`, move the prerendered markdown twins from the
// internal `/_docs-md/*` route output to public `/docs/<slug>.md` files and
// drop the internal directory. This gives every `/docs/<slug>` page a matching
// `/docs/<slug>.md` without exposing an `/_docs-md` URL.
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join, relative } from "node:path";

const SRC_DIR = "build/client/_docs-md";
const OUT_DIR = "build/client/docs";

async function walk(dir) {
  const out = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...(await walk(full)));
    else out.push(full);
  }
  return out;
}

try {
  const files = await walk(SRC_DIR);
  let count = 0;
  for (const file of files) {
    // Skip React Router's `.data` sidecars; we only want the raw markdown.
    if (file.endsWith(".data")) continue;
    const slug = relative(SRC_DIR, file);
    const dest = join(OUT_DIR, `${slug}.md`);
    await mkdir(dirname(dest), { recursive: true });
    await writeFile(dest, await readFile(file));
    count++;
  }
  await rm(SRC_DIR, { recursive: true, force: true });
  console.log(
    `[relocate-md] wrote ${count} markdown files to /docs/<slug>.md and removed _docs-md/`,
  );
} catch (err) {
  if (err && err.code === "ENOENT") {
    console.log("[relocate-md] no _docs-md output found, nothing to relocate");
  } else {
    throw err;
  }
}
