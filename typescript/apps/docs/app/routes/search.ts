import { createFromSource } from "fumadocs-core/search/server";
import { source } from "@/lib/source";

const server = createFromSource(source, {
  // https://docs.orama.com/docs/orama-js/supported-languages
  language: "english",
});

// Prerendered to a static Orama index; the client downloads and queries it
// in-browser (see `RootProvider search={{ options: { type: "static" } }}`).
export async function loader() {
  return server.staticGET();
}
