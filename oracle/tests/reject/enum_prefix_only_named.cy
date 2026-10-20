pub bare.kit_prefix.optional bare.enum_prefix.only enum Mood { calm, loud }
fn say ['m': Mood] -> i32 { return 1; }
MAIN { let 'x' := say[Mood.calm]; }
