import { describe, expect, it, vi } from "vitest";
import { bufferToBase64 } from "./bios";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("bufferToBase64", () => {
  it("encodes a small buffer round-trip", () => {
    const data = new Uint8Array([104, 105]).buffer; // "hi"
    const b64 = bufferToBase64(data);
    expect(b64).toBe("aGk=");
  });

  it("encodes an empty buffer", () => {
    expect(bufferToBase64(new ArrayBuffer(0))).toBe("");
  });

  it("handles a large buffer without throwing", () => {
    // 1 MB random bytes — exercises the chunking path.
    const bytes = new Uint8Array(1024 * 1024);
    for (let i = 0; i < bytes.length; i += 1) {
      bytes[i] = i & 0xff;
    }
    const b64 = bufferToBase64(bytes.buffer);
    expect(typeof b64).toBe("string");
    expect(b64.length).toBeGreaterThan(0);
    // Decode and check it round-trips.
    const decoded = atob(b64);
    expect(decoded.length).toBe(bytes.length);
    expect(decoded.charCodeAt(0)).toBe(0);
    expect(decoded.charCodeAt(255)).toBe(255);
  });

  it("encodes binary data that includes NUL and high bytes", () => {
    const bytes = new Uint8Array([0, 0xff, 0x7f, 0x80, 0x00, 0xab]);
    const b64 = bufferToBase64(bytes.buffer);
    const decoded = atob(b64);
    expect(Array.from(decoded).map((c) => c.charCodeAt(0))).toEqual(
      Array.from(bytes),
    );
  });
});
