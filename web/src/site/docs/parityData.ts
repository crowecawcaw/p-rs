// Build-time inventory of the same fixtures read by the compatibility tests.
// Read manifests and rules, but only enumerate output paths (some traces are large).
const ROOT = "../../../../tests/upstream/";
const manifests = import.meta.glob<{ steps: { name: string; exit_code: number }[] }>(
  "../../../../tests/upstream/goldens/cases/**/case.json", { eager: true, import: "default" },
);
const outputLists = import.meta.glob<string>(
  "../../../../tests/upstream/goldens/cases/**/outputs.txt", { eager: true, query: "?raw", import: "default" },
);
const contentPaths = new Set(Object.keys(import.meta.glob("../../../../tests/upstream/goldens/cases/**/files/**/*", { query: "?raw", import: "default" })));
const observePaths = Object.keys(import.meta.glob("../../../../tests/upstream/PObserve/expected/**/*", { query: "?raw", import: "default" }));
import pin from "../../../../tests/upstream/goldens/UPSTREAM?raw";
import rules from "../../../../tests/upstream/goldens/normalize.tsv?raw";
import checkKnown from "../../../../tests/upstream/goldens/known-differences.tsv?raw";
import observeKnown from "../../../../tests/upstream/PObserve/known-differences.tsv?raw";

function rows(text: string) {
  return text.split("\n").filter((line) => line && !line.startsWith("#")).map((line) => line.split("\t"));
}
export const upstreamPin = pin.trim();
export const normalizations = rows(rules);
export const checkDifferences = rows(checkKnown);
export const observeDifferences = rows(observeKnown);
const checkIds = new Set(checkDifferences.map(([id]) => id));
const observeIds = new Set(observeDifferences.map(([id]) => id));
const casePrefix = `${ROOT}goldens/cases/`;

export const checkAreas = ["regression", "tutorial", "cli", "options"].map((area) => {
  let cases = 0, identical = 0, runs = 0, files = 0, compared = 0, writtenOnly = 0, traces = 0;
  for (const [path, manifest] of Object.entries(manifests)) {
    const id = path.slice(casePrefix.length, -"/case.json".length);
    if (!id.startsWith(`${area}/`)) continue;
    cases++;
    if (!checkIds.has(id)) identical++;
    runs += manifest.steps.length;
    const outputs = outputLists[path.replace(/case.json$/, "outputs.txt")];
    if (outputs === undefined) throw new Error(`Missing output list for ${id}`);
    for (const name of outputs.split("\n").filter(Boolean)) {
      files++;
      const file = path.replace(/case.json$/, `files/${name}`);
      if (contentPaths.has(file) || contentPaths.has(`${file}.sha256`)) compared++;
      else if (name.endsWith(".sci")) writtenOnly++;
      else throw new Error(`Missing parity fixture: ${file}`);
      if (/\.trace\.(json|xml)$/.test(name) || /_\d+_\d+\.txt$/.test(name)) traces++;
    }
  }
  return { area, cases, identical, runs, files, compared, writtenOnly, traces };
});

const observePrefix = `${ROOT}PObserve/expected/`;
export const observeAreas = ["happy-logs", "error-logs"].map((area) => {
  const paths = observePaths.filter((path) => path.startsWith(`${observePrefix}${area}/`));
  const runs = paths.filter((path) => path.endsWith("/exit_code"));
  const comparedPaths = paths.filter((path) => !observeIds.has(path.split("/").at(-2)!));
  return {
    area, runs: runs.length,
    identical: comparedPaths.filter((path) => path.endsWith("/exit_code")).length,
    files: paths.filter((path) => !/\/(exit_code|console.txt)$/.test(path)).length,
    compared: comparedPaths.filter((path) => !/\/(exit_code|console.txt)$/.test(path)).length,
    replays: comparedPaths.filter((path) => /\/replayEvents_.*\.log$/.test(path)).length,
  };
});
export const checkTotals = checkAreas.reduce((sum, a) => ({ cases: sum.cases + a.cases, identical: sum.identical + a.identical }), { cases: 0, identical: 0 });
export const observeTotals = observeAreas.reduce((sum, a) => ({ runs: sum.runs + a.runs, identical: sum.identical + a.identical }), { runs: 0, identical: 0 });
