//go:build !(darwin || dragonfly || freebsd || linux || netbsd || openbsd || windows)

package prstest

import "sync"

var processLock sync.Mutex

// lockFile serializes merges within this process only: this platform has no file lock
// in the standard library.
func lockFile(string) (func(), error) {
	processLock.Lock()
	return processLock.Unlock, nil
}
