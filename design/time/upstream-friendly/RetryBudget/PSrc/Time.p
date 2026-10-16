/*****************************************************************************
 * The entire time "extension": plain P foreign functions. Times are int
 * microseconds.
 *
 * p-next implements them natively (causal clocks, see PROPOSAL.md).
 * Upstream P links the C# stubs in PForeign/, where time stands still: Now()
 * is always 0 and Elapse is a no-op, so the model runs exactly as an untimed
 * model does today and every budget assertion trivially holds.
 *****************************************************************************/

// The machine's local clock: worst / best case of [best, worst].
fun Now(): int;
fun NowBest(): int;

// Local work taking between lo and hi microseconds.
fun Elapse(lo: int, hi: int);

fun ms(n: int): int { return n * 1000; }
