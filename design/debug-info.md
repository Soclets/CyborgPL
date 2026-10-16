# Debug info

Decided 2026-10-08 and 2026-10-14 (owner). Not yet built: kayo (kayo at c90b4186) puts debug info
inside every build, the final one included.

**kayo emits debug info** (owner, 2026-10-08: "There should be"): the tables that map machine code
back to lines, variables and types, for debuggers and readable crash reports. Compiler messages are
not in the program, and a final build has no run-time checks, so the info is all a final build
carries about its source.

**A working build carries it inside; a final build writes it to a separate file**, and the shipped
program has none (owner, 2026-10-08: "`a`"). Builds are reproducible, so the file always matches
its program.

**The file sits beside the program, the same name with the platform's own ending** (owner,
2026-10-14: "(a)", "(a)"), where each platform's debugger already looks:

| Target | Program | Debug info | Format |
|---|---|---|---|
| macOS | `hello` | `hello.dSYM` | DWARF |
| Linux, and no OS (`aarch64-none`) | `hello` | `hello.debug` | DWARF |
| Windows | `hello.exe` | `hello.pdb` | PDB |

The format follows the ending (design claude's reading, not asked separately).
