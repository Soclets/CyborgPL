// Recursive inlining (recursive_inline.cpp), as a pass of LLVM's new pass
// manager.

#pragma once

#include "llvm/IR/PassManager.h"

namespace kayo {

struct RecursiveInlinePass : llvm::PassInfoMixin<RecursiveInlinePass> {
    /// How many levels a function is inlined into itself, at most.
    unsigned depth;
    /// How many instructions a function may grow to, at most.
    unsigned max_size;
    /// Counts the calls inlined, for the report.
    unsigned *inlined;

    llvm::PreservedAnalyses run(llvm::Module &m, llvm::ModuleAnalysisManager &am);
    static bool isRequired() { return false; }
};

} // namespace kayo
