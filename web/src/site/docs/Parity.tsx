import { REPO } from "../SiteHeader";
import { Code } from "./Code";
import { checkAreas, checkDifferences, checkTotals, normalizations, observeAreas, observeDifferences, observeTotals, upstreamPin } from "./parityData";

const BLOB = `${REPO}/blob/main`;

function ruleReason(pattern: string) {
  if (pattern.includes("Elapsed") || pattern.includes("time_seconds") || pattern.includes("memory_max")) return "Elapsed time and memory depend on the runtime and machine.";
  if (pattern.includes("task")) return "Task IDs come from .NET's thread pool.";
  if (pattern.includes(" at ")) return "Stack frames describe upstream's .NET implementation, not the P model.";
  return "Upstream discovers compiled output and logs compilation progress; Snap Pea reads sources directly.";
}

export function Parity({ root: _root }: { root: string }) {
  return <>
    <section className="hero">
      <p className="eyebrow">Docs</p>
      <h1>Upstream parity</h1>
      <p className="subtitle">The same P model, schedules and observable results.</p>
      <p>Parity means running the same project and test case with the same schedule count and seed,
        on one worker, then getting the same exit code, separate stdout and stderr, and the same
        output files after the fixed normalizations below. This measures <code>snappea-compat</code>;
        the native <code>snappea</code> CLI has its own presentation and defaults.</p>
      <p>The committed check corpus contains <strong>{checkTotals.cases} cases</strong>, of which{" "}
        <strong>{checkTotals.identical} must match</strong> completely after normalization.
        The vendored observe corpus contains <strong>{observeTotals.runs} runs</strong>, of which{" "}
        <strong>{observeTotals.identical} must match</strong> completely after normalization.
        The remaining cases and runs are listed with reasons below.</p>
    </section>

    <section>
      <h2 id="measurement">What is measured</h2>
      <table className="compare ref">
        <thead><tr><th>Level</th><th>Checks</th></tr></thead>
        <tbody>
          <tr><th>Exact</th><td>Exit codes, file lists and history rotation; schedules, coverage and scenario coverage contents.</td></tr>
          <tr><th>Exact after normalization</th><td>Console output, parse and type errors, compiler warnings, readable, JSON and XML traces, summaries, and PObserve output files. Large check files are compared by SHA-256 of normalized contents.</td></tr>
          <tr><th>Written only</th><td><code>.sci</code> files must exist at the correct paths. Their contents are a reduced form and are not compared.</td></tr>
          <tr><th>Not compared</th><td>Generated C#/Java and binaries, version strings, multiworker runs, and the exclusions below.</td></tr>
        </tbody>
      </table>
      <p>The check corpus was recorded from upstream P at <a href={`https://github.com/p-org/P/commit/${upstreamPin}`}><code>{upstreamPin}</code></a>,
        pinned in <a href={`${BLOB}/tests/upstream/goldens/UPSTREAM`}>UPSTREAM</a>.
        CI runs Snap Pea against those frozen outputs; it does not need to run P.</p>
    </section>

    <section>
      <h2 id="results">Results by area</h2>
      <p>These inventories are computed at site build time from the committed manifests, output lists,
        file paths and known-difference lists. “Match” is the assertion enforced by passing compatibility CI,
        not a new run performed by the docs build. A run here is one recorded command, including usage errors
        and test listings; it is not one explored schedule.</p>
      <div className="parity-table"><table className="compare ref">
        <caption>Check corpus</caption>
        <thead><tr><th>Area</th><th>Cases</th><th>Match</th><th>Runs</th><th>Files</th><th>Contents compared</th><th>Written only</th><th>Bug trace files</th></tr></thead>
        <tbody>{checkAreas.map((a) => <tr key={a.area}><th>{a.area}</th><td>{a.cases}</td><td>{a.identical}</td><td>{a.runs}</td><td>{a.files}</td><td>{a.compared}</td><td>{a.writtenOnly}</td><td>{a.traces}</td></tr>)}</tbody>
      </table></div>
      <p>Regression cases cover static errors and bounded checks; tutorials list and check each test case.
        CLI and option cases cover help, errors, replay, output paths, rotation, coverage merging,
        strategies and step limits. Bug trace files count readable, JSON and XML traces,
        including history copies. Schedules are also included in the file-content comparisons.
        Files are counted at their final paths per case, not once per command that writes them.
        All content assertions must pass outside the known-difference cases; individual matching-file
        counts inside those cases are not recorded.</p>
      <div className="parity-table"><table className="compare ref">
        <caption>Vendored PObserve corpus checked in CI</caption>
        <thead><tr><th>Area</th><th>Runs</th><th>Match</th><th>Files recorded</th><th>Files compared</th><th>Replay logs compared</th></tr></thead>
        <tbody>{observeAreas.map((a) => <tr key={a.area}><th>{a.area}</th><td>{a.runs}</td><td>{a.identical}</td><td>{a.files}</td><td>{a.compared}</td><td>{a.replays}</td></tr>)}</tbody>
      </table></div>
      <p>Observe file counts exclude the recorded console and exit-code metadata. Each log is run with
        and without <code>--assumeInputsAreSorted</code>. For the known observe differences, CI checks
        only the exit code; console and files are skipped entirely.</p>
      <p>The separate, manually checked full upstream log suite has 192 runs: all exit codes match,
        186 outputs match after normalization, and six differ in the null-dereference message.
        That historical measurement comes from <a href={`${BLOB}/scripts/compare-pobserve.py`}>compare-pobserve.py</a>,
        not the vendored CI inventory above. The vendor script selects logs smaller than 8 KB.</p>
    </section>

    <section>
      <h2 id="normalization">Every normalization</h2>
      <p>The recorder and Rust check test read the same <a href={`${BLOB}/tests/upstream/goldens/normalize.tsv`}>normalize.tsv</a>.
        A “drop” removes matching console lines only; a “mask” replaces matches in both console and files.
        The following table is read directly from that file.</p>
      <div className="parity-table"><table className="compare ref">
        <thead><tr><th>Rule</th><th>Pattern</th><th>Replacement</th><th>Reason</th></tr></thead>
        <tbody>{normalizations.map(([kind, pattern, replacement], i) => <tr key={i}><th>{kind}</th><td><code>{pattern}</code></td><td><code>{kind === "drop" ? "whole line removed" : replacement || "empty"}</code></td><td>{ruleReason(pattern)}</td></tr>)}</tbody>
      </table></div>
      <p>In addition, both sides remove a leading UTF-8 byte order mark and replace the working directory
        with <code>&lt;dir&gt;</code>: encoding markers and temporary paths do not describe model behavior.
        The recorder moves compile warnings to the beginning of check output, because Snap Pea compiles
        when checking. No other check output is ignored.</p>
      <h3>PObserve normalization</h3>
      <ul>
        <li>Strip ANSI colors and replace the output directory with <code>&lt;out&gt;</code> to remove terminal styling and temporary paths.</li>
        <li>Remove configuration lines for <code>outputDir</code>, <code>supplierJars</code>, <code>pproj</code> and <code>pfiles</code>, since the tools load specs differently.</li>
        <li>Mask total runtime and per-event average and maximum times in metrics. Verified event counts remain exact.</li>
        <li>Remove Java stack frames from parser and unknown-error files, since they describe generated Java rather than the P spec.</li>
        <li>Sort console error blocks and mask their numbered filenames; sort error-file contents and renumber files. Upstream checks keys in parallel, so error order and numbering vary.</li>
      </ul>
      <p>The executable definitions are in <a href={`${BLOB}/crates/snappea/tests/compat_observe_upstream.rs`}>compat_observe_upstream.rs</a>
        {" "}and <a href={`${BLOB}/scripts/compare-pobserve.py`}>compare-pobserve.py</a>.</p>
    </section>

    <section>
      <h2 id="known-differences">Known differences</h2>
      <p>Check cases still compare every recorded field, but are expected to differ. The test fails if a
        listed case starts matching, prompting removal of its exception.</p>
      <div className="parity-table"><table className="compare ref"><thead><tr><th>Check case</th><th>Reason</th></tr></thead>
        <tbody>{checkDifferences.map(([id, reason]) => <tr key={id}><th><code>{id}</code></th><td>{reason}</td></tr>)}</tbody></table></div>
      <p>These observe runs compare only exit codes. The null-dereference message refers to upstream's generated Java variables.</p>
      <div className="parity-table"><table className="compare ref"><thead><tr><th>Observe run</th><th>Reason</th></tr></thead>
        <tbody>{observeDifferences.map(([id, reason]) => <tr key={id}><th><code>{id}</code></th><td>{reason}</td></tr>)}</tbody></table></div>
    </section>

    <section>
      <h2 id="limits">What this does not cover</h2>
      <p>Finite schedule counts and step bounds measure the recorded executions, not every possible
        execution of every model. Strategies are sampled by the option cases. PEx mode, the PVerifier
        tutorials, the PriorityQueue tutorial with C# foreign code, and
        <code> Liveness/DynamicError/InfiniteLoopInAtomicBlock</code> are not recorded; upstream hangs on
        the last one. Multiworker schedule ordering, generated code, C#/Java foreign code, and .NET/Java
        stack frames are outside this comparison. Timings and memory are not performance assertions.</p>
      <p>See the <a href={`${BLOB}/docs/compatibility.md`}>compatibility reference</a> for supported features
        and the <a href={`${BLOB}/docs/development.md#validation`}>validation guide</a> for broader compiler,
        checker and tutorial tests.</p>
    </section>

    <section>
      <h2 id="reproduce">Reproduce the comparison</h2>
      <p>These commands run Snap Pea against the committed upstream outputs, without rebuilding upstream:</p>
      <Code lang="shell">{`cargo test -p snappea --locked --test compat_check_upstream --test compat_observe_upstream
# Optional: restrict the check corpus by case ID prefix
UPSTREAM_CASES=cli/ cargo test -p snappea --locked --test compat_check_upstream`}</Code>
      <p>Read each <code>case.json</code> under <a href={`${BLOB}/tests/upstream/goldens/cases`}>goldens/cases</a>
        {" "}for the exact upstream and Snap Pea arguments, exit codes, source directory and command order.
        Adjacent files contain separate stdout/stderr, output paths and normalized file contents.</p>
      <p>To independently reproduce the upstream recordings, follow the
        {" "}<a href={`${BLOB}/docs/compatibility.md#how-compatibility-is-checked`}>recording instructions</a>
        {" "}or manually run <a href={`${REPO}/actions/workflows/upstream-goldens.yml`}>Upstream goldens</a>
        {" "}at the pinned commit. Its weekly run compares upstream master with the pin.
        Observe recording uses <a href={`${BLOB}/scripts/vendor-pobserve-goldens.py`}>vendor-pobserve-goldens.py</a>;
        the full manual comparison uses the environment and command documented in
        {" "}<a href={`${BLOB}/scripts/compare-pobserve.py`}>compare-pobserve.py</a>.</p>
    </section>
  </>;
}
