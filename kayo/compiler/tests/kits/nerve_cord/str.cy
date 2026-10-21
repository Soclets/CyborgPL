# A module named after a built-in type's word, as cord's `str.nrv` is: users write `std::str.shout`.

pub bare.kit_prefix.never fn shout [lent 's': str.utf8] -> str.utf8 {
    return 's' concat "!";
}

# A position after the text says `pos` at the call, as std's `str.at` does
# (owner, 2026-10-08): `std::str.at.byte[lend 's', pos 1]`. This stand-in
# hands back the position plus 103, to show it arrives as the number.
pub coll.fn at {
    bare.kit_prefix.optional;
    at.byte [lent 's': str.utf8, 'pos': u64[min: 1]] -> u64 or fail[out_of_range] {
        if 'pos' greater_than str.bytes[lend 's'] { fail out_of_range; }
        return 'pos' + 103;
    }
}
