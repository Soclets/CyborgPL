// Heap to stack (design claude, 2026-10-26: a fixed array in a variable on
// `aarch64-none` needed the runtime's heap): memory the runtime hands out
// with `cyborg_alloc`, of a size known here and small, that never leaves
// the function that asked for it, is that function's own stack memory
// instead, and its `cyborg_free` goes. `let $a: array[i32] = [5, 3, 9, 1,]`
// is then sixteen bytes of the frame, on every target: a chip with no OS
// has no heap to ask, and one with an OS skips two calls.
//
// "Never leaves" is read from every use of the pointer, through the
// addresses made from it: loads from it and stores into it, copies and
// fills of it, the runtime's `cyborg_out_*` (which read bytes out and keep
// nothing), and the one `cyborg_free` of it. Anything else — stored as a
// value, passed to another function, returned, compared, joined in a phi —
// and it stays on the heap.
//
// And a text's free whose length says it was never allocated (the `STATIC`
// bit, a literal) does nothing; the call goes.

#include "heap_to_stack.h"

#include "llvm/IR/Constants.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/InstIterator.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/IntrinsicInst.h"

#include <algorithm>
#include <vector>

using namespace llvm;

namespace {

/// What one function's frame takes in all, at most: past it the rest stay
/// on the heap. A chip with no OS has a small stack (`bare_station`'s
/// is 64 KiB).
constexpr uint64_t MOST = 4096;

/// The runtime's `STATIC` bit in a text's length (`runtime/src/lib.rs`).
constexpr uint64_t STATIC = uint64_t(1) << 63;

bool calls(const CallBase *cb, StringRef name) {
    const Function *callee = cb->getCalledFunction();
    return callee && callee->getName() == name;
}

/// Whether every use of `p`, and of every address made from it, keeps it
/// in this function; the frees of it go in `frees`.
bool stays(Value *p, std::vector<CallBase *> &frees) {
    std::vector<Value *> work{p};
    std::vector<Value *> seen;
    while (!work.empty()) {
        Value *v = work.back();
        work.pop_back();
        if (std::find(seen.begin(), seen.end(), v) != seen.end()) {
            continue;
        }
        seen.push_back(v);
        for (User *u : v->users()) {
            if (auto *ld = dyn_cast<LoadInst>(u)) {
                if (ld->getPointerOperand() != v || ld->isVolatile()) {
                    return false;
                }
            } else if (auto *st = dyn_cast<StoreInst>(u)) {
                if (st->getPointerOperand() != v || st->getValueOperand() == v || st->isVolatile()) {
                    return false;
                }
            } else if (auto *gep = dyn_cast<GetElementPtrInst>(u)) {
                if (gep->getPointerOperand() != v) {
                    return false;
                }
                work.push_back(gep);
            } else if (auto *mi = dyn_cast<MemIntrinsic>(u)) {
                if (mi->isVolatile()) {
                    return false;
                }
                // A copy's or a fill's address, never its length.
                if (mi->getLength() == v) {
                    return false;
                }
            } else if (auto *cb = dyn_cast<CallBase>(u)) {
                if (calls(cb, "cyborg_free") && v == p && cb->getArgOperand(0) == v) {
                    frees.push_back(cb);
                } else if (cb->getCalledFunction() && cb->getCalledFunction()->getName().starts_with("cyborg_out_")) {
                    // Bytes read out of it into a print; the pointer is not kept.
                } else if (auto *ii = dyn_cast<IntrinsicInst>(cb); ii && ii->isLifetimeStartOrEnd()) {
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
    }
    return true;
}

} // namespace

PreservedAnalyses kayo::HeapToStackPass::run(Function &f, FunctionAnalysisManager &) {
    if (f.isDeclaration()) {
        return PreservedAnalyses::all();
    }
    std::vector<CallBase *> allocs;
    std::vector<CallBase *> static_frees;
    for (Instruction &i : instructions(f)) {
        auto *cb = dyn_cast<CallBase>(&i);
        if (!cb) {
            continue;
        }
        if (calls(cb, "cyborg_alloc")) {
            auto *n = dyn_cast<ConstantInt>(cb->getArgOperand(0));
            // Zero bytes is the runtime's null pointer, not memory.
            if (n && n->getZExtValue() > 0 && n->getZExtValue() <= MOST) {
                allocs.push_back(cb);
            }
        } else if (calls(cb, "cyborg_str_free") && cb->arg_size() == 2) {
            auto *len = dyn_cast<ConstantInt>(cb->getArgOperand(1));
            if (len && len->getBitWidth() == 64 && (len->getZExtValue() & STATIC) != 0) {
                static_frees.push_back(cb);
            }
        }
    }
    bool changed = !static_frees.empty();
    for (CallBase *cb : static_frees) {
        cb->eraseFromParent();
    }
    BasicBlock &entry = f.getEntryBlock();
    uint64_t taken = 0;
    // What earlier runs of this pass already moved here.
    for (Instruction &i : entry) {
        auto *a = dyn_cast<AllocaInst>(&i);
        if (a && a->getName().starts_with("on_stack")) {
            if (auto size = a->getAllocationSize(f.getDataLayout())) {
                taken += size->getFixedValue();
            }
        }
    }
    for (CallBase *cb : allocs) {
        uint64_t n = cast<ConstantInt>(cb->getArgOperand(0))->getZExtValue();
        if (taken + n > MOST) {
            continue;
        }
        std::vector<CallBase *> frees;
        if (!stays(cb, frees)) {
            continue;
        }
        taken += n;
        IRBuilder<> b(&entry, entry.getFirstInsertionPt());
        AllocaInst *slot = b.CreateAlloca(ArrayType::get(b.getInt8Ty(), n), nullptr, "on_stack");
        // As `malloc` aligns: 16 bytes.
        slot->setAlignment(Align(16));
        for (CallBase *fr : frees) {
            fr->eraseFromParent();
        }
        cb->replaceAllUsesWith(slot);
        cb->eraseFromParent();
        changed = true;
    }
    return changed ? PreservedAnalyses::none() : PreservedAnalyses::all();
}
