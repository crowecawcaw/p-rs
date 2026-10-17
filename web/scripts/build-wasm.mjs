// Builds crates/prs-wasm for wasm32 and copies the module to src/generated/.
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const web = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(web, "..");
execFileSync(
  "cargo",
  ["build", "--profile", "wasm", "--target", "wasm32-unknown-unknown", "-p", "prs-wasm"],
  { cwd: root, stdio: "inherit" },
);
mkdirSync(join(web, "src/generated"), { recursive: true });
copyFileSync(
  join(root, "target/wasm32-unknown-unknown/wasm/prs_wasm.wasm"),
  join(web, "src/generated/prs.wasm"),
);
console.log("wasm -> src/generated/prs.wasm");
