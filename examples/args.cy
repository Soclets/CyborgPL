# Command-line arguments, as C hands them over (owner, 2026-09-29: "low-level shit, decided"): MAIN may
# take 'argc' and 'argv', the program's name first, each word NUL-ended, a null pointer after the last.
# Reading them is C's world, so it is no_check; a friendly list will be written in CyborgPL on top.
# examples/args.args holds the words the test passes: apples, 3, hello world.
MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    std::println.stored("argc = ", $argc);
    var $i: i32 = 1;
    while $i < $argc:
        no_check let $word := ptr.read(ptr.at($argv, by: $i));
        var $k: i32 = 0;
        var $bytes: std::ls[u8] = [];
        loop:
            no_check let $b := ptr.read(ptr.at($word, by: $k));
            if $b == 0:
                break;
            ;if
            std::ls.append(lend.mut $bytes, value: $b);
            change $k += 1;
        ;loop
        let $text := to.str.utf8($bytes) ?? "(not UTF-8)";      # bytes into text (owner, 2026-09-30)
        std::println.stored("argument ", $i, ": ", $text);
        change $i += 1;
    ;while
    no_check let $end := ptr.read(ptr.at($argv, by: $argc));
    std::println.stored("argv[argc] is null: ", ptr.is_null($end));
;MAIN
