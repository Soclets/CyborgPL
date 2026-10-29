# `ptr.null`, C's NULL: the pointer to nothing (owner, 2026-10-21). It goes only where its pointer type is
# known: a C call's argument, or a list beside pointers of a known kind. A `:=` holding it alone is refused,
# since nothing would say what it points at.
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C proc free [$p: ptr.mut[u8]];

MAIN:
    no_check free(ptr.null);                          # C's free(NULL) does nothing

    let $key := "HOME";
    no_check let $home := getenv(lend $key);
    let $args := [$home, ptr.null];                 # a list of pointers ending in NULL, as execv wants
    if ptr.is_null($args[pos: 2]):
        std::print("the list ends in NULL", newln);
    ;if
    if not ptr.is_null($args[pos: 1]):
        std::print("HOME is set", newln);
    ;if
;MAIN
