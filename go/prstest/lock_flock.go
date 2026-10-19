//go:build darwin || dragonfly || freebsd || linux || netbsd || openbsd

package prstest

import (
	"os"
	"syscall"
)

// lockFile takes an exclusive lock on path, waiting for other processes.
func lockFile(path string) (func(), error) {
	f, err := os.OpenFile(path, os.O_CREATE|os.O_RDWR, 0o644)
	if err != nil {
		return nil, err
	}
	for {
		err = syscall.Flock(int(f.Fd()), syscall.LOCK_EX)
		if err != syscall.EINTR {
			break
		}
	}
	if err != nil {
		f.Close()
		return nil, err
	}
	return func() { f.Close() }, nil
}
