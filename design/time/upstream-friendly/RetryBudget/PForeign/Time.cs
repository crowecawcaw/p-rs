// Upstream P only: time stands still, so the model runs untimed.
// (Signatures follow upstream's foreign-function convention: P arguments,
// then the calling machine.)
using PChecker.PRuntime;
using PChecker.PRuntime.Values;

namespace PImplementation
{
    public static partial class GlobalFunctions
    {
        public static PrtInt Now(PMachine machine) => (PrtInt)0;
        public static PrtInt NowBest(PMachine machine) => (PrtInt)0;
        public static void Elapse(PrtInt lo, PrtInt hi, PMachine machine) { }
    }
}
