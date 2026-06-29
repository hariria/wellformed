/**
 * JSON Pointer (RFC 6901) implementation with wildcard extension.
 */

/**
 * Resolve a JSON Pointer path against a value.
 * Supports standard RFC 6901 paths like "/foo/bar/0"
 * and wildcard extension "*" for all array elements.
 */
export function resolveJsonPointer(value: unknown, path: string): unknown[] {
  if (path === "") {
    return [value];
  }

  if (!path.startsWith("/")) {
    throw new Error(
      `Invalid JSON Pointer: must start with "/" or be empty: ${path}`,
    );
  }

  const segments = path
    .slice(1)
    .split("/")
    .map((s) => decodePointerSegment(s));

  return resolveSegments(value, segments);
}

/**
 * Resolve a single value at a JSON Pointer path.
 * Returns undefined if the path doesn't exist.
 */
export function resolveJsonPointerSingle(
  value: unknown,
  path: string,
): unknown {
  const results = resolveJsonPointer(value, path);
  return results[0];
}

/**
 * Check if a JSON Pointer path exists in the value.
 */
export function pointerExists(value: unknown, path: string): boolean {
  try {
    const results = resolveJsonPointer(value, path);
    return results.length > 0 && results.some((v) => v !== undefined);
  } catch {
    return false;
  }
}

/**
 * Set a value at a JSON Pointer path.
 * Creates intermediate objects/arrays as needed.
 */
export function setJsonPointer(
  obj: Record<string, unknown>,
  path: string,
  value: unknown,
): void {
  if (path === "") {
    throw new Error("Cannot set root value");
  }

  if (!path.startsWith("/")) {
    throw new Error(`Invalid JSON Pointer: must start with "/": ${path}`);
  }

  const segments = path
    .slice(1)
    .split("/")
    .map((s) => decodePointerSegment(s));

  let current: Record<string, unknown> = obj;

  for (let i = 0; i < segments.length - 1; i++) {
    const segment = segments[i];
    if (segment === undefined) continue;

    if (!(segment in current)) {
      // Create intermediate object or array
      const nextSegment = segments[i + 1];
      current[segment] = isArrayIndex(nextSegment) ? [] : {};
    }

    const next = current[segment];
    if (typeof next !== "object" || next === null) {
      throw new Error(`Cannot traverse through non-object at ${segment}`);
    }
    current = next as Record<string, unknown>;
  }

  const lastSegment = segments[segments.length - 1];
  if (lastSegment !== undefined) {
    current[lastSegment] = value;
  }
}

/**
 * Join two JSON Pointer paths.
 */
export function joinPointers(base: string, relative: string): string {
  if (relative === "") return base;
  if (base === "" || base === "/") return relative;
  const normalizedRelative = relative.startsWith("/")
    ? relative
    : `/${relative}`;
  return `${base}${normalizedRelative}`;
}

// Internal helpers

function resolveSegments(value: unknown, segments: string[]): unknown[] {
  if (segments.length === 0) {
    return [value];
  }

  const [first, ...rest] = segments;
  if (first === undefined) {
    return resolveSegments(value, rest);
  }

  // Wildcard: resolve against all array elements
  if (first === "*") {
    return resolveWildcard(value, rest);
  }

  // Array index
  if (Array.isArray(value)) {
    return resolveArraySegment(value, first, rest);
  }

  // Object property
  if (typeof value === "object" && value !== null) {
    return resolveObjectSegment(value as Record<string, unknown>, first, rest);
  }

  return [];
}

function resolveWildcard(value: unknown, rest: string[]): unknown[] {
  if (!Array.isArray(value)) {
    return [];
  }
  const results: unknown[] = [];
  for (const item of value) {
    results.push(...resolveSegments(item, rest));
  }
  return results;
}

function resolveArraySegment(
  value: unknown[],
  segment: string,
  rest: string[],
): unknown[] {
  if (!isArrayIndex(segment)) {
    return [];
  }
  const index = Number.parseInt(segment, 10);
  if (index >= value.length) return [];
  return resolveSegments(value[index], rest);
}

function resolveObjectSegment(
  value: Record<string, unknown>,
  segment: string,
  rest: string[],
): unknown[] {
  if (!(segment in value)) {
    return [];
  }
  return resolveSegments(value[segment], rest);
}

function decodePointerSegment(segment: string): string {
  // RFC 6901: ~1 → /, ~0 → ~
  return segment.replace(/~1/g, "/").replace(/~0/g, "~");
}

function encodePointerSegment(segment: string): string {
  // RFC 6901: ~ → ~0, / → ~1
  return segment.replace(/~/g, "~0").replace(/\//g, "~1");
}

function isArrayIndex(segment: string | undefined): boolean {
  if (segment === undefined) return false;
  return /^\d+$/.test(segment) && Number.isSafeInteger(Number(segment));
}

/**
 * Create a JSON Pointer string from segments.
 */
export function toPointer(segments: string[]): string {
  if (segments.length === 0) return "";
  return `/${segments.map(encodePointerSegment).join("/")}`;
}

/**
 * Parse a JSON Pointer string into segments.
 */
export function parsePointer(path: string): string[] {
  if (path === "") return [];
  if (!path.startsWith("/")) {
    throw new Error(`Invalid JSON Pointer: must start with "/": ${path}`);
  }
  return path
    .slice(1)
    .split("/")
    .map((s) => decodePointerSegment(s));
}
