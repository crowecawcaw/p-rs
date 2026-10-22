// Docs overview: what P is, for developers who know unit tests.
import { REPO, links } from "../SiteHeader";
import { Code } from "./Code";
import { Loops } from "./diagrams/Loops";
import { Schedules } from "./diagrams/Schedules";
import { Trace } from "./diagrams/Trace";
import { Workflow } from "./diagrams/Workflow";

const BLOB = `${REPO}/blob/main`;

export function Overview({ root }: { root: string }) {
  const l = links(root);
  const DEMO = `${l.playground}?demo=lost-update`;
  return (
    <>
      <section className="hero">
        <p className="eyebrow">Docs</p>
        <h1>Overview</h1>
        <p className="subtitle">What P is, and how it differs from the tests you already write.</p>
        <p className="lede">
          Two clients each read a shared counter, add one and write it back. Your unit tests pass because they
          run the calls in one fixed order, but in production both clients can read 0, and one update is lost. P
          finds that ordering in milliseconds, from a short model of the design, before any service code exists.
        </p>
      </section>

      <section>
        <h2>A model, a spec and your code</h2>
        <p>
          P is a language for writing small, runnable descriptions of a system's design. You use it to check two
          kinds of thing:
        </p>
        <Loops />
        <dl className="faq">
          <div>
            <dt>Is P my code?</dt>
            <dd>
              No. A <b>model</b> describes your design: the components, the messages they send, and what each does
              when a message arrives. It's usually a few hundred lines, and it runs only inside the checker.
            </dd>
          </div>
          <div>
            <dt>Does it check my code?</dt>
            <dd>
              The checker runs the model, not your code. A <b>spec</b> is a rule written in P, like "stock never goes
              negative". The same spec checks the model at design time and, through events your tests record, your
              code at test time.
            </dd>
          </div>
          <div>
            <dt>How does a model help?</dt>
            <dd>
              It finds bugs that depend on timing: races, reordered or lost messages, retries and crashes. Unit tests
              rarely hit these bugs, and they cost the most to fix once the code is written.
            </dd>
          </div>
        </dl>
      </section>

      <section>
        <h2>How it differs from a unit test</h2>
        <p>
          In a unit test, you pick one scenario and its order of events. In a P test, you describe the setup, and
          the checker picks the order: which message arrives first and which component runs next. It runs thousands
          of orderings and stops at the first one that breaks a rule.
        </p>
        <Schedules />
        <table className="compare">
          <thead>
            <tr>
              <th />
              <th>Unit test</th>
              <th>P model check</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <th>Runs</th>
              <td>your code</td>
              <td>a model of your design</td>
            </tr>
            <tr>
              <th>Who picks the order</th>
              <td>you, once</td>
              <td>the checker, thousands of times</td>
            </tr>
            <tr>
              <th>Finds</th>
              <td>logic bugs on the path you wrote</td>
              <td>bugs that need an unlucky ordering</td>
            </tr>
            <tr>
              <th>When</th>
              <td>after the code exists</td>
              <td>before it, and alongside it</td>
            </tr>
          </tbody>
        </table>
      </section>

      <section>
        <h2>Worked example: a lost update</h2>
        <p>
          Each <b>machine</b> is a state machine with an inbox. <code>send</code> puts an event in another machine's
          inbox, and the checker decides when it's delivered. Here the Counter serves reads and writes, and each
          Client reads, then writes back the value plus one:
        </p>
        <Code lang="p" title="LostUpdate.p">{`
machine Counter {
var value: int;
start state Serving {
  on eRead do (c: Client) { send c, eValue, value; }
  on eWrite do (v: int) { value = v; }
  on eGet do (m: Main) { send m, eValue, value; }
}
}

machine Client {
start state Run {
  entry (input: (counter: Counter, owner: Main)) {
    var v: int;
    send input.counter, eRead, this;
    receive { case eValue: (x: int) { v = x; } }
    send input.counter, eWrite, v + 1;
    send input.owner, eDone;
  }
}
}
`}</Code>
        <p>
          The test machine, <code>Main</code>, starts the Counter and two Clients. Once both Clients are done, it
          asserts that the counter reached 2:
        </p>
        <Code lang="p">{`
assert final == 2, format("expected 2 increments, counter is {0}", final);

test tcTwoClients [main=Main]: { Main, Counter, Client };
`}</Code>
        <p>Check it:</p>
        <Code lang="shell">{`
$ snappea check LostUpdate.p tcTwoClients
Checking generatedOutput: 1 test case
FAIL  tcTwoClients (bug found after 9 schedules, 8 workers, 0.00s)
      Assertion Failed: LostUpdate.p:68:7 expected 2 increments, counter is 1
      Replay: snappea replay PCheckerOutput/tcTwoClients/bug.schedule LostUpdate.p
1 of 1 test case failed in 0.01s, peak memory 9 MB. Output: PCheckerOutput/
`}</Code>
        <p>The trace shows exactly how the update was lost:</p>
        <Trace />
        <p>
          <b>The fix is a design change:</b> replace the read and write with one <code>eIncrement</code> event that
          the Counter applies itself. With that change, <code>snappea check --exhaustive</code> covers every possible
          ordering (191 schedules) and finds no bug.
        </p>
        <Code lang="p">{`
on eIncrement do { value = value + 1; }
`}</Code>
        <p>
          <a className="button primary" href={DEMO}>Open this model in the Playground →</a>
        </p>
      </section>

      <section>
        <h2>The same rules, checking your code</h2>
        <p>
          A spec <b>observes</b> events and asserts rules about them. This one checks an inventory service. Specs
          check models in the same way.
        </p>
        <Code lang="p" title="inventory.p (excerpt)">{`
spec InventoryCorrect observes eStocked, eReserved, eReleased {
var available: int;
state Ready {
  on eReserved do (p: (order: string, units: int)) {
    available = available - p.units;
    assert available >= 0, "reservation exceeds available stock";
  }
  // ...
}
}
`}</Code>
        <p>Your code emits the events after each change succeeds, and your ordinary tests check them:</p>
        <Code lang="python" title="Python">{`
def reserve(self, sku, order, units):
    ...
    emit("eReserved", key=sku, order=order, units=units)

def test_release_makes_stock_available_again(inventory_factory):
    with check_spec("InventoryCorrect"):
        inventory = inventory_factory()
        inventory.reserve("coffee", "order-1", 3)
        inventory.release("coffee", "order-1")
`}</Code>
        <p>
          There are helpers for <a href={l.python}>Python</a>, <a href={l.rust}>Rust</a>,{" "}
          <a href={l.java}>Java</a> and <a href={l.go}>Go</a>. To check events from running services, see{" "}
          <a href={`${BLOB}/examples/multiprocess/docker/README.md`}>live observation across processes</a> and{" "}
          <a href={`${BLOB}/docs/observe.md`}>snappea observe</a>. The same emit calls can write production logs
          for <code>snappea observe</code>: see <a href={`${BLOB}/docs/production.md`}>testing and production</a>.
        </p>
      </section>

      <section>
        <h2>Where P fits</h2>
        <Workflow />
        <p>
          AI agents are well suited to writing models: they can turn a design doc into machines and specs quickly,
          and rerun the checker after every change. Your part is to read the specs. They are the short, plain
          statement of what must always be true, and the part of the model worth reviewing line by line.
        </p>
      </section>

      <section className="next">
        <h2>Next</h2>
        <ul>
          <li>
            <a href={DEMO}>Run the lost-update model</a> in the Playground, then try the bigger tutorials in its menu.
          </li>
          <li>
            <a href={`${BLOB}/examples/README.md`}>Integration examples</a>: specs checking real code in four
            languages.
          </li>
          <li>
            <a href="https://p-org.github.io/P/" target="_blank" rel="noreferrer">The P language manual</a>, from the
            upstream project.
          </li>
        </ul>
      </section>
    </>
  );
}
