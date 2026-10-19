package prstest

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"sync"
)

type coverageItems struct {
	Total    int               `json:"total"`
	Covered  int               `json:"covered"`
	Declared []json.RawMessage `json:"declared"`
	Visited  []json.RawMessage `json:"visited"`
}
type modelCoverage struct {
	Scope       string            `json:"scope"`
	States      coverageItems     `json:"states"`
	Handlers    coverageItems     `json:"handlers"`
	Transitions []json.RawMessage `json:"transitions"`
}

var suiteCoverage = struct {
	sync.Mutex
	Specs map[string]*modelCoverage
}{Specs: make(map[string]*modelCoverage)}

func union(a, b []json.RawMessage) []json.RawMessage {
	values := make(map[string]json.RawMessage)
	for _, raw := range append(a, b...) {
		var value any
		if err := json.Unmarshal(raw, &value); err != nil {
			panic(err)
		}
		canonical, err := json.Marshal(value)
		if err != nil {
			panic(err)
		}
		values[string(canonical)] = canonical
	}
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	result := make([]json.RawMessage, 0, len(keys))
	for _, key := range keys {
		result = append(result, values[key])
	}
	return result
}

func collectCoverage(data []byte) error {
	var report struct {
		Specs []struct {
			Spec     string         `json:"spec"`
			Coverage *modelCoverage `json:"coverage"`
		} `json:"specs"`
	}
	if err := json.Unmarshal(data, &report); err != nil {
		return err
	}
	suiteCoverage.Lock()
	defer suiteCoverage.Unlock()
	for _, spec := range report.Specs {
		if spec.Coverage == nil {
			return fmt.Errorf("missing coverage for %s", spec.Spec)
		}
		target := suiteCoverage.Specs[spec.Spec]
		if target == nil {
			target = &modelCoverage{Scope: "observed_monitor"}
			suiteCoverage.Specs[spec.Spec] = target
		}
		for _, pair := range [][2]*coverageItems{{&target.States, &spec.Coverage.States}, {&target.Handlers, &spec.Coverage.Handlers}} {
			pair[0].Declared = union(pair[0].Declared, pair[1].Declared)
			pair[0].Visited = union(pair[0].Visited, pair[1].Visited)
			pair[0].Total = len(pair[0].Declared)
			pair[0].Covered = len(pair[0].Visited)
		}
		target.Transitions = union(target.Transitions, spec.Coverage.Transitions)
	}
	return nil
}

// WriteCoverage is called AFTER m.Run, so all parallel tests and their cleanups
// have finished. This reports model execution, not Go source line coverage.
func WriteCoverage(path string) error {
	suiteCoverage.Lock()
	defer suiteCoverage.Unlock()
	type specReport struct {
		Spec     string         `json:"spec"`
		Coverage *modelCoverage `json:"coverage"`
	}
	report := struct {
		Scope string       `json:"scope"`
		Specs []specReport `json:"specs"`
	}{Scope: "go_completed_checks", Specs: []specReport{}}
	names := make([]string, 0, len(suiteCoverage.Specs))
	for name := range suiteCoverage.Specs {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		c := suiteCoverage.Specs[name]
		report.Specs = append(report.Specs, specReport{name, c})
		fmt.Printf("P model coverage %s: states %d/%d, handlers %d/%d, observed transitions %d\n", name, c.States.Covered, c.States.Total, c.Handlers.Covered, c.Handlers.Total, len(c.Transitions))
	}
	data, err := json.MarshalIndent(report, "", "  ")
	if err != nil {
		return err
	}
	if err = os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	fmt.Printf("P model coverage JSON: %s\n", path)
	return os.WriteFile(path, append(data, '\n'), 0o644)
}
