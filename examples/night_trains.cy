# Night trains across Europe: a longer CyborgPL program, written to be read.
#
# It reads a timetable written as text, finds the quickest way between two stations (Dijkstra's
# algorithm, by hand), books travellers onto it, prices each ticket in exact decimals (`float.%deci.64`, so
# money never picks up a binary float's error), turns minutes into arrival times, prints a
# departure board and ends with a report. The network is made up; the arithmetic is not, and every
# figure it prints can be checked by hand from the timetable below. Along the way it shows:
#
#   records, tagged unions with data, `match`, `is`  ownership said aloud: `give`, `copy`, `lend`
#   failures a caller must handle, `otherwise`      a handler, told whom it is apologising to
#   bounded types, `int.sign.32[min: 1, max: 4]`    generics with an interface, `Number`
#   maps, looked up with a lent key                 a type's own iterator, `Clock.next`
#   counting loops, `loop`, `break`, `continue`     a short lambda, `fn [let $s] with [lend $part] = …`
#   tasks and channels, `%spawn.task`               a writer, `std::print.writer`
#   format holes in words, widths in `cols`         rounding money on purpose, `std::round.near`
#   named bounds, `coll.bounded trip`               guards that let a final build prove every site

# ---------------------------------------------------------------------------------------------------
# The fixed facts
# ---------------------------------------------------------------------------------------------------

const $network_name := "Nightjar";
const $departure: i32 := 1290;        # every train leaves at 21:30, counted in minutes from midnight
const $change_minutes: i32 := 10;     # changing trains at a station costs ten minutes
const $near_rate: %d64 := 0.12;        # the fare per km for the first 500 km
const $far_rate: %d64 := 0.08;         # and per km after that
const $near_km: i32 := 500;
const $by_eight: i32 := 1920;           # 08:00 the next morning, 1440 + 480
const $most_bookings: u64 := 500;     # a night's trains hold no more travellers than this

# ---------------------------------------------------------------------------------------------------
# Types
# ---------------------------------------------------------------------------------------------------

# A stretch of line between two stations, run in both directions.
rec Link:
    var a: str.utf8;
    var b: str.utf8;
    var minutes: i32[min: 1, max: 1440];
    var km: i32[min: 1, max: 3000];
;rec Link

rec Network:
    var stations: std::ls[str.utf8];
    var links: std::ls[Link];
;rec Network

# The longest trip the network sells: ten days and 100,000 km. A way past either is no way, and that limit is what
# keeps every sum of minutes and km below provably inside an `int.sign.32`.
coll.bounded trip:
    member trip.minutes = i32[min: 0, max: 14400];
    member trip.km = i32[min: 0, max: 100000];
;coll.bounded trip

# The quickest known way to a station: when it is reached, how far it is, and where it came from.
rec Reach:
    var minutes: trip.minutes;
    var km: trip.km;
    var prev: str.utf8;
;rec Reach

rec Journey:
    var stops: std::ls[str.utf8];
    var minutes: trip.minutes;
    var km: trip.km;
;rec Journey

# Where a traveller sleeps. A sleeper carries how many berths share the compartment.
union.tag Seat:
    seat;
    couchette;
    sleeper: i32;
;union.tag Seat

union.tag Age:
    adult;
    youth;
    senior;
;union.tag Age

rec Request:
    var traveller: str.utf8;
    var origin: str.utf8;
    var destination: str.utf8;
    var seat: Seat;
    var age: Age;
;rec Request

rec Booking:
    var ticket: i32;
    var traveller: str.utf8;
    var journey: Journey;
    var seat: Seat;
    var fare: %d64;
;rec Booking

# ---------------------------------------------------------------------------------------------------
# Small generic helpers
# ---------------------------------------------------------------------------------------------------

set Number:
    i32;
    %d64;
;set Number

sig Addable <T>:
    op + [lent $a: <T>, lent $b: <T>] -> <T>;
;sig Addable

fn total_of <T: Number and Addable> [lent $xs: std::ls[<T>]] -> <T>:
    var $sum: <T> = 0;
    for $x in lend $xs:
        change $sum to $sum + $x;
    ;for
    ret $sum;
;fn total_of

# ---------------------------------------------------------------------------------------------------
# Reading the timetable: one stretch of line per row, "Lisboa - Madrid 540 625", minutes then km
# ---------------------------------------------------------------------------------------------------

fn timetable [] -> std::ls[str.utf8]:
    ret [
        "Lisboa - Madrid 540 625",
        "Madrid - Barcelona 170 621",
        "Barcelona - Paris 395 1035",
        "Paris - Bruxelles 85 312",
        "Bruxelles - Amsterdam 110 210",
        "Bruxelles - Köln 110 220",
        "Amsterdam - Köln 160 260",
        "Paris - Zürich 245 610",
        "Köln - Berlin 260 580",
        "Köln - München 270 590",
        "Zürich - München 230 320",
        "Zürich - Wien 470 750",
        "München - Wien 240 435",
        "Wien - Praha 240 330",
        "Berlin - Praha 260 350",
        "Wien - Kraków 400 440",
        "Praha - Kraków 450 540",
        "Berlin - København 420 440",
        "København - Malmö 40 45",
        "Malmö - Stockholm 270 615",
        "Wien - Budapest two hours",
        "Palermo - Messina 90 230"
    ];
;fn timetable

# "Lisboa - Madrid 540 625" into a link. The two numbers come last, so a station's name may hold spaces.
fn parse_link [lent $row: str.utf8] -> Link or fail[bad_row: str.utf8]:
    let $sides := std::str.split(lend $row, at: " - ");
    if std::ls.len(lend $sides) != 2:
        fail bad_row[copy $row];
    ;if
    let $words := std::str.split(lend $sides[pos: 2], at: " ");
    let $n := std::ls.len(lend $words);
    if $n < 3:
        fail bad_row[copy $row];
    ;if
    let $minutes: i32[min: 1, max: 1440] := to.i32[min: 1, max: 1440](lend $words[pos: $n - 1]) otherwise fail bad_row[copy $row];
    let $km: i32[min: 1, max: 3000] := to.i32[min: 1, max: 3000](lend $words[pos: $n]) otherwise fail bad_row[copy $row];
    let $name_words := std::ls.slice(lend $words, from: pos 1, before: pos $n - 1);
    let $b := std::str.join(lend $name_words, between: " ");
    ret Link[a = copy $sides[pos: 1], b = give $b, minutes = $minutes, km = $km];
;fn parse_link

proc add_station [lent.mut $stations: std::ls[str.utf8], lent $name: str.utf8]:
    for $s in lend $stations:
        if $s == $name:
            ret;
        ;if
    ;for
    std::ls.append(lend.mut $stations, value: copy $name);
;proc add_station

proc read_network [lent $rows: std::ls[str.utf8]] -> Network:
    var $stations: std::ls[str.utf8] = std::ls.empty;
    var $links: std::ls[Link] = std::ls.empty;
    for $row in lend $rows:
        let $link := parse_link(lend $row)-
        otherwise:
            case bad_row $what:
                std::print("  (skipped a row that does not read as \"A - B minutes km\": \"", $what, "\")", newln);
                continue;
            ;case bad_row
        ;otherwise;
        add_station(lend.mut $stations, lend $link.a);
        add_station(lend.mut $stations, lend $link.b);
        std::ls.append(lend.mut $links, value: give $link);
    ;for
    ret Network[stations = give $stations, links = give $links];
;proc read_network

fn knows [lent $net: Network, lent $name: str.utf8] -> bool:
    for $s in lend $net.stations:
        if $s == $name:
            ret true;
        ;if
    ;for
    ret false;
;fn knows

# ---------------------------------------------------------------------------------------------------
# The quickest way: Dijkstra's algorithm. Each round finishes the unfinished station reached soonest,
# then tries every line out of it. Leaving a station the traveller arrived at by another train costs
# a change; leaving the first station does not.
# ---------------------------------------------------------------------------------------------------

proc quickest_from [lent $net: Network, lent $from: str.utf8] -> std::map.ordered[str.utf8, Reach]:
    var $best: std::map.ordered[str.utf8, Reach] = std::map.empty;
    var $done: std::map[str.utf8, bool] = std::map.empty;
    std::map.set(lend.mut $best, key: copy $from, value: Reach[minutes = 0, km = 0, prev = str.empty]);
    loop:
        # the unfinished station reached soonest; the first in the station list wins a tie
        var $found = false;
        var $pick = str.empty;
        var $pick_minutes: trip.minutes = 0;
        var $pick_km: trip.km = 0;
        for $s in lend $net.stations:
            if std::map.has(lend $done, key: lend $s):
                continue;
            ;if
            if not std::map.has(lend $best, key: lend $s):
                continue;
            ;if
            let $r := std::map.get(lend $best, key: lend $s)-
            otherwise:
                case missing:
                    continue;
                ;case missing
            ;otherwise;
            if not $found or.else $r.minutes < $pick_minutes:
                change $found to true;
                change $pick to copy $s;
                change $pick_minutes to $r.minutes;
                change $pick_km to $r.km;
            ;if
        ;for
        if not $found:
            break;
        ;if
        std::map.set(lend.mut $done, key: copy $pick, value: true);
        var $wait = $change_minutes;
        if $pick == $from:
            change $wait to 0;
        ;if
        for $l in lend $net.links:
            let $minutes := $pick_minutes + $wait + $l.minutes;
            let $km := $pick_km + $l.km;
            if $l.a == $pick:
                try_reach(lend.mut $best, lend $done, lend $l.b, lend $pick, $minutes, $km);
            else if $l.b == $pick:
                try_reach(lend.mut $best, lend $done, lend $l.a, lend $pick, $minutes, $km);
            ;if
        ;for
    ;loop
    ret give $best;
;proc quickest_from

# Reaching `$next` from `$pick` in `$minutes`: kept only if no sooner way is known. The map keeps its own copy of
# both names, made here and nowhere else.
proc try_reach [
    lent.mut $best: std::map.ordered[str.utf8, Reach],
    lent $done: std::map[str.utf8, bool],
    lent $next: str.utf8,
    lent $pick: str.utf8,
    let $minutes: i32,
    let $km: i32
]:
    if std::map.has(lend $done, key: lend $next):
        ret;
    ;if
    # past the longest trip sold, it is no way at all
    if $minutes < 0 or $minutes > 14400 or $km < 0 or $km > 100000:
        ret;
    ;if
    if std::map.has(lend $best, key: lend $next):
        let $old := std::map.get(lend $best, key: lend $next)-
        otherwise:
            case missing:
                ret;
            ;case missing
        ;otherwise;
        if $old.minutes <= $minutes:
            ret;
        ;if
    ;if
    std::map.set(lend.mut $best, key: copy $next, value: Reach[minutes = $minutes, km = $km, prev = copy $pick]);
;proc try_reach

# The stops from `$from` to `$to`, walked back from `$to` along each station's `prev`, then turned round.
proc journey [lent $net: Network, lent $from: str.utf8, lent $to: str.utf8] -> Journey or fail[unknown_station: str.utf8, same_station, no_route]:
    if not knows(lend $net, lend $from):
        fail unknown_station[copy $from];
    ;if
    if not knows(lend $net, lend $to):
        fail unknown_station[copy $to];
    ;if
    if $from == $to:
        fail same_station;
    ;if
    let $best := quickest_from(lend $net, lend $from);
    let $end := std::map.get(lend $best, key: lend $to)-
    otherwise:
        case missing:
            fail no_route;
        ;case missing
    ;otherwise;
    var $back: std::ls[str.utf8] = std::ls.empty;
    var $at = copy $to;
    loop:
        std::ls.append(lend.mut $back, value: copy $at);
        if $at == $from:
            break;
        ;if
        let $r := std::map.get(lend $best, key: lend $at)-
        otherwise:
            case missing:
                fail no_route;
            ;case missing
        ;otherwise;
        change $at to copy $r.prev;
    ;loop
    var $stops: std::ls[str.utf8] = std::ls.empty;
    let $n := std::ls.len(lend $back);
    for $k [from: 1, to: $n, by: 1]:
        std::ls.append(lend.mut $stops, value: copy $back[pos: $n - $k + 1]);
    ;for
    ret Journey[stops = give $stops, minutes = $end.minutes, km = $end.km];
;proc journey

# ---------------------------------------------------------------------------------------------------
# Clock times: minutes since midnight of the departure day as "07:40", with "+1" for the next morning
# ---------------------------------------------------------------------------------------------------

fn clock [let $minutes: i32] -> str.utf8:
    let $day := $minutes div.floor 1440;
    let $in_day := $minutes mod.floor 1440;
    let $text := std::format("{zeros, width: 2 cols}:{zeros, width: 2 cols}", $in_day div.floor 60, $in_day mod.floor 60);
    if $day == 0:
        ret give $text;
    ;if
    ret $text concat " +" concat to.stored.str.utf8($day);
;fn clock

# A clock that hands out the next quarter hour each time it is asked, until it reaches its end.
rec Clock:
    var now: i32[min: 0, max: 2895];
    var stop: i32[min: 0, max: 2880];
;rec Clock

proc Clock.next [lent.mut $c: Clock] -> optional[i32]:
    # the stop is never past the next day's end, so neither is the last time handed out
    let $t := $c.now;
    if $t >= $c.stop:
        ret optional.none;
    ;if
    if $t >= 2880:
        ret optional.none;
    ;if
    change $c.now to $t + 15;
    ret optional.value($t);
;proc Clock.next

# ---------------------------------------------------------------------------------------------------
# Fares, in exact decimals
# ---------------------------------------------------------------------------------------------------

fn distance_fare [let $km: i32] -> %d64:
    if $km <= $near_km:
        ret $near_rate x to.%d64($km);
    ;if
    let $far := to.%d64($km - $near_km);
    ret $near_rate x to.%d64($near_km) + $far_rate x $far;
;fn distance_fare

# The bed costs the same per compartment, shared by however many sleep in it.
fn supplement [lent $seat: Seat] -> %d64:
    match $seat:
        case seat:
            ret 0;
        ;case seat
        case couchette:
            ret 25.00;
        ;case couchette
        case sleeper $berths:
            ret 120.00 div.near to.%d64($berths);
        ;case sleeper
    ;match
;fn supplement

fn discount [let $age: Age] -> i32[min: 0, max: 50]:
    let $percent: i32[min: 0, max: 50] := match $age:
        case adult:
            is 0;
        ;case adult
        case youth:
            is 25;
        ;case youth
        case senior:
            is 30;
        ;case senior
    ;match;
    ret $percent;
;fn discount

# The fare before rounding can hold more than two places (30% of 0.15 is 0.045): it is rounded to the
# cent once, at the end, half to even, and said so.
fn fare [let $km: i32, lent $seat: Seat, let $age: Age] -> %d64:
    let $full := distance_fare($km) + supplement(lend $seat);
    let $off := to.%d64(discount($age));
    let $exact := $full - ($full x $off) div.near 100;
    ret std::round.near($exact, places: 2);
;fn fare

fn seat_name [lent $seat: Seat] -> str.utf8:
    match $seat:
        case seat:
            ret "seat";
        ;case seat
        case couchette:
            ret "couchette";
        ;case couchette
        case sleeper $berths:
            ret "sleeper for " concat to.stored.str.utf8($berths);
        ;case sleeper
    ;match
;fn seat_name

fn age_name [let $age: Age] -> str.utf8:
    match $age:
        case adult:
            ret "adult";
        ;case adult
        case youth:
            ret "youth";
        ;case youth
        case senior:
            ret "senior";
        ;case senior
    ;match
;fn age_name

# ---------------------------------------------------------------------------------------------------
# Reading requests: "Ana: Lisboa > Wien, sleeper 2, youth"
# ---------------------------------------------------------------------------------------------------

fn parse_seat [lent $text: str.utf8] -> Seat or fail[bad_request: str.utf8]:
    if $text == "seat":
        ret Seat.seat;
    ;if
    if $text == "couchette":
        ret Seat.couchette;
    ;if
    let $words := std::str.split(lend $text, at: " ");
    if std::ls.len(lend $words) != 2 or.else $words[pos: 1] != "sleeper":
        fail bad_request[copy $text];
    ;if
    let $berths: i32[min: 1, max: 4] := to.i32[min: 1, max: 4](lend $words[pos: 2])-
    otherwise:
        case not_a_number:
            fail bad_request[copy $text];
        ;case not_a_number
        case out_of_range:
            fail bad_request[copy $text];
        ;case out_of_range
    ;otherwise;
    ret Seat.sleeper($berths);
;fn parse_seat

fn parse_age [lent $text: str.utf8] -> Age or fail[bad_request: str.utf8]:
    if $text == "adult":
        ret Age.adult;
    ;if
    if $text == "youth":
        ret Age.youth;
    ;if
    if $text == "senior":
        ret Age.senior;
    ;if
    fail bad_request[copy $text];
;fn parse_age

fn parse_request [lent $line: str.utf8] -> Request or fail[bad_request: str.utf8]:
    let $who := std::str.split(lend $line, at: ": ");
    if std::ls.len(lend $who) != 2:
        fail bad_request[copy $line];
    ;if
    let $parts := std::str.split(lend $who[pos: 2], at: ", ");
    if std::ls.len(lend $parts) != 3:
        fail bad_request[copy $line];
    ;if
    let $ends := std::str.split(lend $parts[pos: 1], at: " > ");
    if std::ls.len(lend $ends) != 2:
        fail bad_request[copy $line];
    ;if
    let $seat := parse_seat(lend $parts[pos: 2])?;
    let $age := parse_age(lend $parts[pos: 3])?;
    ret Request[traveller = copy $who[pos: 1], origin = copy $ends[pos: 1], destination = copy $ends[pos: 2], seat = give $seat, age = $age];
;fn parse_request

fn todays_requests [] -> std::ls[str.utf8]:
    ret [
        "Ana: Lisboa > Wien, sleeper 2, youth",
        "Bram: Amsterdam > Kraków, couchette, adult",
        "Chloé: Paris > Stockholm, seat, senior",
        "Dario: Madrid > Madrid, seat, adult",
        "Eun: Berlin > Palermo, sleeper 1, adult",
        "Femi: Zürich > Oslo, seat, adult",
        "Gil: Praha > Lisboa, sleeper 4, adult",
        "Hana: Köln > Wien sleeper 2",
        "Ines: München > København, sleeper 1, senior",
        "Jo: Wien > Berlin, sleeper 1, youth",
        "Kai: Bruxelles > Zürich, sleeper 3, youth"
    ];
;fn todays_requests

# ---------------------------------------------------------------------------------------------------
# Booking: a journey, a bed of the kind asked for if one is left, and a fare
# ---------------------------------------------------------------------------------------------------

# How many of each kind of place are left on tonight's trains, by `seat_name`.
fn places_left [] -> std::map.ordered[str.utf8, i32]:
    var $left: std::map.ordered[str.utf8, i32] = std::map.empty;
    std::map.set(lend.mut $left, key: "seat", value: 40);
    std::map.set(lend.mut $left, key: "couchette", value: 6);
    std::map.set(lend.mut $left, key: "sleeper for 1", value: 1);
    std::map.set(lend.mut $left, key: "sleeper for 2", value: 4);
    std::map.set(lend.mut $left, key: "sleeper for 3", value: 3);
    std::map.set(lend.mut $left, key: "sleeper for 4", value: 8);
    ret give $left;
;fn places_left

proc book [
    lent $net: Network,
    lent.mut $left: std::map.ordered[str.utf8, i32],
    lent $req: Request,
    let $ticket: i32
] -> Booking or fail[unknown_station: str.utf8, same_station, no_route, sold_out: str.utf8]:
    let $trip := journey(lend $net, lend $req.origin, lend $req.destination)?;
    let $kind := seat_name(lend $req.seat);
    # a kind of place the trains do not carry has none left, said here rather than read as a zero
    let $free := std::map.get(lend $left, key: lend $kind)-
    otherwise:
        case missing:
            fail sold_out[give $kind];
        ;case missing
    ;otherwise;
    if $free <= 0:
        fail sold_out[give $kind];
    ;if
    std::map.set(lend.mut $left, key: copy $kind, value: $free - 1);
    let $price := fare($trip.km, lend $req.seat, $req.age);
    ret Booking[ticket = $ticket, traveller = copy $req.traveller, journey = give $trip, seat = copy $req.seat, fare = $price];
;proc book

# A request that cannot be booked is explained the same way wherever it is tried: one handler, told
# whom it is explaining to. Its arms act as if written at the call, so `continue` moves on to the next
# request there (owner, 2026-10-25).
coll.handler booking_failed:
    member booking_failed.explain [let $who: str.utf8]:
        case unknown_station $name:
            std::print("  ", $who, ": no station called ", $name, " on this network", newln);
            continue;
        ;case unknown_station
        case same_station:
            std::print("  ", $who, ": that journey starts where it ends", newln);
            continue;
        ;case same_station
        case no_route:
            std::print("  ", $who, ": no night train joins those two stations", newln);
            continue;
        ;case no_route
        case sold_out $kind:
            std::print("  ", $who, ": no ", $kind, " left tonight", newln);
            continue;
        ;case sold_out
    ;member booking_failed.explain
;coll.handler booking_failed

proc show_booking [lent $b: Booking, let $age: Age]:
    let $changes := std::ls.len(lend $b.journey.stops) - 2;
    std::print("  #", $b.ticket, " ", $b.traveller, " (", age_name($age), "), ", seat_name(lend $b.seat), newln);
    std::print("    ", std::str.join(lend $b.journey.stops, between: " → "), newln);
    std::print.format("    {} km, {} change(s), leaves {}, arrives {}, fare {places: 2}{newln}", $b.journey.km, $changes, clock($departure), clock($departure + $b.journey.minutes), $b.fare);
;proc show_booking

# ---------------------------------------------------------------------------------------------------
# The departure board: everywhere reachable from one station tonight, soonest first
# ---------------------------------------------------------------------------------------------------

rec Arrival:
    var station: str.utf8;
    var minutes: trip.minutes;
    var km: trip.km;
;rec Arrival

# Soonest first: each arrival goes in before the first one later than it, so arrivals at the same time keep their
# order. The list is written out anew around each one, which needs no positions.
fn soonest_first [lent $xs: std::ls[Arrival]] -> std::ls[Arrival]:
    var $out: std::ls[Arrival] = std::ls.empty;
    for $x in lend $xs:
        var $next: std::ls[Arrival] = std::ls.empty;
        var $placed = false;
        for $y in lend $out:
            if not $placed and $x.minutes < $y.minutes:
                std::ls.append(lend.mut $next, value: copy $x);
                change $placed to true;
            ;if
            std::ls.append(lend.mut $next, value: copy $y);
        ;for
        if not $placed:
            std::ls.append(lend.mut $next, value: copy $x);
        ;if
        change $out to give $next;
    ;for
    ret give $out;
;fn soonest_first

proc board [lent $net: Network, lent $from: str.utf8]:
    var $out = std::print.writer(size: 4096 bytes);
    std::print(lend.mut $out, newln, "departures from ", $from, " at ", clock($departure), newln);
    std::print.format(lend.mut $out, "  {align: left, width: 12 cols} {align: left, width: 8 cols} {align: right, width: 6 cols}{newln}", "to", "arrives", "km");
    let $best := quickest_from(lend $net, lend $from);
    var $arrivals: std::ls[Arrival] = std::ls.empty;
    for $e in std::map.pairs(lend $best):
        if $e.key == $from:
            continue;
        ;if
        std::ls.append(lend.mut $arrivals, value: Arrival[station = copy $e.key, minutes = $e.value.minutes, km = $e.value.km]);
    ;for
    let $sorted := soonest_first(lend $arrivals);
    for $a in lend $sorted:
        std::print.format(lend.mut $out, "  {align: left, width: 12 cols} {width: 8 cols} {width: 6 cols}{newln}", $a.station, clock($departure + $a.minutes), $a.km);
    ;for
    std::print.flush(lend.mut $out);
;proc board

# ---------------------------------------------------------------------------------------------------
# Counting at the same time: how many stations each hub reaches by eight in the morning, each hub
# worked out by its own task, which reports back over a channel
# ---------------------------------------------------------------------------------------------------

proc reached_by [lent $net: Network, lent $hub: str.utf8, let $deadline: i32] -> u64:
    let $best := quickest_from(lend $net, lend $hub);
    var $reached: std::ls[str.utf8] = std::ls.empty;
    for $e in std::map.pairs(lend $best):
        if $e.key != $hub and $departure + $e.value.minutes <= $deadline:
            std::ls.append(lend.mut $reached, value: copy $e.key);
        ;if
    ;for
    ret std::ls.len(lend $reached);
;proc reached_by

proc compare_hubs [lent $net: Network]:
    let $paris_ch: %channel[u64] := %channel.empty;
    let $koln_ch: %channel[u64] := %channel.empty;
    let $wien_ch: %channel[u64] := %channel.empty;
    let $paris := "Paris";
    let $koln := "Köln";
    let $wien := "Wien";
    %spawn.task with [lend $paris_ch, lend $net, lend $paris]:
        %channel.send(lend $paris_ch, value: reached_by(lend $net, lend $paris, $by_eight));
    ;%spawn.task
    %spawn.task with [lend $koln_ch, lend $net, lend $koln]:
        %channel.send(lend $koln_ch, value: reached_by(lend $net, lend $koln, $by_eight));
    ;%spawn.task
    %spawn.task with [lend $wien_ch, lend $net, lend $wien]:
        %channel.send(lend $wien_ch, value: reached_by(lend $net, lend $wien, $by_eight));
    ;%spawn.task
    let $p := %channel.take(lend $paris_ch) ?? 0;
    let $k := %channel.take(lend $koln_ch) ?? 0;
    let $w := %channel.take(lend $wien_ch) ?? 0;
    std::print(newln, "stations reached by ", clock($by_eight), ": Paris ", $p, ", Köln ", $k, ", Wien ", $w, newln);
;proc compare_hubs

# ---------------------------------------------------------------------------------------------------
# Looking things up: the stations whose names hold a given letter
# ---------------------------------------------------------------------------------------------------

fn stations_where [lent $net: Network, let $keep: fn[str.utf8] -> bool] -> std::ls[str.utf8]:
    var $out: std::ls[str.utf8] = std::ls.empty;
    for $s in lend $net.stations:
        if $keep(copy $s):
            std::ls.append(lend.mut $out, value: copy $s);
        ;if
    ;for
    ret give $out;
;fn stations_where

proc show_letters [lent $net: Network]:
    std::print(newln, "stations by letter:", newln);
    let $letters := ["ö", "ü", "ó", "x"];
    for $part in lend $letters:
        let $found := stations_where(lend $net, fn [let $s] with [lend $part] = std::str.contains(lend $s, part: lend $part));
        if std::ls.len(lend $found) == 0:
            std::print("  none with ", $part, newln);
        else:
            std::print("  with ", $part, ": ", std::str.join(lend $found, between: ", "), newln);
        ;if
    ;for
;proc show_letters

# ---------------------------------------------------------------------------------------------------
# The report
# ---------------------------------------------------------------------------------------------------

proc report [lent $bookings: std::ls[Booking][max: 500], lent $left: std::map.ordered[str.utf8, i32]]:
    var $out = std::print.writer(size: 4096 bytes);
    std::print(lend.mut $out, newln, "=== ", $network_name, ", tonight ===", newln);
    var $fares: std::ls[%d64][max: 500] = std::ls.empty;
    var $kms: std::ls[i32][max: 500] = std::ls.empty;
    var $longest = 0;
    var $longest_who = "nobody";
    for $b in lend $bookings:
        std::ls.append(lend.mut $fares, value: $b.fare);
        std::ls.append(lend.mut $kms, value: $b.journey.km);
        if $b.journey.km > $longest:
            change $longest to $b.journey.km;
            change $longest_who to copy $b.traveller;
        ;if
    ;for
    let $count := std::ls.len(lend $bookings);
    let $takings := total_of(lend $fares);
    std::print(lend.mut $out, "tickets sold: ", $count, newln);
    std::print.format(lend.mut $out, "takings: {places: 2}{newln}", $takings);
    if $count > 0:
        let $average := std::round.near($takings div.near to.%d64(to.i32($count)), places: 2);
        std::print.format(lend.mut $out, "average fare: {places: 2}{newln}", $average);
    ;if
    std::print(lend.mut $out, "km travelled: ", total_of(lend $kms), ", the longest ", $longest, " km (", $longest_who, ")", newln);
    std::print(lend.mut $out, "places left:", newln);
    for $e in std::map.pairs(lend $left):
        std::print.format(lend.mut $out, "  {align: left, width: 14 cols} {width: 3 cols}{newln}", $e.key, $e.value);
    ;for
    std::print.flush(lend.mut $out);
;proc report

# Every request of the day, each with the next ticket number; the ones that cannot be read or booked are explained.
proc book_all [lent $net: Network, lent.mut $left: std::map.ordered[str.utf8, i32], lent $lines: std::ls[str.utf8]] -> std::ls[Booking][max: 500]:
    std::print(newln, "bookings:", newln);
    var $bookings: std::ls[Booking][max: 500] = std::ls.empty;
    var $ticket = 500;
    for $line in lend $lines:
        if std::ls.len(lend $bookings) >= $most_bookings:
            std::print("  (tonight's trains are full)", newln);
            break;
        ;if
        let $req := parse_request(lend $line)-
        otherwise:
            case bad_request $what:
                if $what == $line:
                    std::print("  (cannot read \"", $line, "\")", newln);
                else:
                    std::print("  (cannot read \"", $what, "\" in \"", $line, "\")", newln);
                ;if
                continue;
            ;case bad_request
        ;otherwise;
        let $b := book(lend $net, lend.mut $left, lend $req, $ticket + 1) otherwise booking_failed.*explain(who: copy $req.traveller);
        change $ticket to $ticket + 1;
        show_booking(lend $b, $req.age);
        std::ls.append(lend.mut $bookings, value: give $b);
    ;for
    ret give $bookings;
;proc book_all

# The night's quarter hours, from departure to midnight, walked by the clock's own iterator.
proc show_quarter_hours []:
    std::print(newln, "the last quarter hours before midnight:");
    var $quarters = Clock[now = $departure, stop = 1440];
    for $t in lend.mut $quarters:
        std::print(" ", clock($t));
    ;for
    std::print(newln);
;proc show_quarter_hours

# Exact decimals: why a fare is a `float.%deci.64`. A fare of 0.10 three times is 0.30 exactly.
proc show_exact_money []:
    let $dime: %d64 := 0.10;
    let $three := $dime + $dime + $dime;
    let $float: f64 := 0.1 + 0.1 + 0.1;
    std::print("three fares of 0.10 as float.deci.64: ");
    std::print.stored($three, newln);
    std::print("the same in float.bin.64: ");
    std::print.stored($float, newln);
;proc show_exact_money

# ---------------------------------------------------------------------------------------------------
# The night: read the timetable, then each part in turn
# ---------------------------------------------------------------------------------------------------

MAIN:
    std::print("the ", $network_name, " night trains", newln, newln, "reading the timetable:", newln);
    let $rows := timetable();
    let $net := read_network(lend $rows);
    std::print("  ", std::ls.len(lend $net.links), " lines between ", std::ls.len(lend $net.stations), " stations", newln);
    board(lend $net, "Wien");
    var $left = places_left();
    let $requests := todays_requests();
    let $bookings := book_all(lend $net, lend.mut $left, lend $requests);
    report(lend $bookings, lend $left);
    compare_hubs(lend $net);
    show_letters(lend $net);
    show_quarter_hours();
    show_exact_money();
    std::print("good night from ", $network_name, newln);
;MAIN
