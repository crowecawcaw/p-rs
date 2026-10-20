#!/usr/bin/env node
// Release-note generation: two deterministic steps with one Claude step between.
// Adapted from xa11y's generator.
//
//   release-notes.mjs context <tag> [--since TAG] [--repo OWNER/NAME]
//       Walk main's history since the previous tag, attach each merged PR's title
//       and description, and print a context document on stdout.
//
//   release-notes.mjs render <tag> --entries FILE [--since TAG] [--repo OWNER/NAME]
//       Validate the JSON entries Claude wrote and render release-note markdown.
//
// Code does the mechanical parts (commit range, PR lookup, section order,
// changelog link). Claude only decides which changes users would notice and
// describes them, following .github/release-notes-prompt.md.
//
// This repo merges PRs with merge commits, so the walk follows main's first
// parent: each merge commit is one PR, and anything else is a direct push.
// Squash merges ("Title (#12)") work too.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { argv, env, exit, stderr, stdout } from "node:process";

// Generous caps: the substance of a change is often deep in a PR body.
const PR_DESCRIPTION_MAX_CHARS = 12_000;
const COMMIT_BODY_MAX_CHARS = 16_000;
const INNER_COMMITS_MAX = 50;

// Release bumps and dependency updates: keep the subject, skip the PR lookup.
const NO_ENRICHMENT_SUBJECT = /^(?:Release Snap Pea v|Bump |build\(deps[^)]*\)|chore(?:\([^)]*\))?:)/i;

const CATEGORIES = ["breaking", "deprecations", "features", "bug fixes"];
const SECTION_TITLES = {
  breaking: "Breaking Changes",
  deprecations: "Deprecations",
  features: "Features",
  "bug fixes": "Bug Fixes",
};
const RELEASE_TAG_PATTERN = /^v\d+\.\d+\.\d+/;

function runGit(...args) {
  return execFileSync("git", args, { encoding: "utf8", maxBuffer: 64 << 20 }).trim();
}

function getPreviousTag(currentTag) {
  const tags = runGit("tag", "--sort=-v:refname")
    .split("\n")
    .filter((tag) => RELEASE_TAG_PATTERN.test(tag));
  const idx = tags.indexOf(currentTag);
  if (idx === -1) return tags[0] ?? null;
  return tags[idx + 1] ?? null;
}

function getReadme() {
  return existsSync("README.md") ? readFileSync("README.md", "utf8").slice(0, 2500) : "";
}

// One entry per change that landed on main: a merged PR or a direct commit.
function getChanges(rangeSpec) {
  const log = runGit("log", "--first-parent", rangeSpec, "--pretty=format:%H%x00%P%x00%s%x00%b%x1e");
  if (!log) return [];
  const changes = [];
  for (const raw of log.split("\x1e")) {
    const entry = raw.trim();
    if (!entry) continue;
    const [hash, parents = "", subject = "", body = ""] = entry.split("\x00");
    const parentList = parents.split(" ").filter(Boolean);
    const merge = subject.match(/^Merge pull request #(\d+)/);
    const squash = subject.match(/\(#(\d+)\)\s*$/);
    let commits = [];
    if (parentList.length > 1) {
      commits = runGit("log", "--no-merges", "--pretty=format:%s", `${parentList[0]}..${parentList[1]}`)
        .split("\n")
        .filter(Boolean)
        .slice(0, INNER_COMMITS_MAX);
    }
    changes.push({
      hash: hash.slice(0, 7),
      subject,
      body: parentList.length > 1 ? "" : stripTrailers(body),
      pr: merge?.[1] ?? squash?.[1] ?? null,
      commits,
    });
  }
  return changes;
}

// Drop "Co-Authored-By:"-style trailers; they are noise for classification.
function stripTrailers(body) {
  return body.replace(/^[A-Za-z-]+: .*$/gm, "").trim();
}

function getPrDescription(pr) {
  try {
    const out = execFileSync("gh", ["pr", "view", pr, "--json", "title,body", "-q", '.title + "\\n\\n" + .body'], {
      encoding: "utf8",
      timeout: 30_000,
      stdio: ["ignore", "pipe", "pipe"],
    });
    return out.trim().slice(0, PR_DESCRIPTION_MAX_CHARS);
  } catch (err) {
    if (err.code === "ENOENT") {
      throw new Error("gh CLI not found. PR descriptions are needed to classify changes.");
    }
    stderr.write(`No PR found for #${pr}; continuing.\n`);
    return null;
  }
}

function buildChangeBlock(change) {
  const lines = [];
  if (change.pr && !NO_ENRICHMENT_SUBJECT.test(change.subject)) {
    const description = getPrDescription(change.pr);
    lines.push(`PR #${change.pr} (merge ${change.hash})`);
    if (description) lines.push(`  Title and description:\n${indent(description)}`);
  } else {
    lines.push(`COMMIT ${change.hash}: ${change.subject}`);
    if (change.body) lines.push(`  Body: ${change.body.slice(0, COMMIT_BODY_MAX_CHARS)}`);
  }
  if (change.commits.length) {
    lines.push("  Commits:");
    for (const c of change.commits) lines.push(`    - ${c}`);
  }
  lines.push("");
  return lines.join("\n");
}

function indent(text) {
  return text
    .split("\n")
    .map((l) => `    ${l}`)
    .join("\n");
}

function buildContext({ repoName, prevTag, newTag, changes }) {
  const sections = [
    `# Release context for ${repoName} ${newTag}`,
    "",
    `Range: ${prevTag}..${newTag} (${changes.length} changes on main)`,
    "",
  ];
  const readme = getReadme();
  if (readme) sections.push("## What this project is", "", "<readme>", readme, "</readme>", "");
  sections.push("## Changes in this release", "", changes.map(buildChangeBlock).join("\n"));
  return sections.join("\n");
}

// Fail loudly on bad entries rather than silently dropping them.
function parseEntries(raw, sourceLabel) {
  let parsed;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`${sourceLabel} is not valid JSON: ${err.message}`);
  }
  const entries = Array.isArray(parsed) ? parsed : parsed?.entries;
  if (!Array.isArray(entries)) {
    throw new Error(`${sourceLabel} must contain an "entries" array. Got: ${JSON.stringify(parsed).slice(0, 300)}`);
  }
  return entries.map((entry, i) => {
    const where = `${sourceLabel} entry ${i}`;
    if (typeof entry !== "object" || entry === null) throw new Error(`${where} is not an object`);
    if (!CATEGORIES.includes(entry.category)) {
      throw new Error(`${where} has category ${JSON.stringify(entry.category)}; expected one of ${CATEGORIES.join(", ")}`);
    }
    const description = String(entry.description ?? "").trim();
    if (!description) throw new Error(`${where} has an empty description`);
    return { category: entry.category, description, reference: String(entry.reference ?? "").trim() };
  });
}

function renderMarkdown(entries, repo, prevTag, newTag) {
  const lines = [`## What's Changed in ${newTag}`, ""];
  let any = false;
  for (const key of CATEGORIES) {
    const items = entries.filter((e) => e.category === key);
    if (!items.length) continue;
    any = true;
    lines.push(`### ${SECTION_TITLES[key]}`);
    for (const item of items) {
      // Drop a trailing inline PR ref so it isn't doubled by the reference field.
      const description = item.description.replace(/\s*\((?:#\d+|`?[0-9a-f]{7,40}`?)\)\s*\.?\s*$/i, "").trim();
      lines.push(`- ${description}${item.reference ? ` ${item.reference}` : ""}`);
    }
    lines.push("");
  }
  if (!any) lines.push("_No user-visible changes in this release._", "");
  if (repo && prevTag) lines.push(`**Full Changelog**: https://github.com/${repo}/compare/${prevTag}...${newTag}`);
  return lines.join("\n").replace(/\s+$/, "") + "\n";
}

function parseArgs(args) {
  const opts = { since: null, entries: null, repo: env.GITHUB_REPOSITORY ?? "" };
  const positional = [];
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--since") opts.since = args[++i];
    else if (args[i] === "--repo") opts.repo = args[++i];
    else if (args[i] === "--entries") opts.entries = args[++i];
    else positional.push(args[i]);
  }
  [opts.command, opts.tag] = positional;
  if (!["context", "render"].includes(opts.command) || !opts.tag || (opts.command === "render" && !opts.entries)) {
    stderr.write(
      "usage:\n  release-notes.mjs context <tag> [--since TAG]\n  release-notes.mjs render <tag> --entries FILE [--since TAG]\n",
    );
    exit(2);
  }
  return opts;
}

function main() {
  const args = parseArgs(argv.slice(2));
  const newTag = args.tag.startsWith("v") ? args.tag : `v${args.tag}`;
  const prevTag = args.since || getPreviousTag(newTag);
  const repoName = args.repo ? args.repo.split("/").pop() : "snappea";

  if (args.command === "render") {
    const entries = parseEntries(readFileSync(args.entries, "utf8"), args.entries);
    stderr.write(`Rendering ${entries.length} entries.\n`);
    stdout.write(renderMarkdown(entries, args.repo, prevTag, newTag));
    return;
  }

  let end = newTag;
  try {
    runGit("rev-parse", "--verify", "-q", `${newTag}^{commit}`);
  } catch {
    end = "HEAD"; // tag not created yet
  }
  // With no previous tag, cover the whole history.
  const rangeSpec = prevTag ? `${prevTag}..${end}` : end;
  const changes = getChanges(rangeSpec);
  stderr.write(`Found ${changes.length} changes in ${rangeSpec}.\n`);
  stdout.write(buildContext({ repoName, prevTag: prevTag ?? "(start)", newTag, changes }));
}

try {
  main();
} catch (err) {
  stderr.write(`${err.stack ?? err.message ?? err}\n`);
  exit(1);
}
