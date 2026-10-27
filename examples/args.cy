# Command-line arguments, as C hands them over (owner, 2026-09-29: "low-level shit, decided"): MAIN may
# take 'argc' and 'argv', the program's name first, each word NUL-ended, a null pointer after the last.
# Reading them is C's world, so it is no_check; a friendly list will be written in CyborgPL on top.
# examples/args.args holds the words the test passes: apples, 3, hello world.
!MAIN [$argc: integer.signed.32_bit, $argv: pointer[pointer[integer.unsigned.8_bit]]]:
    std::print.stored("argc = ", $argc, newline);
    variable $i: integer.signed.32_bit = 1;
    while $i less_than $argc:
        !no_check let $word := pointer.read(pointer.at($argv, by: $i));
        variable $k: integer.signed.32_bit = 0;
        variable $bytes: std::list[integer.unsigned.8_bit] = [];
        loop:
            !no_check let $b := pointer.read(pointer.at($word, by: $k));
            if $b equal_to 0:
                break;
            end of if
            std::list.append(lend.mutable $bytes, value: $b);
            change $k = $k plus 1;
        end of loop
        let $text := to.string.utf8($bytes) otherwise "(not UTF-8)";      # bytes into text (owner, 2026-09-30)
        std::print.stored("argument ", $i, ": ", $text, newline);
        change $i = $i plus 1;
    end of while
    !no_check let $end := pointer.read(pointer.at($argv, by: $argc));
    std::print.stored("argv[argc] is null: ", pointer.is_null($end), newline);
!end of MAIN
