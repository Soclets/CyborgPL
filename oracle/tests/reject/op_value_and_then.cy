# `op and.then` could never skip: the arguments are worked out first (owner, 2026-09-29)
!MAIN:
    let $g: function[boolean, boolean] -> boolean = operator and.then;
!end of MAIN
