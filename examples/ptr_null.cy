# `ptr.null`, C's NULL: the pointer to nothing (owner, 2026-10-21). It goes only where its pointer type is
# known: a C call's argument, or a list beside pointers of a known kind. A `:=` holding it alone is refused,
# since nothing would say what it points at.
%external.C procedure getenv [$name: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
%external.C procedure free [$p: pointer.mutable[integer.unsigned.8_bit]];

!MAIN:
    !no_check free(pointer.null);                          # C's free(NULL) does nothing

    let $key := "HOME";
    !no_check let $home := getenv(lend $key);
    let $args := [$home, pointer.null];                 # a list of pointers ending in NULL, as execv wants
    if pointer.is_null($args[position: 2]):
        std::print.shortest("the list ends in NULL", newline);
    end of if
    if not pointer.is_null($args[position: 1]):
        std::print.shortest("HOME is set", newline);
    end of if
!end of MAIN
