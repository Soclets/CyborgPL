// Heap to stack (heap_to_stack.cpp), as a pass of LLVM's new pass manager.

#pragma once

#include "llvm/IR/PassManager.h"

namespace kayo {

struct HeapToStackPass : llvm::PassInfoMixin<HeapToStackPass> {
    llvm::PreservedAnalyses run(llvm::Function &f, llvm::FunctionAnalysisManager &am);
    static bool isRequired() { return false; }
};

} // namespace kayo
