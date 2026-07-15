import { describe, expect, it } from "vitest";
import { asError, errorMessage } from "./errors";

describe("errorMessage", () => {
  it("extracts message from Error instances", () => {
    expect(errorMessage(new Error("boom"))).toBe("boom");
    expect(errorMessage(new TypeError("bad type"))).toBe("bad type");
  });

  it("passes through plain strings unchanged", () => {
    expect(errorMessage("plain string")).toBe("plain string");
    expect(errorMessage("")).toBe("");
  });

  it("extracts message property from error-like objects", () => {
    expect(errorMessage({ message: "hello" })).toBe("hello");
    expect(errorMessage({ message: 42 })).toBe("42");
  });

  it("falls back to Unknown error for unrecognised values", () => {
    expect(errorMessage(null)).toBe("Unknown error");
    expect(errorMessage(undefined)).toBe("Unknown error");
    expect(errorMessage(42)).toBe("Unknown error");
    expect(errorMessage({})).toBe("Unknown error");
  });
});

describe("asError", () => {
  it("returns the same Error instance", () => {
    const e = new Error("original");
    expect(asError(e)).toBe(e);
  });

  it("wraps a plain string in an Error", () => {
    const result = asError("plain string");
    expect(result).toBeInstanceOf(Error);
    expect(result.message).toBe("plain string");
  });

  it("creates a new Error with the message from an error-like object", () => {
    const result = asError({ message: "nested" });
    expect(result).toBeInstanceOf(Error);
    expect(result.message).toBe("nested");
  });
});
