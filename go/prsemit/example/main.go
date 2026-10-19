// A stand-in for a Go service started by a test (python/tests/test_other_languages.py).
package main

import (
	"os"

	"github.com/crowecawcaw/p-rs/go/prsemit"
)

type client struct {
	Client string `json:"client"`
}

func main() {
	prsemit.Emit("eEnter", "L1", client{Client: os.Args[1]})
}
