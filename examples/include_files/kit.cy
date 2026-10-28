# Include bytes (owner, 2026-10-10): a file's bytes go into the program while it is built. The path
# starts at the kit's root (this folder) in every file, as `my.` does; `incl.bytes` gives an
# `array[u8]`, `incl.str.utf8` the text as if written in quotes, checked as UTF-8. Both are a
# const's value only, so they are read where they sit and copied only where something takes them.
# UTF-16 and UTF-32 say their byte order: `.low_first` (as Windows writes) or `.high_first`.

MAIN:
    const $tiny := incl.bytes["assets/tiny.bin"];
    std::print(arr.len($tiny), " bytes, the last is ", $tiny[pos: 4], newln);
    std::print(ui.header.greeting());
    const $windows := incl.str.%utf16.low_first["assets/windows.txt"];
    std::print($windows);
;MAIN
