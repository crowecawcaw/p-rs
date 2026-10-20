package snappeabin

import (
	"archive/tar"
	"archive/zip"
	"bytes"
	"compress/gzip"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
)

const payload = "#!/bin/sh\necho snappea 9.9.9\n"

func tarGz(t *testing.T, name, body string) []byte {
	t.Helper()
	var buf bytes.Buffer
	gz := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gz)
	for _, f := range []struct{ name, body string }{{"README.md", "readme"}, {name, body}} {
		if err := tw.WriteHeader(&tar.Header{Name: f.name, Mode: 0o644, Size: int64(len(f.body)), Typeflag: tar.TypeReg}); err != nil {
			t.Fatal(err)
		}
		tw.Write([]byte(f.body))
	}
	tw.Close()
	gz.Close()
	return buf.Bytes()
}

func zipped(t *testing.T, name, body string) []byte {
	t.Helper()
	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)
	w, err := zw.Create(name)
	if err != nil {
		t.Fatal(err)
	}
	w.Write([]byte(body))
	zw.Close()
	return buf.Bytes()
}

func sum(b []byte) string {
	h := sha256.Sum256(b)
	return hex.EncodeToString(h[:])
}

// release serves assets under /v1.2.3/ and counts requests.
func release(t *testing.T, assets map[string][]byte) (*httptest.Server, *atomic.Int32) {
	t.Helper()
	var hits atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		hits.Add(1)
		body, ok := assets[strings.TrimPrefix(r.URL.Path, "/v1.2.3/")]
		if !ok {
			http.NotFound(w, r)
			return
		}
		w.Write(body)
	}))
	t.Cleanup(srv.Close)
	return srv, &hits
}

func source(t *testing.T, base, sums, goos string) Source {
	return Source{Version: "1.2.3", BaseURL: base, Sums: sums, CacheDir: t.TempDir(), GOOS: goos, GOARCH: "arm64"}
}

func TestDownloadVerifyExtractCache(t *testing.T) {
	t.Setenv("SNAPPEA_DOWNLOAD", "")
	archive := tarGz(t, "snappea-v1.2.3-aarch64-apple-darwin/snappea", payload)
	srv, hits := release(t, map[string][]byte{"snappea-v1.2.3-aarch64-apple-darwin.tar.gz": archive})
	sums := fmt.Sprintf("%s  snappea-v1.2.3-x86_64-pc-windows-msvc.zip\n%s  snappea-v1.2.3-aarch64-apple-darwin.tar.gz\n", strings.Repeat("0", 64), sum(archive))
	s := source(t, srv.URL, sums, "darwin")

	// Parallel callers (and processes, through rename) end with one good binary.
	var wg sync.WaitGroup
	paths := make([]string, 8)
	errs := make([]error, 8)
	for i := range paths {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			paths[i], errs[i] = s.Get(context.Background())
		}(i)
	}
	wg.Wait()
	for i := range paths {
		if errs[i] != nil || paths[i] != filepath.Join(s.CacheDir, "1.2.3", "snappea") {
			t.Fatalf("Get: %q %v", paths[i], errs[i])
		}
	}
	got, err := os.ReadFile(paths[0])
	if err != nil || string(got) != payload {
		t.Fatalf("extracted %q %v", got, err)
	}
	if info, _ := os.Stat(paths[0]); info.Mode().Perm()&0o100 == 0 {
		t.Fatalf("not executable: %v", info.Mode())
	}
	entries, _ := os.ReadDir(filepath.Dir(paths[0]))
	if len(entries) != 1 {
		t.Fatalf("temporary files left behind: %v", entries)
	}
	if hits.Load() != 1 {
		t.Fatalf("downloaded %d times", hits.Load())
	}

	// The cache wins, even with downloading off.
	srv.Close()
	t.Setenv("SNAPPEA_DOWNLOAD", "0")
	if p, err := s.Get(context.Background()); err != nil || p != paths[0] {
		t.Fatalf("cached Get: %q %v", p, err)
	}
}

func TestWindowsZip(t *testing.T) {
	archive := zipped(t, "snappea.exe", payload)
	srv, _ := release(t, map[string][]byte{"snappea-v1.2.3-x86_64-pc-windows-msvc.zip": archive})
	s := source(t, srv.URL, sum(archive)+" *snappea-v1.2.3-x86_64-pc-windows-msvc.zip\n", "windows")
	s.GOARCH = "amd64"
	p, err := s.Get(context.Background())
	if err != nil || filepath.Base(p) != "snappea.exe" {
		t.Fatalf("Get: %q %v", p, err)
	}
	if got, _ := os.ReadFile(p); string(got) != payload {
		t.Fatalf("extracted %q", got)
	}
}

func TestChecksumMismatchKeepsNothing(t *testing.T) {
	archive := tarGz(t, "snappea", payload)
	srv, _ := release(t, map[string][]byte{"snappea-v1.2.3-aarch64-unknown-linux-gnu.tar.gz": archive})
	s := source(t, srv.URL, strings.Repeat("ab", 32)+"  snappea-v1.2.3-aarch64-unknown-linux-gnu.tar.gz\n", "linux")
	_, err := s.Get(context.Background())
	var de *DownloadError
	if !errors.As(err, &de) || !strings.Contains(err.Error(), "sha256") || !strings.HasSuffix(de.URL, "/v1.2.3/snappea-v1.2.3-aarch64-unknown-linux-gnu.tar.gz") {
		t.Fatalf("want checksum DownloadError, got %v", err)
	}
	if entries, _ := os.ReadDir(filepath.Join(s.CacheDir, "1.2.3")); len(entries) != 0 {
		t.Fatalf("left %v", entries)
	}
}

func TestDownloadFailures(t *testing.T) {
	archive := tarGz(t, "other", payload)
	srv, _ := release(t, map[string][]byte{"snappea-v1.2.3-aarch64-unknown-linux-gnu.tar.gz": archive})
	line := "  snappea-v1.2.3-aarch64-unknown-linux-gnu.tar.gz\n"
	for name, s := range map[string]Source{
		"not found":   source(t, srv.URL+"/missing", sum(archive)+line, "linux"),
		"no snappea":      source(t, srv.URL, sum(archive)+line, "linux"),
		"unreachable": source(t, "http://127.0.0.1:1", sum(archive)+line, "linux"),
	} {
		_, err := s.Get(context.Background())
		var de *DownloadError
		if !errors.As(err, &de) || !strings.Contains(err.Error(), de.URL) {
			t.Errorf("%s: want DownloadError with URL, got %v", name, err)
		}
	}
}

func TestUnavailable(t *testing.T) {
	line := strings.Repeat("0", 64) + "  snappea-v1.2.3-aarch64-unknown-linux-gnu.tar.gz\n"
	dev := source(t, "http://127.0.0.1:1", line, "linux")
	dev.Version = "dev"
	plan9 := source(t, "http://127.0.0.1:1", line, "plan9")
	off := source(t, "http://127.0.0.1:1", line, "linux")
	for name, c := range map[string]struct {
		s    Source
		env  string
		want string
	}{
		"dev":         {dev, "", "not a release"},
		"no checksum": {source(t, "http://127.0.0.1:1", "", "linux"), "", "no checksum"},
		"platform":    {plan9, "", "plan9/arm64"},
		"opt out":     {off, "0", "SNAPPEA_DOWNLOAD=0"},
	} {
		t.Run(name, func(t *testing.T) {
			t.Setenv("SNAPPEA_DOWNLOAD", c.env)
			_, err := c.s.Get(context.Background())
			var u *Unavailable
			if !errors.As(err, &u) || !strings.Contains(err.Error(), c.want) {
				t.Fatalf("want Unavailable %q, got %v", c.want, err)
			}
		})
	}
}

func TestCommittedFiles(t *testing.T) {
	// Between releases: version dev and no checksums. At a release tag: every target.
	if Version == "dev" {
		if strings.TrimSpace(sums) != "" {
			t.Fatal("SHA256SUMS must be empty while Version is dev")
		}
		return
	}
	for platform := range targets {
		goos, goarch, _ := strings.Cut(platform, "/")
		s := Release
		s.GOOS, s.GOARCH = goos, goarch
		asset, err := s.asset()
		if err != nil {
			t.Fatal(err)
		}
		if _, ok := s.checksum(asset); !ok {
			t.Errorf("SHA256SUMS lacks %s", asset)
		}
	}
}
