// The optimisation pipeline kayo runs: LLVM's `default<O3>`, as
// `LLVMRunPasses` builds it, with kayo's own passes in their places.
// Recursive inlining goes in after the early simplification, so it copies a
// function already cleaned up, and ahead of the inliner and the passes that
// make use of what it unfolded.

#include "recursive_inline.h"

#include "llvm-c/Error.h"
#include "llvm-c/TargetMachine.h"
#include "llvm/Analysis/CGSCCPassManager.h"
#include "llvm/Analysis/LoopAnalysisManager.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/PassManager.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Passes/StandardInstrumentations.h"
#include "llvm/Support/CommandLine.h"
#include "llvm/Transforms/Utils/LoopUtils.h"
#include "llvm/Target/TargetMachine.h"

using namespace llvm;

/// `default<O3>` over `module` for `machine`, with recursive inlining to
/// `depth` levels and `max_size` instructions (0 levels: none). How many
/// calls it inlined goes in `*inlined`.
extern "C" void kayo_optimize(LLVMModuleRef module, LLVMTargetMachineRef machine, unsigned depth, unsigned max_size, unsigned *inlined) {
    Module *m = unwrap(module);
    TargetMachine *tm = reinterpret_cast<TargetMachine *>(machine);
    *inlined = 0;

    LoopAnalysisManager lam;
    FunctionAnalysisManager fam;
    CGSCCAnalysisManager cgam;
    ModuleAnalysisManager mam;
    PassInstrumentationCallbacks pic;
    PassBuilder pb(tm, PipelineTuningOptions(), std::nullopt, &pic);
    StandardInstrumentations si(m->getContext(), false, false);
    si.registerCallbacks(pic, &mam);
    pb.registerModuleAnalyses(mam);
    pb.registerCGSCCAnalyses(cgam);
    pb.registerFunctionAnalyses(fam);
    pb.registerLoopAnalyses(lam);
    pb.crossRegisterProxies(lam, fam, cgam, mam);

    if (depth > 0) {
        pb.registerPipelineEarlySimplificationEPCallback([&](ModulePassManager &mpm, OptimizationLevel, ThinOrFullLTOPhase) {
            mpm.addPass(kayo::RecursiveInlinePass{{}, depth, max_size, inlined});
        });
    }
    // A build with a loop under `no_cheat` (the module flag
    // `kayo.keep_rounds`, `codegen/keep.rs`): no loop's answer is replaced
    // by its closed form (`n x (n + 1) / 2` for a sum), so every round runs.
    // LLVM's own option, `-replexitval=never`, set for this run only.
    cl::opt<ReplaceExitVal> *rev = nullptr;
    ReplaceExitVal was = OnlyCheapRepl;
    if (m->getModuleFlag("kayo.keep_rounds")) {
        auto &opts = cl::getRegisteredOptions();
        auto it = opts.find("replexitval");
        if (it != opts.end()) {
            rev = static_cast<cl::opt<ReplaceExitVal> *>(it->second);
            was = rev->getValue();
            rev->setValue(NeverRepl);
        }
    }
    ModulePassManager mpm = pb.buildPerModuleDefaultPipeline(OptimizationLevel::O3);
    mpm.run(*m, mam);
    if (rev) {
        rev->setValue(was);
    }
}
