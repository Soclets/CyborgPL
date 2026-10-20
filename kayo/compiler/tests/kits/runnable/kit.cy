# A Kit others use that also runs on its own (owner, 2026-09-29: a): its
# MAIN runs when it is run itself, and is left out when another uses it.

bare.never [fn];

extern.C fn exit ['code': i32];
pub fn seven [] -> i32 { return 7; }
MAIN { no_check my.exit[99]; }
