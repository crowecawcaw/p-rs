// Package snappeabin downloads the snappea release binary that matches this module's version
// into a per-version user cache, verified against the checksums committed with the
// module. The module itself holds no binaries.
package snappeabin

import (
	"archive/tar"
	"archive/zip"
	"bufio"
	"compress/gzip"
	"context"
	"crypto/sha256"
	_ "embed"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"time"
)

// SHA256SUMS lists the release archives in sha256sum format ("<hex>  <file name>").
// The release workflow writes it, with Version, in the commit it tags.
//
//go:embed SHA256SUMS
var sums string

// maxArchive bounds a download.
const maxArchive = 256 << 20

// Release is where Get downloads from. Tests replace it.
var Release = Source{
	Version: Version,
	BaseURL: "https://github.com/crowecawcaw/snappea/releases/download",
	Sums:    sums,
}

// Source is a set of release archives and their checksums.
type Source struct {
	Version string // the release, without "v"; "dev" turns downloading off
	BaseURL string // archives are at <BaseURL>/v<Version>/<asset>
	Sums    string // sha256sum-format checksums of the archives
	// CacheDir holds one folder per version. Default: os.UserCacheDir()/snappea.
	CacheDir string
	// GOOS and GOARCH select the archive. Default: the running platform.
	GOOS, GOARCH string
}

// Unavailable is returned when this source cannot download a binary at all (a
// development version, an unsupported platform, SNAPPEA_DOWNLOAD=0).
type Unavailable struct{ Reason string }

func (e *Unavailable) Error() string { return e.Reason }

// DownloadError is a failed download or a corrupt archive.
type DownloadError struct {
	URL string
	Err error
}

func (e *DownloadError) Error() string { return fmt.Sprintf("downloading %s failed: %v", e.URL, e.Err) }
func (e *DownloadError) Unwrap() error { return e.Err }

var targets = map[string]string{
	"linux/amd64":   "x86_64-unknown-linux-gnu",
	"linux/arm64":   "aarch64-unknown-linux-gnu",
	"darwin/amd64":  "x86_64-apple-darwin",
	"darwin/arm64":  "aarch64-apple-darwin",
	"windows/amd64": "x86_64-pc-windows-msvc",
}

// one download at a time per process; processes race safely through rename.
var mu sync.Mutex

// URL is where the archive for this platform is, or an *Unavailable error.
func (s Source) URL() (string, error) {
	asset, err := s.asset()
	if err != nil {
		return "", err
	}
	return s.BaseURL + "/v" + s.Version + "/" + asset, nil
}

func (s Source) platform() (goos, goarch string) {
	goos, goarch = s.GOOS, s.GOARCH
	if goos == "" {
		goos = runtime.GOOS
	}
	if goarch == "" {
		goarch = runtime.GOARCH
	}
	return goos, goarch
}

func (s Source) asset() (string, error) {
	if s.Version == "" || s.Version == "dev" {
		return "", &Unavailable{"this snappea module is not a release, so it downloads no snappea binary"}
	}
	goos, goarch := s.platform()
	target, ok := targets[goos+"/"+goarch]
	if !ok {
		return "", &Unavailable{fmt.Sprintf("snappea v%s has no snappea release binary for %s/%s", s.Version, goos, goarch)}
	}
	ext := ".tar.gz"
	if goos == "windows" {
		ext = ".zip"
	}
	return "snappea-v" + s.Version + "-" + target + ext, nil
}

func (s Source) exe() string {
	if goos, _ := s.platform(); goos == "windows" {
		return "snappea.exe"
	}
	return "snappea"
}

// checksum is the expected sha256 of asset.
func (s Source) checksum(asset string) (string, bool) {
	sc := bufio.NewScanner(strings.NewReader(s.Sums))
	for sc.Scan() {
		fields := strings.Fields(sc.Text())
		if len(fields) == 2 && strings.TrimPrefix(fields[1], "*") == asset && len(fields[0]) == 64 {
			return strings.ToLower(fields[0]), true
		}
	}
	return "", false
}

// Get returns the cached snappea binary, downloading it first if needed. Errors are
// *Unavailable or *DownloadError.
func (s Source) Get(ctx context.Context) (string, error) {
	asset, err := s.asset()
	if err != nil {
		return "", err
	}
	want, ok := s.checksum(asset)
	if !ok {
		return "", &Unavailable{fmt.Sprintf("this snappea module lists no checksum for %s", asset)}
	}
	dir := s.CacheDir
	if dir == "" {
		base, err := os.UserCacheDir()
		if err != nil {
			return "", &Unavailable{"no user cache folder to download snappea into: " + err.Error()}
		}
		dir = filepath.Join(base, "snappea")
	}
	dir = filepath.Join(dir, s.Version)
	final := filepath.Join(dir, s.exe())

	mu.Lock()
	defer mu.Unlock()
	if info, err := os.Stat(final); err == nil && info.Mode().IsRegular() {
		return final, nil
	}
	if os.Getenv("SNAPPEA_DOWNLOAD") == "0" {
		return "", &Unavailable{"SNAPPEA_DOWNLOAD=0 turns downloading snappea off"}
	}
	url, _ := s.URL()
	if err := s.download(ctx, url, want, dir, final); err != nil {
		return "", &DownloadError{URL: url, Err: err}
	}
	return final, nil
}

func (s Source) download(ctx context.Context, url, want, dir, final string) error {
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	archive, err := os.CreateTemp(dir, ".download-*")
	if err != nil {
		return err
	}
	defer os.Remove(archive.Name())
	defer archive.Close()

	ctx, cancel := context.WithTimeout(ctx, 5*time.Minute)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return err
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("HTTP %s", resp.Status)
	}
	h := sha256.New()
	n, err := io.Copy(io.MultiWriter(archive, h), io.LimitReader(resp.Body, maxArchive+1))
	if err != nil {
		return err
	}
	if n > maxArchive {
		return fmt.Errorf("archive is larger than %d bytes", maxArchive)
	}
	if got := hex.EncodeToString(h.Sum(nil)); got != want {
		return fmt.Errorf("sha256 %s, want %s from the module's SHA256SUMS", got, want)
	}

	tmp, err := os.CreateTemp(dir, ".snappea-*")
	if err != nil {
		return err
	}
	defer os.Remove(tmp.Name())
	if strings.HasSuffix(url, ".zip") {
		err = extractZip(archive, n, s.exe(), tmp)
	} else {
		if _, err = archive.Seek(0, io.SeekStart); err == nil {
			err = extractTarGz(archive, s.exe(), tmp)
		}
	}
	if cerr := tmp.Close(); err == nil {
		err = cerr
	}
	if err != nil {
		return err
	}
	if err := os.Chmod(tmp.Name(), 0o755); err != nil {
		return err
	}
	if err := os.Rename(tmp.Name(), final); err != nil {
		// Another process won the race (Windows cannot replace a running file).
		if info, serr := os.Stat(final); serr == nil && info.Mode().IsRegular() {
			return nil
		}
		return err
	}
	return nil
}

func extractTarGz(r io.Reader, exe string, w io.Writer) error {
	gz, err := gzip.NewReader(r)
	if err != nil {
		return err
	}
	tr := tar.NewReader(gz)
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			return fmt.Errorf("archive has no %s", exe)
		}
		if err != nil {
			return err
		}
		if hdr.Typeflag == tar.TypeReg && path.Base(hdr.Name) == exe {
			_, err = io.Copy(w, io.LimitReader(tr, maxArchive))
			return err
		}
	}
}

func extractZip(r io.ReaderAt, size int64, exe string, w io.Writer) error {
	zr, err := zip.NewReader(r, size)
	if err != nil {
		return err
	}
	for _, f := range zr.File {
		if f.Mode().IsRegular() && path.Base(f.Name) == exe {
			rc, err := f.Open()
			if err != nil {
				return err
			}
			defer rc.Close()
			_, err = io.Copy(w, io.LimitReader(rc, maxArchive))
			return err
		}
	}
	return errors.New("archive has no " + exe)
}
