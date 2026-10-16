# Uses a Kit that has a MAIN of its own: that MAIN is left out, and this
# one runs (7 + 1 = 8, never 99).
extern.C fn exit ['code': int32];
MAIN { no_check my.exit[runnable::seven[] plus 1]; }
