%extern.C fn exit [$code: i32];
fn produce [lent $out: %channel[std::list[i32]][min: 0, max: 4], $base: i32] {
    for $i: i32 [from: 1, to: 50, by: 1] {
        let $batch: std::list[i32] = [$base, $i];
        %channel.send(lend $out, value: give $batch);
    }
}
MAIN {
    let $pipe: %channel[std::list[i32]][min: 0, max: 4] = %channel.empty;
    let $quiet: %channel[i32] = %channel.empty;
    %concur with [lend $pipe] { produce(lend $pipe, 1000); }
    %concur with [lend $pipe] { produce(lend $pipe, 2000); }
    %concur with [lend $quiet] { %channel.send(lend $quiet, value: 7); }
    let mut $batches: i32 = 0;
    let mut $sevens: i32 = 0;
    let mut $closed: i32 = 0;
    for $k: i32 [from: 1, to: 102, by: 1] {
        %whichever {
            $b from lend $pipe  { change $batches = $batches plus to.i32(std::list.length(lend $b)) minus 1; }
            $s from lend $quiet { change $sevens = $sevens plus $s; }
        } otherwise { closed { change $closed = $closed plus 1; } };
    }
    no_check my.exit($batches plus $sevens plus $closed times 10);
}
