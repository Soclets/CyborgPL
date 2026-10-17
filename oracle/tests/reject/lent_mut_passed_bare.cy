fn add1 [lent.mut 'out': list[int32]] { list.append[lend.mut 'out', value 1]; }
fn walk [lent.mut 'out': list[int32]] { add1['out']; list.append[lend.mut 'out', value 2]; }
MAIN { let mut 'xs': list[int32] = []; walk[lend.mut 'xs']; std::print.useful[list.length[lend 'xs'], newln]; }
