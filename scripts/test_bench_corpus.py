"""Checks the discovery accounting used by the corpus benchmark."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('bench_corpus', Path(__file__).with_name('bench-corpus.py'))
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)


class Accounting(unittest.TestCase):
    def run_row(self, strategy, first, complete=False):
        return dict(case='tc', strategy=strategy, seed=0, status='ok', schedules_tested=100,
                    first_issue_schedules=first, exhaustive_complete=complete)

    def test_every_strategy_uses_the_same_catalog_and_counts_successful_schedules(self):
        cases = bench.analyze([dict(id='model', cases=['tc'], runs=[
            self.run_row('random', {'site-a': 5}),
            self.run_row('feedback', {'site-a': 2, 'site-b': 91}),
            self.run_row('dfs', {'site-b': 1}),
        ])])
        case = cases[0]
        self.assertEqual(case['known_issues'], ['site-a', 'site-b'])
        self.assertEqual([r['schedules_to_all_known'] for r in case['runs']], [None, 91, None])
        self.assertFalse(case['catalog_complete'])

    def test_no_discovery_is_unknown_rather_than_zero_schedules_to_all(self):
        case = bench.analyze([dict(id='model', cases=['tc'], runs=[self.run_row('random', {})])])[0]
        self.assertFalse(case['catalog_complete'])
        self.assertIsNone(case['runs'][0]['schedules_to_all_known'])

    def test_exhaustion_certifies_only_a_catalog_that_it_covers(self):
        rows = [self.run_row('dfs', {'site-a': 3}, True), self.run_row('random', {'site-b': 8})]
        self.assertFalse(bench.analyze([dict(id='model', runs=rows)])[0]['catalog_complete'])
        rows[0]['first_issue_schedules']['site-b'] = 4
        self.assertTrue(bench.analyze([dict(id='model', runs=rows)])[0]['catalog_complete'])

    def test_known_case_remains_present_without_any_completed_runs(self):
        case = bench.analyze([dict(id='model', cases=['tc'], runs=[])])[0]
        self.assertEqual(case['case'], 'tc')
        self.assertEqual(case['runs'], [])
        self.assertFalse(case['catalog_complete'])

if __name__ == '__main__':
    unittest.main()
