/**
 * Parse every GitHub Actions workflow, and check the few things about them that are
 * easy to get wrong and expensive to get wrong.
 *
 * **An unparseable workflow file does not fail CI — it stops CI existing.** GitHub
 * cannot read a `name:` it cannot parse, so the run is listed under the file path,
 * lasts zero seconds, produces no jobs and no log, and reports only "this run likely
 * failed because of a workflow file issue". Nothing in the repository notices, because
 * the thing that would have noticed is the workflow.
 *
 * That is not hypothetical: a `run:` value ending in `diagnostics::` was read as the
 * start of a nested mapping and silently disabled the whole CI workflow.
 *
 * Runs at pre-commit, on staged workflow files — the only place it can run, since by
 * the time it reaches GitHub it is already too late.
 *
 *   node scripts/check-workflows.mjs [paths...]
 */

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";
import { parse } from "yaml";

const DIRECTORY = join(".github", "workflows");

const files =
  process.argv.slice(2).length > 0
    ? process.argv.slice(2)
    : readdirSync(DIRECTORY)
        .filter((file) => file.endsWith(".yml") || file.endsWith(".yaml"))
        .map((file) => join(DIRECTORY, file));

const problems = [];

for (const file of files) {
  let document;

  try {
    document = parse(readFileSync(file, "utf8"));
  } catch (error) {
    problems.push(`${file}: ${error.message.split("\n")[0]}`);
    continue;
  }

  if (typeof document !== "object" || document === null) {
    problems.push(`${file}: not a mapping`);
    continue;
  }

  // Without a name GitHub falls back to the file path everywhere it lists the run,
  // which is also the symptom of a file it could not read at all.
  if (typeof document.name !== "string" || document.name.trim() === "") {
    problems.push(`${file}: no \`name:\``);
  }

  // `on` is the YAML 1.1 boolean `true`, which is why the parsed key may be either.
  if (document.on === undefined && document.true === undefined) {
    problems.push(`${file}: no \`on:\` trigger`);
  }

  if (typeof document.jobs !== "object" || Object.keys(document.jobs ?? {}).length === 0) {
    problems.push(`${file}: no jobs`);
  }
}

if (problems.length > 0) {
  process.stderr.write(`\n  ${problems.length} workflow problem(s):\n`);
  for (const problem of problems) {
    process.stderr.write(`    ${problem}\n`);
  }
  process.stderr.write(
    "\n  A workflow file that does not parse is not a failing check — it is no check\n" +
      "  at all. Fix before committing.\n\n",
  );
  process.exit(1);
}

process.stdout.write(`  ${files.length} workflow(s) parsed.\n`);
