//go:build windows

package snappeatest

import (
	"os"
	"syscall"
	"unsafe"
)

var procLockFileEx = syscall.NewLazyDLL("kernel32.dll").NewProc("LockFileEx")

// lockFile takes an exclusive lock on path, waiting for other processes.
func lockFile(path string) (func(), error) {
	f, err := os.OpenFile(path, os.O_CREATE|os.O_RDWR, 0o644)
	if err != nil {
		return nil, err
	}
	const lockfileExclusiveLock = 2
	var overlapped syscall.Overlapped
	r1, _, e := procLockFileEx.Call(f.Fd(), lockfileExclusiveLock, 0, 1, 0, uintptr(unsafe.Pointer(&overlapped)))
	if r1 == 0 {
		f.Close()
		return nil, e
	}
	return func() { f.Close() }, nil
}
