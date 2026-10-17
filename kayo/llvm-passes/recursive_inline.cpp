// Recursive inlining (owner, 2026-10-17: "Oh hell yeah, we're doing 1 and
// 2"; DECISIONS "Beating GCC on recursion"): a function that calls itself is
// inlined into itself a few levels deep, as GCC's
// `max-inline-recursive-depth` does, which none of LLVM's inliners will.
//
// How: the body is copied once, as it is, into a snapshot; each call the
// function makes to itself is pointed at the snapshot and inlined, which
// leaves calls to the function itself one level further down; repeated to a
// depth, and stopped where the function would grow past a size. The snapshot
// is then dropped. What the program computes does not change: the same body
// runs, only without the call between two levels.

#include "recursive_inline.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/InstIterator.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Module.h"
#include "llvm/Transforms/Utils/Cloning.h"
#include "llvm/Transforms/Utils/ValueMapper.h"

#include <vector>

using namespace llvm;

namespace {

/// The calls `f` makes to itself, directly.
std::vector<CallBase *> self_calls(Function &f) {
    std::vector<CallBase *> out;
    for (Instruction &i : instructions(f)) {
        if (auto *cb = dyn_cast<CallBase>(&i)) {
            if (cb->getCalledFunction() == &f) {
                out.push_back(cb);
            }
        }
    }
    return out;
}

/// Whether `f` may be inlined into itself: a body, its own calls, nothing
/// that asks it to stay whole (`noinline`, what `--emit-llvm-tv` marks), and
/// no call another function can see the return address of.
bool may_unfold(Function &f) {
    if (f.isDeclaration() || f.hasFnAttribute(Attribute::NoInline) || f.hasFnAttribute(Attribute::OptimizeNone) || f.isVarArg()) {
        return false;
    }
    return !self_calls(f).empty();
}

/// Inline `f` into itself up to `depth` levels, while it stays within
/// `max_size` instructions. Gives back how many calls were inlined.
unsigned unfold(Function &f, unsigned depth, unsigned max_size) {
    ValueToValueMapTy map;
    Function *snapshot = CloneFunction(&f, map);
    snapshot->setLinkage(GlobalValue::InternalLinkage);
    const unsigned body = snapshot->getInstructionCount();
    unsigned inlined = 0;
    for (unsigned level = 0; level < depth; ++level) {
        std::vector<CallBase *> calls = self_calls(f);
        if (calls.empty() || f.getInstructionCount() + calls.size() * body > max_size) {
            break;
        }
        for (CallBase *cb : calls) {
            cb->setCalledFunction(snapshot);
            InlineFunctionInfo info;
            if (InlineFunction(*cb, info).isSuccess()) {
                ++inlined;
            } else {
                cb->setCalledFunction(&f);
            }
        }
    }
    snapshot->eraseFromParent();
    return inlined;
}

} // namespace

namespace kayo {

llvm::PreservedAnalyses RecursiveInlinePass::run(Module &m, ModuleAnalysisManager &) {
    std::vector<Function *> recursive;
    for (Function &f : m) {
        if (may_unfold(f)) {
            recursive.push_back(&f);
        }
    }
    unsigned count = 0;
    for (Function *f : recursive) {
        count += unfold(*f, depth, max_size);
    }
    if (inlined != nullptr) {
        *inlined += count;
    }
    return count == 0 ? PreservedAnalyses::all() : PreservedAnalyses::none();
}

} // namespace kayo
