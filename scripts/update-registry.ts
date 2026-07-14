/**
 * Update package versions in src/types/store.json by querying the GitHub
 * API for the latest release of each repository.
 *
 * Usage: bun run scripts/update-registry.ts
 *
 * In CI (GITHUB_TOKEN set), authenticated requests avoid 60-req/hr limit.
 * Without a token the script still works but may hit rate limits with
 * repos > 60.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  apiDelay,
  fetchApi,
  repoSlug,
  stripLeadingV,
  type StoreRegistry,
} from "./shared";

interface GitHubRelease {
  tag_name: string;
  draft: boolean;
  prerelease: boolean;
}

const STORE_PATH = join(import.meta.dirname, "..", "src", "types", "store.json");

async function fetchLatestRelease(
  owner: string,
  repo: string,
): Promise<GitHubRelease | null> {
  const url = `https://api.github.com/repos/${owner}/${repo}/releases/latest`;
  const response = await fetchApi(url);
  if (!response) return null;

  if (response.status === 404) {
    // No releases — repo may use tags without releases
    console.warn(`  ⚠ No releases found for ${owner}/${repo} (404)`);
    return null;
  }

  if (!response.ok) {
    console.warn(`  ⚠ Failed to fetch releases for ${owner}/${repo}: HTTP ${response.status}`);
    return null;
  }

  const data = (await response.json()) as GitHubRelease;
  return data;
}

/** Compare two version strings using simple semver-like comparison. */
function isNewer(current: string, latest: string): boolean {
  // If they're the same, not newer
  if (current === latest) return false;

  // Strip pre-release suffixes (e.g. "1.0.0-beta" → "1.0.0") before
  // comparison so isNewer("1.0.0", "1.0.0-beta") correctly returns true.
  const parse = (v: string): number[] =>
    v.replace(/-.*$/, "").split(".").map((p) => {
      const n = parseInt(p, 10);
      return isNaN(n) ? 0 : n;
    });

  const curr = parse(current);
  const latest_ = parse(latest);

  for (let i = 0; i < Math.max(curr.length, latest_.length); i++) {
    const c = curr[i] ?? 0;
    const l = latest_[i] ?? 0;
    if (l > c) return true;
    if (c > l) return false;
  }
  return false;
}

async function main() {
  console.log("Reading store.json...");
  const store: StoreRegistry = JSON.parse(readFileSync(STORE_PATH, "utf-8"));
  let changed = false;
  let updated = 0;
  let skipped = 0;

  const allPaks = [...store.emu_paks, ...store.tool_paks];

  for (const pak of allPaks) {
    const slug = repoSlug(pak.repository);
    if (!slug) {
      console.warn(`⚠ Skipping ${pak.name}: invalid repository URL "${pak.repository}"`);
      skipped++;
      continue;
    }

    // Repos with explicit download_url use a custom download flow;
    // their version field may not correspond to a GitHub release tag.
    if (pak.download_url) {
      console.log(`⏭ ${pak.name}: uses download_url override, skipping`);
      skipped++;
      continue;
    }

    console.log(`🔍 ${pak.name} (${slug.owner}/${slug.repo}): current=${pak.version}`);
    const release = await fetchLatestRelease(slug.owner, slug.repo);

    if (!release) {
      skipped++;
      continue;
    }

    const latestVersion = stripLeadingV(release.tag_name);
    console.log(`   latest=${latestVersion} (tag=${release.tag_name}, draft=${release.draft}, prerelease=${release.prerelease})`);

    if (release.draft) {
      console.log(`   ⏭ skipping draft release`);
      skipped++;
      continue;
    }

    if (isNewer(pak.version, latestVersion)) {
      console.log(`   ✅ updating ${pak.version} → ${latestVersion}`);
      pak.version = latestVersion;
      changed = true;
      updated++;
    } else if (pak.version !== latestVersion) {
      console.log(`   (current ${pak.version} is newer or equal to ${latestVersion}, keeping)`);
      skipped++;
    } else {
      console.log(`   ✓ up to date`);
      skipped++;
    }

    // Small delay between requests to be polite to the API
    await apiDelay();
  }

  if (changed) {
    console.log(`\nWriting updated store.json (${updated} packages updated)...`);
    writeFileSync(STORE_PATH, JSON.stringify(store, null, 2) + "\n");
    console.log("Done!");
  } else {
    console.log(`\nNo updates needed (${skipped} packages checked).`);
  }
}

// Only run when executed directly, not when imported
if (import.meta.main) {
  main().catch((err) => {
    console.error("Failed:", err);
    process.exit(1);
  });
}
