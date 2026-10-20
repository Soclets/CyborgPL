# Its one target says `default = false`: checking is fine, and a build
# must name the target (owner, 2026-09-30). Exits 5 when built and run.
extern.C fn exit ['code': i32];
MAIN { no_check my.exit[5]; }
