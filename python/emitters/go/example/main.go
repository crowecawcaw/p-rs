// A stand-in for a Go service started by a Python test.
package main

import (
	"os"

	"github.com/crowecawcaw/p-rs/python/emitters/go/prsemit"
)

type client struct {
	Client string `json:"client"`
}

func main() {
	prsemit.Emit("eEnter", "L1", client{Client: os.Args[1]})
}
