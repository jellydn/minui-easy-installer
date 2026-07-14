/**
 * Discover new MinUI pak repositories from known contributors' GitHub
 * profiles that aren't yet in src/types/store.json.
 *
 * Usage: bun run scripts/discover-packages.ts
 *
 * Outputs a candidate list to stdout (human-readable) and to
 * scripts/discovered.json (machine-readable, for review).
 */

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const STORE_PATH = join(import.meta.dirname, "..", "src", "types", "store.json");
const OUTPUT_PATH = join(import.meta.dirname, "discovered.json");
const GITHUB_TOKEN = process.env.GITHUB_TOKEN;

interface StorePak {
  name: string;
  repository: string;
  version: string;
  pak_name: string;
  rom_folder?: string;
  download_url?: string;
  device?: string[];
  description?: string;
}

interface StoreRegistry {
  emu_paks: StorePak[];
  tool_paks: StorePak[];
}

interface GitHubRepo {
  name: string;
  full_name: string;
  html_url: string;
  description: string | null;
  topics: string[];
  stargazers_count: number;
  updated_at: string;
}

interface Candidate {
  name: string;
  repository: string;
  latest_version: string;
  description: string;
  author: string;
  stars: number;
  suggested_category: "Emulators" | "Utilities";
  suggested_pak_name: string;
  found_via: string;
}

/** Contributors whose repos to scan for new MinUI paks. */
const CONTRIBUTORS = [
  "josegonzalez",
  "ben16w",
  "jiserra",
  "laesetuc",
  "rommapp",
  // Common MinUI pak authors found across the community
  "shauninman",
];

function repoSlug(repository: string): { owner: string; repo: string } | null {
  const match = repository.match(/^https:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/);
  if (!match) return null;
  return { owner: match[1], repo: match[2] };
}

function headers(): Record<string, string> {
  const h: Record<string, string> = {
    "User-Agent": "minui-easy-installer-package-discoverer",
  };
  if (GITHUB_TOKEN) {
    h["Authorization"] = `Bearer ${GITHUB_TOKEN}`;
  }
  return h;
}

async function fetchRepos(username: string): Promise<GitHubRepo[]> {
  const all: GitHubRepo[] = [];
  let page = 1;

  while (true) {
    const url = `https://api.github.com/users/${username}/repos?per_page=100&page=${page}&sort=updated`;
    let response: Response;
    try {
      response = await fetch(url, { headers: headers() });
    } catch {
      console.warn(`  ⚠ Network error fetching repos for ${username}`);
      break;
    }

    if (!response.ok) {
      console.warn(`  ⚠ HTTP ${response.status} fetching repos for ${username}`);
      break;
    }

    const repos = (await response.json()) as GitHubRepo[];
    if (repos.length === 0) break;
    all.push(...repos);
    page++;
    await new Promise((r) => setTimeout(r, 200));
  }

  return all;
}

/** Check if a repo looks like a MinUI pak. */
function isMinUIRepo(repo: GitHubRepo): boolean {
  const signals = [repo.name, repo.description ?? "", ...repo.topics]
    .join(" ")
    .toLowerCase();

  return (
    signals.includes("minui") ||
    signals.includes("pak") ||
    repo.name.endsWith("-pak") ||
    (repo.topics.includes("minui") || repo.topics.includes("minui-pak"))
  );
}

/** Check if a repo has a release with .pak.zip or -MinUI.zip assets. */
async function hasPakRelease(owner: string, repo: string): Promise<string | null> {
  const url = `https://api.github.com/repos/${owner}/${repo}/releases/latest`;
  let response: Response;
  try {
    response = await fetch(url, { headers: headers() });
  } catch {
    return null;
  }

  if (!response.ok) return null;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const release = (await response.json()) as any;
  const tag = release?.tag_name;
  const assets: Array<{ name: string }> = release?.assets ?? [];

  const hasPak = assets.some(
    (a) => a.name.endsWith(".pak.zip") || a.name.endsWith("-MinUI.zip"),
  );

  if (hasPak && tag) {
    return tag.startsWith("v") ? tag.slice(1) : tag;
  }
  return null;
}

/** Guess the pak_name from the repo name. */
function guessPakName(repoName: string): string {
  // Strip common suffixes, convert kebab/snake to words
  return repoName
    .replace(/-pak$/, "")
    .replace(/^minui-/, "")
    .split(/[-_]/)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

/** Guess category based on repo name/description keywords. */
function guessCategory(repo: GitHubRepo): "Emulators" | "Utilities" {
  const signals = [repo.name, repo.description ?? "", ...repo.topics]
    .join(" ")
    .toLowerCase();

  const emuKeywords = ["emulator", "emu", "core", "mame", "retroarch", "game boy", "nes", "snes", "n64", "psp", "psx", "dreamcast", "ds", "gba", "gbc"];
  return emuKeywords.some((kw) => signals.includes(kw)) ? "Emulators" : "Utilities";
}

async function main() {
  console.log("Reading store.json...");
  const store: StoreRegistry = JSON.parse(readFileSync(STORE_PATH, "utf-8"));
  const knownRepos = new Set(
    [...store.emu_paks, ...store.tool_paks]
      .map((p) => repoSlug(p.repository))
      .filter(Boolean)
      .map((s) => `${s!.owner}/${s!.repo}`),
  );

  const candidates: Candidate[] = [];

  for (const contributor of CONTRIBUTORS) {
    console.log(`\n🔍 Scanning repos for ${contributor}...`);
    const repos = await fetchRepos(contributor);
    console.log(`   ${repos.length} repos found`);

    const minuiRepos = repos.filter(isMinUIRepo);
    console.log(`   ${minuiRepos.length} MinUI-related repos`);

    for (const repo of minuiRepos) {
      const fullName = repo.full_name;
      if (knownRepos.has(fullName)) {
        console.log(`   ⏭ ${fullName} — already in store.json`);
        continue;
      }

      console.log(`   🔍 Checking ${fullName}...`);
      const latestVersion = await hasPakRelease(
        fullName.split("/")[0],
        fullName.split("/")[1],
      );

      if (latestVersion) {
        const candidate: Candidate = {
          name: repo.name,
          repository: repo.html_url,
          latest_version: latestVersion,
          description: repo.description ?? "",
          author: contributor,
          stars: repo.stargazers_count,
          suggested_category: guessCategory(repo),
          suggested_pak_name: guessPakName(repo.name),
          found_via: `${contributor}'s repos`,
        };
        candidates.push(candidate);
        console.log(`      ✅ Found! v${latestVersion} (${candidate.suggested_category}, ${repo.stargazers_count} ⭐)`);
      } else {
        console.log(`      ⏭ No .pak.zip release found`);
      }

      await new Promise((r) => setTimeout(r, 200));
    }
  }

  // Report
  console.log(`\n${"=".repeat(60)}`);
  console.log(`Discovered ${candidates.length} new MinUI pak candidate(s):\n`);

  if (candidates.length === 0) {
    console.log("No new packages found.");
  } else {
    for (const c of candidates) {
      console.log(`  📦 ${c.name}`);
      console.log(`     Repo:     ${c.repository}`);
      console.log(`     Version:  ${c.latest_version}`);
      console.log(`     Category: ${c.suggested_category}`);
      console.log(`     Pak name: ${c.suggested_pak_name}`);
      console.log(`     Stars:    ${c.stars} ⭐`);
      console.log(`     Via:      ${c.found_via}`);
      if (c.description) console.log(`     Desc:     ${c.description}`);
      console.log();
    }

    console.log("Writing candidates to scripts/discovered.json...");
    writeFileSync(OUTPUT_PATH, JSON.stringify(candidates, null, 2) + "\n");
    console.log(`\nReview the candidates in ${OUTPUT_PATH}, then manually add`);
    console.log("the ones you want to src/types/store.json.");
  }
}

if (import.meta.main) {
  main().catch((err) => {
    console.error("Failed:", err);
    process.exit(1);
  });
}
