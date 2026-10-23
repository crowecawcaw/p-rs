import { REPO, links } from "../SiteHeader";
import { Code } from "./Code";

export function Coverage({ root }: { root: string }) {
  const l = links(root);
  return (
    <>
      <section className="hero">
        <p className="eyebrow">Docs</p>
        <h1>Coverage</h1>
        <p className="subtitle">See what your checks exercised and where to improve the model or workload.</p>
        <p>
          <code>snappea check</code> reports coverage for each test and for the selected tests together.
          <code> snappea observe</code> measures the monitors exercised by your application events.
          A passing check and high coverage are useful evidence; neither establishes that every requirement
          is modeled or that the model matches your application.
        </p>
      </section>
      <section>
        <h2>What is measured</h2>
        <table className="compare">
          <thead><tr><th>Type</th><th>Question it answers</th></tr></thead>
          <tbody>
            <tr><th>Statements</th><td>Which executable statements ran? Locations are reported as source lines.</td></tr>
            <tr><th>Branches</th><td>Did each <code>if</code> take both its true and false paths, including an empty else?</td></tr>
            <tr><th>Condition outcomes</th><td>Did each atomic condition in an <code>if</code> or <code>while</code> evaluate both true and false? Short-circuited operands count only when evaluated.</td></tr>
            <tr><th>Event handlers</th><td>Did each state's <code>do</code> or <code>goto</code> handler run?</td></tr>
            <tr><th>Transition pairs</th><td>Did each structural path <code>A → B → C</code> occur on one instance in one execution?</td></tr>
            <tr><th>Scenarios</th><td>Did a named behavioral target reach its accepting state? Unwitnessed scenarios remain coverage gaps.</td></tr>
          </tbody>
        </table>
        <p>
          Condition coverage distinguishes operands inside decisions such as <code>ready &amp;&amp; authorized</code>.
          Both branch outcomes can occur while <code>authorized</code> is always true when evaluated.
          This measures operand outcomes, not whether each operand independently changes the decision (MC/DC).
          Literal Boolean constants and assertion conditions are excluded from condition targets.
        </p>
        <p>
          Transition-pair targets come from declared transitions and gotos in actions and called helpers.
          Guards and data constraints can make a structural pair unreachable. Covering two edges in separate
          executions does not cover their pair. Other actors may run between the two transitions.
        </p>
      </section>
      <section>
        <h2>Use the gaps</h2>
        <Code lang="shell">{`
snappea check
snappea check tcRecovery --json-report coverage.json
`}</Code>
        <p>
          Uncovered locations identify missing driver actions, inputs, or event orderings worth investigating.
          Components loaded from source but outside the selected tests are listed separately and excluded
          from the totals. They may belong to another test or be checked only through application events.
          Code beginning with <code>assert false</code> is also excluded because it must never run.
        </p>
        <p>
          Counts combine hits across executions; they do not describe one complete execution.
          A bug stops a test early and marks its coverage partial. An incomplete search can miss reachable targets;
          uncovered does not mean impossible. JSON includes the full gap list and the separate
          <code> conditions</code>, <code>transition_pairs</code>, and <code>excluded_components</code> fields.
        </p>
      </section>
      <section>
        <h2>Behavior needs a target</h2>
        <p>
          Full structural coverage can still miss two clients competing for the same resource, or a retry
          reusing a request ID after restart. Express those relationships as scenarios, observing the
          relevant identities and events. A scenario recognizes a challenging situation; a spec checks
          that the system handles it correctly. Reaching a cold state after an observed event satisfies a scenario.
        </p>
        <p>
          Mutation testing complements coverage by changing behavior and asking whether checks reject it.
          It measures sensitivity to those changes, rather than adding a reachability guarantee.
          Missing requirements and inaccurate application instrumentation still need review.
        </p>
        <p>
          See the <a href={l.cli}>command line guide</a>,
          {" "}<a href={`${REPO}/blob/main/docs/checking-models.md`}>checking reference</a>, and
          {" "}<a href={`${REPO}/blob/main/docs/observe.md`}>application-event coverage and merging</a>.
        </p>
      </section>
    </>
  );
}
