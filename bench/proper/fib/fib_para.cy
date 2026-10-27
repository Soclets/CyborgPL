# In parallel: the top of the recursion as four `concur` tasks, each sending its part on a channel;
# below that, naive recursion as the program asks.
# The Nth Fibonacci number, recursively; the exit code is its low byte. N
# comes from the environment and must be 0 to 92 — fib(93) does not fit an
# int64 — and anything else ends the program with exit code 2, as does no N at all.
%external.C procedure exit [$code: int32];
%external.C procedure getenv [$name: pointer[uint8]] -> pointer[uint8];
%external.C procedure atoll [$s: pointer[uint8]] -> int64[min 0, max 92];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
procedure read_n [$text: pointer[uint8]] -> int64[min 0, max 92] or fail[missing, out_of_range]:
    if pointer.is_null[$text]:
        fail missing;
    end of if
    !no_check let $n := atoll[$text] otherwise fail;
    return $n;
end of procedure read_n

function fib [$n: int64[min 0, max 92]] -> int64:
    if $n less_than 2:
        return $n;
    end of if
    return fib[$n minus 1] plus fib[$n minus 2];
end of function fib

procedure run [] -> int32 or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n[getenv[lend $key]] otherwise fail;
    if $n less_than 4:
        return to.int32[fib[$n] modulo.floor 256];
    end of if
    # fib(n) = fib(n-2) + 2 fib(n-3) + fib(n-4): four tasks, each on its own channel
    let $c1 := %channel[int64];
    let $c2 := %channel[int64];
    let $c3 := %channel[int64];
    let $c4 := %channel[int64];
    %spawn.task with [lend $c1, lend $n]:
        %channel.send[lend $c1, value fib[$n minus 2]];
    end of %spawn.task
    %spawn.task with [lend $c2, lend $n]:
        %channel.send[lend $c2, value fib[$n minus 3]];
    end of %spawn.task
    %spawn.task with [lend $c3, lend $n]:
        %channel.send[lend $c3, value fib[$n minus 3]];
    end of %spawn.task
    %spawn.task with [lend $c4, lend $n]:
        %channel.send[lend $c4, value fib[$n minus 4]];
    end of %spawn.task
    let $a := %channel.take[lend $c1] otherwise 0;
    let $b := %channel.take[lend $c2] otherwise 0;
    let $c := %channel.take[lend $c3] otherwise 0;
    let $d := %channel.take[lend $c4] otherwise 0;
    # each part's low byte first: what comes off a channel could be any int64
    return to.int32[(($a modulo.floor 256) plus ($b modulo.floor 256) plus ($c modulo.floor 256) plus ($d modulo.floor 256)) modulo.floor 256];
end of procedure run

!MAIN:
    let $code := run[] otherwise 2;
    !no_check my.exit[$code];
!end of MAIN
