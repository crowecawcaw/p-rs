// Upstream P only: time stands still, so the model runs untimed.
//
// Not yet compiled against upstream. Types follow the current runtime
// (PChecker.Runtime.Values.PInt); foreign functions take the P arguments and
// then the caller, which is a machine or (for Budget.p) a spec monitor, hence
// `object`. Adjust to your P version's foreign-function convention if needed.
using PChecker.Runtime.Values;

namespace PImplementation
{
    public static partial class GlobalFunctions
    {
        public static PInt Now(object caller) => (PInt)0;
        public static PInt NowBest(object caller) => (PInt)0;
        public static void Elapse(PInt lo, PInt hi, object caller) { }
        public static PInt Mark(object caller) => (PInt)0;
        public static PInt Elapsed(PInt m, object caller) => (PInt)0;
        public static PInt ElapsedBest(PInt m, object caller) => (PInt)0;
    }
}
