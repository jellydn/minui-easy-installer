import { describe, expect, it } from "vitest";
import {
  buildCustomFork,
  buildReleaseUrl,
  FORK_PRESETS,
  getForkCacheKey,
  rehydrateFork,
} from "./fork";

describe("buildReleaseUrl", () => {
  it("builds URL for official fork", () => {
    expect(buildReleaseUrl(FORK_PRESETS.official)).toBe(
      "https://api.github.com/repos/shauninman/MinUI/releases/latest",
    );
  });

  it("builds URL for MinUI-Zero", () => {
    expect(buildReleaseUrl(FORK_PRESETS["minui-zero"])).toBe(
      "https://api.github.com/repos/danklammer/MinUI-Zero/releases/latest",
    );
  });

  it("builds URL for MinUITSP", () => {
    expect(buildReleaseUrl(FORK_PRESETS.minuitsp)).toBe(
      "https://api.github.com/repos/jellydn/MinUITSP/releases/latest",
    );
  });
});

describe("getForkCacheKey", () => {
  it("returns owner/repo for presets", () => {
    expect(getForkCacheKey(FORK_PRESETS.official)).toBe("shauninman/MinUI");
    expect(getForkCacheKey(FORK_PRESETS["minui-zero"])).toBe(
      "danklammer/MinUI-Zero",
    );
    expect(getForkCacheKey(FORK_PRESETS.minuitsp)).toBe("jellydn/MinUITSP");
  });
});

describe("buildCustomFork", () => {
  it("parses valid owner/repo", () => {
    const fork = buildCustomFork("my-user/my-repo");
    expect(fork).toEqual({
      label: "my-user/my-repo",
      owner: "my-user",
      repo: "my-repo",
      minuiTxtPrefix: "my-repo",
    });
  });

  it("trims whitespace", () => {
    const fork = buildCustomFork("  user/repo  ");
    expect(fork?.owner).toBe("user");
    expect(fork?.repo).toBe("repo");
  });

  it("rejects empty input", () => {
    expect(buildCustomFork("")).toBeNull();
    expect(buildCustomFork("  ")).toBeNull();
  });

  it("rejects missing slash", () => {
    expect(buildCustomFork("justrepo")).toBeNull();
  });

  it("rejects too many slashes", () => {
    expect(buildCustomFork("a/b/c")).toBeNull();
  });

  it("rejects owner with spaces", () => {
    expect(buildCustomFork("my user/repo")).toBeNull();
  });

  it("accepts dots, hyphens, underscores", () => {
    expect(buildCustomFork("my-org.my_team/repo-name")).not.toBeNull();
  });
});

describe("rehydrateFork", () => {
  it("returns preset when owner/repo match official", () => {
    const result = rehydrateFork({ owner: "shauninman", repo: "MinUI" });
    expect(result).toBe(FORK_PRESETS.official);
  });

  it("returns preset when owner/repo match MinUI-Zero", () => {
    const result = rehydrateFork({ owner: "danklammer", repo: "MinUI-Zero" });
    expect(result).toBe(FORK_PRESETS["minui-zero"]);
  });

  it("returns preset when owner/repo match MinUITSP", () => {
    const result = rehydrateFork({ owner: "jellydn", repo: "MinUITSP" });
    expect(result).toBe(FORK_PRESETS.minuitsp);
  });

  it("returns custom fork for unknown owner/repo", () => {
    const result = rehydrateFork({ owner: "custom", repo: "thing" });
    expect(result).toEqual({
      label: "custom/thing",
      owner: "custom",
      repo: "thing",
      minuiTxtPrefix: "thing",
    });
  });

  it("returns null for null input", () => {
    expect(rehydrateFork(null)).toBeNull();
  });

  it("returns null for non-object input", () => {
    expect(rehydrateFork("string")).toBeNull();
  });

  it("returns null when owner is missing", () => {
    expect(rehydrateFork({ repo: "MinUI" })).toBeNull();
  });

  it("returns null when repo is missing", () => {
    expect(rehydrateFork({ owner: "shauninman" })).toBeNull();
  });

  it("returns null for empty owner/repo strings", () => {
    expect(rehydrateFork({ owner: "", repo: "" })).toBeNull();
  });
});
