"""Global mode: the pytest plugin, run on small throwaway projects."""

import shutil

import pytest
from conftest import HERE, SPEC

APP = f"{HERE}/example/app/locks.py"

TESTS = '''
import pytest
from locks import LockServer
from prs import emit, expect_violation


class Clock:
    t = 0.0
    def __call__(self):
        return self.t


def test_correct():
    clock = Clock()
    server = LockServer(lease=1, clock=clock)
    server.acquire("c1")
    clock.t = 2
    server.acquire("c2")


def test_buggy_expiry():
    clock = Clock()
    server = LockServer(lease=1, clock=clock, buggy_expiry=True)
    server.acquire("c1")
    clock.t = 2
    server.acquire("c2")


def test_no_events():
    assert 1 + 1 == 2


@pytest.mark.prs(skip=True)
def test_skipped_on_purpose():
    emit("eEnter", client="a")
    emit("eEnter", client="b")


@pytest.mark.prs(skip=["MutualExclusion"])
def test_skip_one_spec():
    emit("eEnter", client="a")
    emit("eEnter", client="b")


def test_waiting_forever_passes_without_liveness():
    LockServer().acquire("c1")
    LockServer.acquire(LockServer(), "c2", "L2")
    emit("eRequest", key="L3", client="c3")


@pytest.mark.prs(liveness=True)
def test_waiting_forever_fails_with_liveness():
    emit("eRequest", key="L3", client="c3")


def test_fixture(prs_events):
    server = LockServer()
    server.acquire("c1")
    server.release("c1")
    assert prs_events.count("eEnter") == 1


def test_expected_violation_is_not_reported_twice():
    with expect_violation("MutualExclusion"):
        emit("eEnter", client="a")
        emit("eEnter", client="b")


def test_failing_test_is_not_also_checked():
    emit("eEnter", client="a")
    emit("eEnter", client="b")
    assert False, "the test's own failure"
'''


@pytest.fixture
def project(pytester):
    pytester.makepyprojecttoml('[tool.prs]\nspec_files = ["specs/*.p"]\n')
    (pytester.path / "specs").mkdir()
    shutil.copy(SPEC, pytester.path / "specs" / "lock.p")
    shutil.copy(APP, pytester.path / "locks.py")
    pytester.makepyfile(test_locks=TESTS)
    return pytester


def run(pytester, *args):
    return pytester.runpytest_subprocess("-p", "no:cacheprovider", *args)


def test_every_test_is_checked(project):
    r = run(project, "-v")
    r.assert_outcomes(passed=7, failed=3)
    r.stdout.fnmatch_lines(
        [
            "prs: checking P specs from specs/*.p (auto)",
            "*test_buggy_expiry FAILED*",
            "*test_waiting_forever_passes_without_liveness PASSED*",
            "*test_waiting_forever_fails_with_liveness FAILED*",
            "*test_expected_violation_is_not_reported_twice PASSED*",
        ]
    )
    out = r.stdout.str()
    assert "P spec MutualExclusion was broken: c2 entered while c1 is inside" in out
    assert "locks.py:27  <- broke the spec here" in out
    assert "Spec: specs/lock.p:19" in out
    assert "P spec RequestsGranted was still waiting when the events ended" in out
    assert "AssertionError: the test's own failure" in out
    # the plugin's frames are hidden: the report points at the test
    assert "pytest_plugin.py" not in out


def test_marked_mode(project):
    r = run(project, "--prs", "marked")
    # only the marked tests are checked; test_buggy_expiry passes unchecked
    r.assert_outcomes(passed=8, failed=2)


def test_off(project):
    r = run(project, "--prs", "off")
    r.assert_outcomes(passed=9, failed=1)
    assert "prs:" not in r.stdout.str()


def test_exclude_setting(project):
    project.makepyprojecttoml('[tool.prs]\nspec_files = ["specs/*.p"]\nexclude = ["MutualExclusion"]\n')
    r = run(project)
    r.assert_outcomes(passed=8, failed=2)


def test_without_settings_the_plugin_is_inert(pytester):
    pytester.makepyfile(
        """
        from prs import emit
        def test_a():
            emit("eEnter", client="a")
        """
    )
    r = run(pytester)
    r.assert_outcomes(passed=1)


def test_bad_settings_are_reported(pytester):
    pytester.makepyprojecttoml('[tool.prs]\nspec_files = ["specs/*.p"]\nmode = "sometimes"\n')
    pytester.makepyfile("def test_a(): pass")
    r = run(pytester)
    assert "mode must be one of auto, marked, off" in r.stdout.str() + r.stderr.str()
    assert r.ret != 0


def test_events_in_fixture_setup_count(project):
    project.makepyfile(
        test_fixture_events="""
        import pytest
        from prs import emit

        @pytest.fixture
        def held_lock():
            emit("eEnter", client="setup")

        def test_double_enter(held_lock):
            emit("eEnter", client="test")
        """
    )
    r = run(project, "test_fixture_events.py")
    r.assert_outcomes(failed=1)
    r.stdout.fnmatch_lines(["*test entered while setup is inside*"])
