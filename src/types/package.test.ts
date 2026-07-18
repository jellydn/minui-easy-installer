import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { fetchPackageRegistry, RegistryCache } from "./package";
import type { PackageRegistry } from "./package";

function makeRegistry(overrides?: Partial<PackageRegistry>): PackageRegistry {
  return {
    version: "1.0.0",
    packages: [
      {
        name: "TestPak",
        version: "1.0.0",
        category: "Utilities",
        description: "A test package",
        repository: "https://github.com/test/pak",
        downloads: 100,
        rating: 4.5,
        artifactUrl: "https://example.com/pak.zip",
        checksum: null,
        supportedDevices: [],
        installPathRules: {
          targetDir: "/Tools",
          extractToRoot: false,
        },
      },
    ],
    ...overrides,
  };
}

describe("RegistryCache", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("get() returns null when cache is empty", () => {
    const cache = new RegistryCache();
    expect(cache.get()).toBeNull();
  });

  test("set/get roundtrip — stores and retrieves a registry", () => {
    const cache = new RegistryCache();
    const registry = makeRegistry();

    cache.set(registry);
    const result = cache.get();

    expect(result).not.toBeNull();
    expect(result!.version).toBe("1.0.0");
    expect(result!.packages[0].name).toBe("TestPak");
  });

  test("get() returns null after TTL expires", () => {
    const cache = new RegistryCache(0); // instant expiry
    const registry = makeRegistry();

    cache.set(registry);

    // Advance time past the TTL (0ms + 1ms)
    vi.advanceTimersByTime(1);

    expect(cache.get()).toBeNull();
  });

  test("get() returns cached value before TTL expires", () => {
    const cache = new RegistryCache(60_000); // 1 minute TTL
    const registry = makeRegistry();

    cache.set(registry);

    // Advance time to just before expiry
    vi.advanceTimersByTime(59_999);

    const result = cache.get();
    expect(result).not.toBeNull();
    expect(result!.version).toBe("1.0.0");
  });

  test("get() returns null exactly at TTL boundary", () => {
    const cache = new RegistryCache(10_000);
    cache.set(makeRegistry());

    vi.advanceTimersByTime(10_000);

    expect(cache.get()).toBeNull();
  });

  test("clear() invalidates the cache", () => {
    const cache = new RegistryCache();
    cache.set(makeRegistry());

    expect(cache.get()).not.toBeNull();

    cache.clear();
    expect(cache.get()).toBeNull();
  });

  test("clear() on empty cache is a no-op", () => {
    const cache = new RegistryCache();
    expect(cache.get()).toBeNull();
    cache.clear();
    expect(cache.get()).toBeNull();
  });

  test("multiple set/get cycles work correctly", () => {
    const cache = new RegistryCache(60_000);

    // First set
    cache.set(makeRegistry({ version: "1.0.0" }));
    expect(cache.get()!.version).toBe("1.0.0");

    // Overwrite
    cache.set(makeRegistry({ version: "2.0.0" }));
    expect(cache.get()!.version).toBe("2.0.0");

    // Clear and re-set
    cache.clear();
    expect(cache.get()).toBeNull();

    cache.set(makeRegistry({ version: "3.0.0" }));
    expect(cache.get()!.version).toBe("3.0.0");
  });

  test("set() updates the timestamp — TTL resets on each set", () => {
    const cache = new RegistryCache(60_000);
    cache.set(makeRegistry());

    // Advance 55 seconds
    vi.advanceTimersByTime(55_000);
    expect(cache.get()).not.toBeNull(); // still valid

    // Re-set resets the timer
    cache.set(makeRegistry({ version: "2.0.0" }));

    // Advance another 55 seconds — should still be valid because set reset the timer
    vi.advanceTimersByTime(55_000);
    expect(cache.get()).not.toBeNull();
  });

  test("cache instances are independent", () => {
    const cacheA = new RegistryCache();
    const cacheB = new RegistryCache();

    cacheA.set(makeRegistry({ version: "A" }));
    cacheB.set(makeRegistry({ version: "B" }));

    expect(cacheA.get()!.version).toBe("A");
    expect(cacheB.get()!.version).toBe("B");

    cacheA.clear();
    expect(cacheA.get()).toBeNull();
    expect(cacheB.get()!.version).toBe("B"); // cacheB unaffected
  });

  test("default TTL is 5 minutes", () => {
    const cache = new RegistryCache();
    cache.set(makeRegistry());

    // Advance 4 minutes 59 seconds — should still be valid
    vi.advanceTimersByTime(4 * 60_000 + 59_000);
    expect(cache.get()).not.toBeNull();

    // Advance 1 more second to hit exactly 5 minutes
    vi.advanceTimersByTime(1_000);
    expect(cache.get()).toBeNull();
  });

  test("set() with different data doesn't leave stale state", () => {
    const cache = new RegistryCache();
    const pakA = makeRegistry().packages[0];
    const pakB = {
      ...pakA,
      name: "PakB",
      version: "2.0.0",
      category: "Emulators" as const,
    };

    cache.set(
      makeRegistry({ packages: [{ ...pakA, name: "PakA", version: "1.0.0" }] }),
    );
    cache.set(makeRegistry({ packages: [pakB] }));

    const result = cache.get()!;
    expect(result.packages[0].name).toBe("PakB");
    expect(result.packages).toHaveLength(1);
  });
});

describe("fetchPackageRegistry", () => {
  test("loads and converts local store.json", async () => {
    const result = await fetchPackageRegistry();

    expect(result.success).toBe(true);
    if (!result.success) return;

    expect(result.data.version).toBe("1.0.0");
    expect(result.data.packages.length).toBeGreaterThan(0);

    // Verify all packages are present
    const names = result.data.packages.map((p) => p.name);

    // Emu paks
    expect(names).toContain("Dreamcast");
    expect(names).toContain("Nintendo DS");
    expect(names).toContain("N64");
    expect(names).toContain("Pico-8");
    expect(names).toContain("Portmaster");
    expect(names).toContain("PSP");
    expect(names).toContain("MAME 2003 Plus");

    // Tool paks
    expect(names).toContain("Wifi");
    expect(names).toContain("SSH Server");
    expect(names).toContain("Grout");
    expect(names).toContain("Syncthing");
    expect(names).toContain("DotClean");

    // Verify Grout uses custom download_url
    const grout = result.data.packages.find((p) => p.name === "Grout")!;
    expect(grout.artifactUrl).toBe(
      "https://github.com/rommapp/grout/releases/download/v4.8.1.0/Grout-MinUI.zip",
    );
    expect(grout.category).toBe("Utilities");

    // Verify emu paks go to Emus
    const dc = result.data.packages.find((p) => p.name === "Dreamcast")!;
    expect(dc.category).toBe("Emulators");
    expect(dc.installPathRules.targetDir).toBe("/Emus");

    // Verify device-filtered tools have supportedDevices
    const mediaPlayer = result.data.packages.find(
      (p) => p.name === "Media Player",
    )!;
    expect(mediaPlayer.supportedDevices).toEqual(["trimui-brick"]);

    const dotClean = result.data.packages.find((p) => p.name === "DotClean")!;
    expect(dotClean.supportedDevices).toEqual([
      "trimui-brick",
      "trimui-smart-pro",
    ]);
    expect(dotClean.artifactUrl).toBe(
      "https://github.com/tanbase/minui-dotclean-pak/releases/download/0.2.0/DotClean.pak.zip",
    );
    expect(dotClean.category).toBe("Utilities");
    expect(dotClean.installPathRules.targetDir).toBe("/Tools");

    // Verify default tools have empty supportedDevices
    const wifi = result.data.packages.find((p) => p.name === "Wifi")!;
    expect(wifi.supportedDevices).toEqual([]);
  });
});
