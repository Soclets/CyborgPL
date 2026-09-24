//! The span-recording renderer writes exactly what the plain one writes, and
//! every span it records points at text that is the node it names.

use check_gen::{generate, Config};

#[test]
fn same_text_and_spans_hit_the_node() {
    for seed in 1..400u64 {
        for (inputs, exhaust, loose_risk) in [(0, 0, 5), (2, 0, 40), (2, 4096, 5)] {
            let cfg = Config { inputs, exhaust, risk: loose_risk, ..Config::default() };
            let p = generate(seed, &cfg);
            let (text, spans) = p.render_spans();
            assert_eq!(text, p.render(), "seed {seed}: the two renderers differ");
            let lines: Vec<&str> = text.lines().collect();
            for (_, (l, c, e)) in &spans.expr {
                let line = lines[*l as usize - 1];
                assert!(*c >= 1 && *e as usize <= line.len() + 1 && c < e, "seed {seed}: bad span {l}:{c}..{e}");
            }
            for (_, (l, _c0, _c1, e)) in &spans.stmt {
                let line = lines[*l as usize - 1];
                assert_eq!(&line[*e as usize - 2..*e as usize - 1], ";", "seed {seed}: a statement span must end at its `;`");
            }
        }
    }
}
