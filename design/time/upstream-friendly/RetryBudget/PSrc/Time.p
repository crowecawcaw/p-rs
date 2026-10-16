/*****************************************************************************
 * The entire time "extension": plain P foreign functions. Times are int
 * microseconds.
 *
 * p-next implements them natively (causal clocks, see PROPOSAL.md).
 * Upstream P links the C# stubs in PForeign/, where time stands still: Now()
 * is always 0 and Elapse is a no-op, so the model runs exactly as an untimed
 * model does today and every budget assertion trivially holds.
 *****************************************************************************/

// The machine's local clock: worst / best case. Use for absolute deadlines.
fun Now(): int;
fun NowBest(): int;

// A stopwatch. Mark() captures the current clock; Elapsed(m) is the worst
// (ElapsedBest the best) case of now - mark, computed with the correlation
// kept, so shared history cancels. Use these for budgets, not Now() - t0.
// Upstream: Mark() is 0 and Elapsed is 0.
fun Mark(): int;
fun Elapsed(m: int): int;
fun ElapsedBest(m: int): int;

// Local work taking between lo and hi microseconds.
fun Elapse(lo: int, hi: int);

fun ms(n: int): int { return n * 1000; }
