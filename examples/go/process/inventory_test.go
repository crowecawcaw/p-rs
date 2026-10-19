// Tests for code that emits with prsemit, as cmd/inventory does: in the test process,
// in processes the test starts (r.Env), and in production (a log file).
package process_test

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"example.com/inventory/internal/fixture"
	"example.com/inventory/inventory"
	"github.com/crowecawcaw/p-rs/go/prsemit"
)

var binary string

// TestMain builds cmd/inventory once for the package's tests.
func TestMain(m *testing.M) {
	dir, err := os.MkdirTemp("", "inventory-")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	binary = filepath.Join(dir, "inventory")
	if runtime.GOOS == "windows" {
		binary += ".exe"
	}
	if out, err := exec.Command("go", "build", "-o", binary, "../cmd/inventory").CombinedOutput(); err != nil {
		fmt.Fprintf(os.Stderr, "build cmd/inventory: %v\n%s", err, out)
		os.Exit(1)
	}
	code := m.Run()
	os.RemoveAll(dir)
	os.Exit(code)
}

// prsemit.Emit in the test process reaches the test's recording. It cannot tell
// parallel tests apart, so this test is not parallel (or pass contexts from
// r.Context to prsemit.EmitCtx).
func TestInProcess(t *testing.T) {
	fixture.Start(t)
	app := inventory.New(prsemit.Emit)
	if err := app.Stock("coffee", 3); err != nil {
		t.Fatal(err)
	}
	if err := app.Reserve("coffee", "order-1", 3); err != nil {
		t.Fatal(err)
	}
}

// Two worker processes, one per product, write into the test's recording. Waiting for
// them before the test returns means the check (at cleanup) sees every event.
func TestWorkerProcesses(t *testing.T) {
	t.Parallel()
	r := fixture.Start(t)
	workers := map[string]string{
		"coffee": "stock coffee 3\nreserve coffee order-1 2\nreserve coffee order-2 2\nrelease coffee order-1\n",
		"tea":    "stock tea 2\nreserve tea order-1 2\n",
	}
	var cmds []*exec.Cmd
	var stderr []*strings.Builder
	for _, ops := range workers {
		cmd := exec.Command(binary)
		cmd.Env = r.Env() // PRS_EVENTS_DIR: the worker writes into this recording
		cmd.Stdin = strings.NewReader(ops)
		out := &strings.Builder{}
		cmd.Stderr = out
		if err := cmd.Start(); err != nil {
			t.Fatal(err)
		}
		cmds, stderr = append(cmds, cmd), append(stderr, out)
	}
	for n, cmd := range cmds {
		if err := cmd.Wait(); err != nil {
			t.Fatalf("worker: %v\n%s", err, stderr[n])
		}
	}
	if all := stderr[0].String() + stderr[1].String(); !strings.Contains(all, "order-2") {
		t.Fatalf("want coffee order-2 rejected; worker output:\n%s", all)
	}
}

// In production the same instrumentation writes a log for `prs observe`. AddLog checks
// such a log from a test.
func TestProductionLog(t *testing.T) {
	t.Parallel()
	log := filepath.Join(t.TempDir(), "events.jsonl")
	cmd := exec.Command(binary, "-events", log)
	cmd.Stdin = strings.NewReader("stock coffee 3\nreserve coffee order-1 3\nrelease coffee order-1\n")
	if out, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("inventory: %v\n%s", err, out)
	}
	r := fixture.New(t)
	r.AddLog(log, nil)
	r.Check(t)
}
