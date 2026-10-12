//! CNC, an official lobe (`design/cores.md`): everything about C ("CNC Not C"): `extern.C`, `incl.C`, `layout.C`, callbacks and the `C.` types; Clang ships in it at release.
//! Empty for now: its features still live in the Brainstem (`synapse/compiler`)
//! as switches, and move here at their step of `synapse/PLUGINS.md`'s build
//! order.

/// The lobe's name, for messages.
pub const NAME: &str = "CNC";
/// Its registry ID.
pub const ID: &str = "04b99a7cbb141b84b9f2f38c085c124e2badc37d663f71c6deebdd6ac24e79b1";
/// The one version of it Synapse knows.
pub const VERSION: &str = "2026-10-10T16:42:00+07:00";
