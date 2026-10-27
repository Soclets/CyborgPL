# a slice is a list, an array's too (owner, 2026-09-23)
!MAIN:
    let $f: array[integer.signed.32_bit] = [10, 20, 30];
    let $b: array[integer.signed.32_bit] = array.slice(lend $f, from: position 1, to: position 2);
!end of MAIN
