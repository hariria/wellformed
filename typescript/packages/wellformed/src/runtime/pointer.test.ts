import { describe, expect, it } from "vitest";
import {
  joinPointers,
  parsePointer,
  pointerExists,
  resolveJsonPointer,
  resolveJsonPointerSingle,
  setJsonPointer,
  toPointer,
} from "./pointer.js";

describe("resolveJsonPointer", () => {
  it("resolves empty path to root value", () => {
    expect(resolveJsonPointer({ foo: 1 }, "")).toEqual([{ foo: 1 }]);
  });

  it("treats slash as the empty object key per RFC 6901", () => {
    expect(resolveJsonPointer({ "": 1 }, "/")).toEqual([1]);
    expect(resolveJsonPointer({ foo: 1 }, "/")).toEqual([]);
  });

  it("resolves simple object paths", () => {
    const obj = { foo: { bar: 42 } };
    expect(resolveJsonPointer(obj, "/foo")).toEqual([{ bar: 42 }]);
    expect(resolveJsonPointer(obj, "/foo/bar")).toEqual([42]);
  });

  it("resolves array indices", () => {
    const arr = [10, 20, 30];
    expect(resolveJsonPointer(arr, "/0")).toEqual([10]);
    expect(resolveJsonPointer(arr, "/1")).toEqual([20]);
    expect(resolveJsonPointer(arr, "/2")).toEqual([30]);
  });

  it("rejects partial numeric array index segments", () => {
    expect(resolveJsonPointer([10, 20, 30], "/1abc")).toEqual([]);
  });

  it("resolves nested array paths", () => {
    const obj = { items: [{ name: "a" }, { name: "b" }] };
    expect(resolveJsonPointer(obj, "/items/0/name")).toEqual(["a"]);
    expect(resolveJsonPointer(obj, "/items/1/name")).toEqual(["b"]);
  });

  it("resolves wildcard paths", () => {
    const obj = { items: [{ id: 1 }, { id: 2 }, { id: 3 }] };
    expect(resolveJsonPointer(obj, "/items/*/id")).toEqual([1, 2, 3]);
  });

  it("returns empty array for missing paths", () => {
    expect(resolveJsonPointer({ foo: 1 }, "/bar")).toEqual([]);
    expect(resolveJsonPointer({ foo: 1 }, "/foo/bar")).toEqual([]);
  });

  it("handles escaped characters per RFC 6901", () => {
    const obj = { "a/b": { "c~d": 42 } };
    expect(resolveJsonPointer(obj, "/a~1b/c~0d")).toEqual([42]);
  });

  it("throws on invalid pointer format", () => {
    expect(() => resolveJsonPointer({}, "foo")).toThrow(/Invalid JSON Pointer/);
  });
});

describe("resolveJsonPointerSingle", () => {
  it("returns first match", () => {
    expect(resolveJsonPointerSingle({ foo: 42 }, "/foo")).toBe(42);
  });

  it("returns undefined for missing paths", () => {
    expect(resolveJsonPointerSingle({ foo: 42 }, "/bar")).toBeUndefined();
  });
});

describe("pointerExists", () => {
  it("returns true for existing paths", () => {
    expect(pointerExists({ foo: { bar: 42 } }, "/foo")).toBe(true);
    expect(pointerExists({ foo: { bar: 42 } }, "/foo/bar")).toBe(true);
  });

  it("returns false for missing paths", () => {
    expect(pointerExists({ foo: 42 }, "/bar")).toBe(false);
    expect(pointerExists({ foo: 42 }, "/foo/bar")).toBe(false);
  });

  it("returns false for undefined values", () => {
    expect(pointerExists({ foo: undefined }, "/foo")).toBe(false);
  });

  it("returns true for null values (path exists, value is null)", () => {
    expect(pointerExists({ foo: null }, "/foo")).toBe(true);
  });

  it("returns false for invalid pointers", () => {
    expect(pointerExists({}, "invalid")).toBe(false);
  });
});

describe("setJsonPointer", () => {
  it("sets value at simple path", () => {
    const obj: Record<string, unknown> = {};
    setJsonPointer(obj, "/foo", 42);
    expect(obj.foo).toBe(42);
  });

  it("sets value at nested path, creating intermediates", () => {
    const obj: Record<string, unknown> = {};
    setJsonPointer(obj, "/foo/bar/baz", 42);
    expect(obj).toEqual({ foo: { bar: { baz: 42 } } });
  });

  it("overwrites existing values", () => {
    const obj: Record<string, unknown> = { foo: 1 };
    setJsonPointer(obj, "/foo", 2);
    expect(obj.foo).toBe(2);
  });

  it("throws for root path", () => {
    expect(() => setJsonPointer({}, "", 42)).toThrow(/Cannot set root value/);
  });

  it("sets slash path as the empty object key", () => {
    const obj: Record<string, unknown> = {};
    setJsonPointer(obj, "/", 42);
    expect(obj[""]).toBe(42);
  });

  it("throws for invalid pointer", () => {
    expect(() => setJsonPointer({}, "foo", 42)).toThrow(/Invalid JSON Pointer/);
  });
});

describe("joinPointers", () => {
  it("joins base and relative paths", () => {
    expect(joinPointers("/foo", "/bar")).toBe("/foo/bar");
    expect(joinPointers("/foo/bar", "/baz")).toBe("/foo/bar/baz");
  });

  it("adds leading slash to relative if missing", () => {
    expect(joinPointers("/foo", "bar")).toBe("/foo/bar");
  });

  it("returns relative if base is empty or root", () => {
    expect(joinPointers("", "/foo")).toBe("/foo");
    expect(joinPointers("/", "/foo")).toBe("/foo");
  });

  it("returns base if relative is empty", () => {
    expect(joinPointers("/foo", "")).toBe("/foo");
  });
});

describe("toPointer", () => {
  it("creates pointer from segments", () => {
    expect(toPointer(["foo", "bar"])).toBe("/foo/bar");
    expect(toPointer(["0", "1"])).toBe("/0/1");
  });

  it("returns empty string for empty segments", () => {
    expect(toPointer([])).toBe("");
  });

  it("encodes special characters", () => {
    expect(toPointer(["a/b", "c~d"])).toBe("/a~1b/c~0d");
  });
});

describe("parsePointer", () => {
  it("parses pointer into segments", () => {
    expect(parsePointer("/foo/bar")).toEqual(["foo", "bar"]);
    expect(parsePointer("/0/1")).toEqual(["0", "1"]);
  });

  it("returns empty array for empty root path", () => {
    expect(parsePointer("")).toEqual([]);
  });

  it("parses slash as the empty object key", () => {
    expect(parsePointer("/")).toEqual([""]);
  });

  it("decodes special characters", () => {
    expect(parsePointer("/a~1b/c~0d")).toEqual(["a/b", "c~d"]);
  });

  it("throws for invalid pointer", () => {
    expect(() => parsePointer("foo")).toThrow(/Invalid JSON Pointer/);
  });
});
