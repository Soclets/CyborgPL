# CCOESS — Control & Chat Over Something Service

A Cyborg Assembly & Manufacturing Site project, named 2026-10-02. The
owner's words:

> Will probably need something like CCOESS (Control & Chat Over Something
> Service). Which is another one of Cyborg Assembly & Manufacturing Site.

> CCOESS is the successor of AICNA and Collab, with more features. No
> definition yet, just the full name

**Said "CC O S"** (owner, 2026-10-02: "Pronounced as CC O S").

So far only the name and its lineage: it succeeds AICNA (the owner's
self-hosted server for humans and AI to talk, in Go) and Collab, with
more features. Raised while planning the defining licence
(`defining-license.md`), whose program is to run on the owner's gaming
PC, not the MacBook.

**No language lock** (owner, 2026-10-02: "It has sooo many ways to run,
and most are likely not defined yet. So... How about no lang lock").
Each part picks the language that fits it when it is defined. The owner
expects it to execute assembly on the machine ("it would likely be able
to execute ASM on the machine") and, as it grows, to have a kernel side
("It would also likely be a kernel service when it grows"). Design
claude's notes from that talk, for when parts get defined: Go suits the
chat and networking; Rust suits running machine code and anything
security-critical; kernels choose for you (Windows drivers in C or C++,
Microsoft's Rust support still experimental, and drivers must be signed;
Linux takes Rust or C; macOS has no kernel extensions, only user-space
DriverKit in C++). A service that runs code it receives needs strong
authentication, encryption and a permission model, whatever it is
written in.
