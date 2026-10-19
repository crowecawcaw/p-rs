// Command inventory runs inventory operations read from stdin, one per line:
//
//	stock coffee 3
//	reserve coffee order-1 2
//	release coffee order-1
//
// It emits events with prsemit: into the test's recording when a test starts it
// (PRS_EVENTS_DIR), to the -events file in production, and nowhere otherwise.
package main

import (
	"bufio"
	"flag"
	"fmt"
	"os"
	"strconv"
	"strings"

	"example.com/inventory/inventory"
	"github.com/crowecawcaw/p-rs/go/prsemit"
)

func main() {
	events := flag.String("events", "", "append events as JSON lines to this file, for prs observe")
	flag.Parse()
	if *events != "" {
		if err := prsemit.OpenFile(*events); err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
	}
	defer prsemit.Close()

	app := inventory.New(prsemit.Emit)
	in := bufio.NewScanner(os.Stdin)
	for in.Scan() {
		f := strings.Fields(in.Text())
		var err error
		switch {
		case len(f) == 3 && f[0] == "stock":
			err = app.Stock(f[1], atoi(f[2]))
		case len(f) == 4 && f[0] == "reserve":
			err = app.Reserve(f[1], f[2], atoi(f[3]))
		case len(f) == 3 && f[0] == "release":
			err = app.Release(f[1], f[2])
		case len(f) == 0:
		default:
			err = fmt.Errorf("unknown operation %q", in.Text())
		}
		if err != nil {
			fmt.Fprintf(os.Stderr, "%s: %v\n", in.Text(), err)
		}
	}
}

func atoi(s string) int {
	n, err := strconv.Atoi(s)
	if err != nil {
		return -1
	}
	return n
}
