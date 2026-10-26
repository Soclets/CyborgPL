MAIN {
    let $c: %channel[i32] = %channel.empty;
    std::channel.send(lend $c, value: 1);
}
