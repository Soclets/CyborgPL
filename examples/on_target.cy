# Code for one target (owner, 2026-10-06): `on.target` is chosen when compiling, and a branch it rules
# out is not compiled at all; `on.machine` asks the actual machine once, when running.
# The oracle's target is aarch64-macos.

on.target.os windows:
    fn where [] -> str.utf8:
        ret "windows";
    end of fn where
end of on.target.os
else:
    fn where [] -> str.utf8:
        ret "not windows";
    end of fn where
end of else

!MAIN:
    on.target aarch64-macos:
        std::print.shortest("built for aarch64-macos", newln);
    end of on.target
    else on.target.chip x86_64:
        let $n: int.sign.64 := 5;
        !no_check inline.ASM with [$n -> rax, $n <- rax]:   # never compiled here: x86 assembly
            neg rax
        end of inline.ASM
    end of else on.target.chip
    else:
        std::print.shortest("built for something else", newln);
    end of else
    on.target.chip aarch64:
        std::print.shortest("an ARM chip, ", where(), newln);
    end of on.target.chip
    on.machine.chip aarch64:
        std::print.shortest("running on ARM", newln);
    end of on.machine.chip
    else:
        std::print.shortest("running elsewhere", newln);
    end of else
!end of MAIN
