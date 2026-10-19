// The recursive inliner on hand-written IR, built with AddressSanitizer and
// UndefinedBehaviorSanitizer (kayo/compiler/tests/llvm_passes.rs): the module
// still verifies; a function calling itself is unfolded; two calling each
// other are joined and unfolded; one marked `noinline` and one too big for
// the size are left as they are.

#include "../recursive_inline.h"

#include "llvm/AsmParser/Parser.h"
#include "llvm/IR/InstIterator.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Verifier.h"
#include "llvm/Support/SourceMgr.h"
#include "llvm/Support/raw_ostream.h"

using namespace llvm;

static const char *IR = R"(
define i64 @fib(i64 %n) {
entry:
  %small = icmp ult i64 %n, 2
  br i1 %small, label %done, label %more
more:
  %a = sub i64 %n, 1
  %fa = call i64 @fib(i64 %a)
  %b = sub i64 %n, 2
  %fb = call i64 @fib(i64 %b)
  %s = add i64 %fa, %fb
  ret i64 %s
done:
  ret i64 %n
}

declare void @say(i64)

define void @countdown(i64 %n) {
entry:
  %z = icmp eq i64 %n, 0
  br i1 %z, label %done, label %more
more:
  call void @say(i64 %n)
  %m = sub i64 %n, 1
  call void @countdown(i64 %m)
  call void @say(i64 %n)
  ret void
done:
  ret void
}

define i64 @kept(i64 %n) noinline {
  %z = icmp eq i64 %n, 0
  br i1 %z, label %done, label %more
more:
  %m = sub i64 %n, 1
  %r = call i64 @kept(i64 %m)
  ret i64 %r
done:
  ret i64 0
}

define i64 @even(i64 %n) {
  %z = icmp eq i64 %n, 0
  br i1 %z, label %done, label %more
more:
  %m = sub i64 %n, 1
  %r = call i64 @odd(i64 %m)
  ret i64 %r
done:
  ret i64 1
}

define i64 @odd(i64 %n) {
  %z = icmp eq i64 %n, 0
  br i1 %z, label %done, label %more
more:
  %m = sub i64 %n, 1
  %r = call i64 @even(i64 %m)
  ret i64 %r
done:
  ret i64 0
}
)";

static unsigned calls_to(Function &f, const Function &callee) {
    unsigned n = 0;
    for (Instruction &i : instructions(f)) {
        if (auto *cb = dyn_cast<CallBase>(&i)) {
            n += cb->getCalledFunction() == &callee;
        }
    }
    return n;
}

static int fail(const char *what) {
    errs() << "FAIL: " << what << "\n";
    return 1;
}

int main() {
    LLVMContext ctx;
    SMDiagnostic err;
    std::unique_ptr<Module> m = parseAssemblyString(IR, err, ctx);
    if (!m) {
        err.print("recursive_inline_test", errs());
        return 1;
    }
    ModuleAnalysisManager mam;
    unsigned inlined = 0;
    kayo::RecursiveInlinePass pass{{}, 3, 1000, &inlined};
    pass.run(*m, mam);
    if (verifyModule(*m, &errs())) {
        return fail("the module does not verify after the pass");
    }
    Function &fib = *m->getFunction("fib");
    Function &countdown = *m->getFunction("countdown");
    // Three levels of two calls each: 2 + 4 + 8 inlined, 16 calls left.
    if (calls_to(fib, fib) != 16) {
        return fail("fib is not unfolded three levels");
    }
    if (calls_to(countdown, countdown) != 1 || calls_to(countdown, *m->getFunction("say")) != 8) {
        return fail("countdown is not unfolded with its effects in order");
    }
    if (calls_to(*m->getFunction("kept"), *m->getFunction("kept")) != 1) {
        return fail("a noinline function was unfolded");
    }
    // Mutual recursion is joined: each partner inlined where the other calls
    // it, so each calls itself, and is unfolded three levels.
    Function &even = *m->getFunction("even");
    Function &odd = *m->getFunction("odd");
    if (calls_to(even, odd) != 0 || calls_to(odd, even) != 0 || calls_to(even, even) != 1 || calls_to(odd, odd) != 1) {
        return fail("mutual recursion was not joined into self-recursion");
    }
    if (inlined != 2 + 4 + 8 + 3 + 2 + 3 + 3) {
        errs() << "inlined " << inlined << "\n";
        return fail("the count of inlined calls is off");
    }
    if (m->getFunctionList().size() != 6) {
        return fail("a snapshot was left in the module");
    }
    // A size too small for one more level leaves the function whole.
    std::unique_ptr<Module> small = parseAssemblyString(IR, err, ctx);
    unsigned none = 0;
    kayo::RecursiveInlinePass tight{{}, 3, 10, &none};
    tight.run(*small, mam);
    if (none != 0 || calls_to(*small->getFunction("fib"), *small->getFunction("fib")) != 2) {
        return fail("the size limit did not hold");
    }
    outs() << "ok\n";
    return 0;
}
