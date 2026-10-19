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
// runs, only without the call between two levels. Two functions calling
// each other are first joined: each partner is inlined where the other
// calls it, which leaves each calling itself.

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

/// Whether `f` may be inlined anywhere: a body, nothing that asks it to stay
/// whole (`noinline`, what `--emit-llvm-tv` marks).
bool inlinable(const Function &f) {
    return !f.isDeclaration() && !f.hasFnAttribute(Attribute::NoInline) && !f.hasFnAttribute(Attribute::OptimizeNone) && !f.isVarArg();
}

/// Whether `f` may be inlined into itself: inlinable, and calling itself.
bool may_unfold(Function &f) {
    return inlinable(f) && !self_calls(f).empty();
}

/// Whether `f` calls `g` directly.
bool calls(const Function &f, const Function &g) {
    for (const Instruction &i : instructions(f)) {
        if (const auto *cb = dyn_cast<CallBase>(&i)) {
            if (cb->getCalledFunction() == &g) {
                return true;
            }
        }
    }
    return false;
}

/// Two functions calling each other (owner, 2026-10-18: "Extend your
/// recursive-inline pass to mutual recursion"): where `f` calls a `g` that
/// calls `f` back, `g`'s body, as it was before anything here, is inlined
/// into `f`, so `f` calls itself and is unfolded as any function that does.
/// Gives back how many calls were inlined.
unsigned join_pairs(Module &m, unsigned max_size) {
    struct Pair {
        Function *f;
        CallBase *call;
        Function *g;
    };
    std::vector<Pair> pairs;
    for (Function &f : m) {
        if (!inlinable(f)) {
            continue;
        }
        for (Instruction &i : instructions(f)) {
            auto *cb = dyn_cast<CallBase>(&i);
            Function *g = cb ? cb->getCalledFunction() : nullptr;
            if (g != nullptr && g != &f && inlinable(*g) && g->getInstructionCount() <= max_size && calls(*g, f)) {
                pairs.push_back({&f, cb, g});
            }
        }
    }
    // Every partner's body as it was, before any of them changes.
    std::vector<std::pair<Function *, Function *>> snapshots;
    auto snapshot_of = [&](Function *g) {
        for (auto &[orig, snap] : snapshots) {
            if (orig == g) {
                return snap;
            }
        }
        ValueToValueMapTy map;
        Function *snap = CloneFunction(g, map);
        snap->setLinkage(GlobalValue::InternalLinkage);
        snapshots.emplace_back(g, snap);
        return snap;
    };
    for (const Pair &p : pairs) {
        snapshot_of(p.g);
    }
    unsigned inlined = 0;
    for (const Pair &p : pairs) {
        if (p.f->getInstructionCount() + p.g->getInstructionCount() > max_size) {
            continue;
        }
        p.call->setCalledFunction(snapshot_of(p.g));
        InlineFunctionInfo info;
        if (InlineFunction(*p.call, info).isSuccess()) {
            ++inlined;
        } else {
            p.call->setCalledFunction(p.g);
        }
    }
    for (auto &[orig, snap] : snapshots) {
        snap->eraseFromParent();
    }
    return inlined;
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
    unsigned joined = join_pairs(m, max_size);
    std::vector<Function *> recursive;
    for (Function &f : m) {
        if (may_unfold(f)) {
            recursive.push_back(&f);
        }
    }
    unsigned count = joined;
    for (Function *f : recursive) {
        count += unfold(*f, depth, max_size);
    }
    if (inlined != nullptr) {
        *inlined += count;
    }
    return count == 0 ? PreservedAnalyses::all() : PreservedAnalyses::none();
}

} // namespace kayo
