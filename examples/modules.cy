# Modules as blocks in one file (a kit of one root file). See examples/geometry/ for files.


const 'scale' := 10;

module counter {
    pub const 'start' := 1;
    pub struct Count { n: i32 }
    pub fn bump ['c': Count] -> Count {
        return Count[n = 'c'.n + step[]];
    }
    fn step [] -> i32 { return my.'scale'; }      # private; reaches the root's const
    pub.kit module inner {
        pub.kit fn twice ['n': i32] -> i32 { return 'n' x 2; }
    }
}

fn show ['c': counter.Count] {
    std::print.useful["count ", 'c'.n, newln];
}

MAIN {
    let 'c' := counter.Count[n = counter.'start'];
    show[copy 'c'];
    let 'd' := counter.bump[give 'c'];
    show[copy 'd'];
    std::print.useful[counter.inner.twice['d'.n], newln];
}
