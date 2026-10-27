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
const $departure: int.sign.32 := 1290;        # every train leaves at 21:30, counted in minutes from midnight
const $change_minutes: int.sign.32 := 10;     # changing trains at a station costs ten minutes
const $near_rate: float.%deci.64 := 0.12;        # the fare per km for the first 500 km
const $far_rate: float.%deci.64 := 0.08;         # and per km after that
const $near_km: int.sign.32 := 500;
const $by_eight: int.sign.32 := 1920;           # 08:00 the next morning, 1440 + 480
const $most_bookings: int.unsign.64 := 500;     # a night's trains hold no more travellers than this

# ---------------------------------------------------------------------------------------------------
# Types
# ---------------------------------------------------------------------------------------------------

# A stretch of line between two stations, run in both directions.
rec Link:
    var a: str.utf8;
    var b: str.utf8;
    var minutes: int.sign.32[min: 1, max: 1440];
    var km: int.sign.32[min: 1, max: 3000];
end of rec Link

rec Network:
    var stations: std::ls[str.utf8];
    var links: std::ls[Link];
end of rec Network

# The longest trip the network sells: ten days and 100,000 km. A way past either is no way, and that limit is what
# keeps every sum of minutes and km below provably inside an `int.sign.32`.
coll.bounded trip:
    member trip.minutes = int.sign.32[min: 0, max: 14400];
    member trip.km = int.sign.32[min: 0, max: 100000];
end of coll.bounded trip

# The quickest known way to a station: when it is reached, how far it is, and where it came from.
rec Reach:
    var minutes: trip.minutes;
    var km: trip.km;
    var prev: str.utf8;
end of rec Reach

rec Journey:
    var stops: std::ls[str.utf8];
    var minutes: trip.minutes;
    var km: trip.km;
end of rec Journey

# Where a traveller sleeps. A sleeper carries how many berths share the compartment.
union.tag Seat:
    seat;
    couchette;
    sleeper: int.sign.32;
end of union.tag Seat

union.tag Age:
    adult;
    youth;
    senior;
end of union.tag Age

rec Request:
    var traveller: str.utf8;
    var origin: str.utf8;
    var destination: str.utf8;
    var seat: Seat;
    var age: Age;
end of rec Request

rec Booking:
    var ticket: int.sign.32;
    var traveller: str.utf8;
    var journey: Journey;
    var seat: Seat;
    var fare: float.%deci.64;
end of rec Booking

# ---------------------------------------------------------------------------------------------------
# Small generic helpers
# ---------------------------------------------------------------------------------------------------

interface Number:
    int.sign.32;
    float.%deci.64;
end of interface Number

fn total_of <T: Number> [lent $xs: std::ls[<T>]] -> <T>:
    var $sum: <T> = 0;
    for $x in lend $xs:
        change $sum to $sum plus $x;
    end of for
    ret $sum;
end of fn total_of

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
end of fn timetable

# "Lisboa - Madrid 540 625" into a link. The two numbers come last, so a station's name may hold spaces.
fn parse_link [lent $row: str.utf8] -> Link or fail[bad_row: str.utf8]:
    let $sides := std::str.split(lend $row, at: " - ");
    if std::ls.len(lend $sides) not_eq 2:
        fail bad_row[copy $row];
    end of if
    let $words := std::str.split(lend $sides[pos: 2], at: " ");
    let $n := std::ls.len(lend $words);
    if $n lt 3:
        fail bad_row[copy $row];
    end of if
    let $minutes: int.sign.32[min: 1, max: 1440] := to.int.sign.32[min: 1, max: 1440]($words[pos: $n minus 1])-
    otherwise:
        case not_a_number:
            fail bad_row[copy $row];
        end of case not_a_number
        case out_of_range:
            fail bad_row[copy $row];
        end of case out_of_range
    end of otherwise;
    let $km: int.sign.32[min: 1, max: 3000] := to.int.sign.32[min: 1, max: 3000]($words[pos: $n])-
    otherwise:
        case not_a_number:
            fail bad_row[copy $row];
        end of case not_a_number
        case out_of_range:
            fail bad_row[copy $row];
        end of case out_of_range
    end of otherwise;
    let $name_words := std::ls.slice(lend $words, from: pos 1, before: pos $n minus 1);
    let $b := std::str.join(lend $name_words, between: " ");
    ret Link[a = copy $sides[pos: 1], b = give $b, minutes = $minutes, km = $km];
end of fn parse_link

proc add_station [lent.mut $stations: std::ls[str.utf8], lent $name: str.utf8]:
    for $s in lend $stations:
        if $s eq $name:
            ret;
        end of if
    end of for
    std::ls.append(lend.mut $stations, value: copy $name);
end of proc add_station

proc read_network [lent $rows: std::ls[str.utf8]] -> Network:
    var $stations: std::ls[str.utf8] = std::ls.empty;
    var $links: std::ls[Link] = std::ls.empty;
    for $row in lend $rows:
        let $link := parse_link(lend $row)-
        otherwise:
            !case bad_row $what:
                std::print.shortest("  (skipped a row that does not read as \"A - B minutes km\": \"", $what, "\")", newln);
                continue;
            !end of case bad_row
        end of otherwise;
        add_station(lend.mut $stations, lend $link.a);
        add_station(lend.mut $stations, lend $link.b);
        std::ls.append(lend.mut $links, value: give $link);
    end of for
    ret Network[stations = give $stations, links = give $links];
end of proc read_network

fn knows [lent $net: Network, lent $name: str.utf8] -> bool:
    for $s in lend $net.stations:
        if $s eq $name:
            ret true;
        end of if
    end of for
    ret false;
end of fn knows

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
            end of if
            if not std::map.has(lend $best, key: lend $s):
                continue;
            end of if
            let $r := std::map.get(lend $best, key: lend $s)-
            otherwise:
                !case missing:
                    continue;
                !end of case missing
            end of otherwise;
            if not $found or.else $r.minutes lt $pick_minutes:
                change $found to true;
                change $pick to copy $s;
                change $pick_minutes to $r.minutes;
                change $pick_km to $r.km;
            end of if
        end of for
        if not $found:
            break;
        end of if
        std::map.set(lend.mut $done, key: copy $pick, value: true);
        var $wait = $change_minutes;
        if $pick eq $from:
            change $wait to 0;
        end of if
        for $l in lend $net.links:
            let $minutes := $pick_minutes plus $wait plus $l.minutes;
            let $km := $pick_km plus $l.km;
            if $l.a eq $pick:
                try_reach(lend.mut $best, lend $done, lend $l.b, lend $pick, $minutes, $km);
            end of if
            else if $l.b eq $pick:
                try_reach(lend.mut $best, lend $done, lend $l.a, lend $pick, $minutes, $km);
            end of else if
        end of for
    end of loop
    ret give $best;
end of proc quickest_from

# Reaching `$next` from `$pick` in `$minutes`: kept only if no sooner way is known. The map keeps its own copy of
# both names, made here and nowhere else.
proc try_reach [
    lent.mut $best: std::map.ordered[str.utf8, Reach],
    lent $done: std::map[str.utf8, bool],
    lent $next: str.utf8,
    lent $pick: str.utf8,
    let $minutes: int.sign.32,
    let $km: int.sign.32
]:
    if std::map.has(lend $done, key: lend $next):
        ret;
    end of if
    # past the longest trip sold, it is no way at all
    if $minutes lt 0 or $minutes gt 14400 or $km lt 0 or $km gt 100000:
        ret;
    end of if
    if std::map.has(lend $best, key: lend $next):
        let $old := std::map.get(lend $best, key: lend $next)-
        otherwise:
            !case missing:
                ret;
            !end of case missing
        end of otherwise;
        if $old.minutes at_most $minutes:
            ret;
        end of if
    end of if
    std::map.set(lend.mut $best, key: copy $next, value: Reach[minutes = $minutes, km = $km, prev = copy $pick]);
end of proc try_reach

# The stops from `$from` to `$to`, walked back from `$to` along each station's `prev`, then turned round.
proc journey [lent $net: Network, lent $from: str.utf8, lent $to: str.utf8] -> Journey or fail[unknown_station: str.utf8, same_station, no_route]:
    if not knows(lend $net, lend $from):
        fail unknown_station[copy $from];
    end of if
    if not knows(lend $net, lend $to):
        fail unknown_station[copy $to];
    end of if
    if $from eq $to:
        fail same_station;
    end of if
    let $best := quickest_from(lend $net, lend $from);
    let $end := std::map.get(lend $best, key: lend $to)-
    otherwise:
        case missing:
            fail no_route;
        end of case missing
    end of otherwise;
    var $back: std::ls[str.utf8] = std::ls.empty;
    var $at = copy $to;
    loop:
        std::ls.append(lend.mut $back, value: copy $at);
        if $at eq $from:
            break;
        end of if
        let $r := std::map.get(lend $best, key: lend $at)-
        otherwise:
            case missing:
                fail no_route;
            end of case missing
        end of otherwise;
        change $at to copy $r.prev;
    end of loop
    var $stops: std::ls[str.utf8] = std::ls.empty;
    let $n := std::ls.len(lend $back);
    for $k [from: 1, to: $n, by: 1]:
        std::ls.append(lend.mut $stops, value: copy $back[pos: $n minus $k plus 1]);
    end of for
    ret Journey[stops = give $stops, minutes = $end.minutes, km = $end.km];
end of proc journey

# ---------------------------------------------------------------------------------------------------
# Clock times: minutes since midnight of the departure day as "07:40", with "+1" for the next morning
# ---------------------------------------------------------------------------------------------------

fn clock [let $minutes: int.sign.32] -> str.utf8:
    let $day := $minutes div.floor 1440;
    let $in_day := $minutes mod.floor 1440;
    let $text := std::format("{zeros, width: 2 cols}:{zeros, width: 2 cols}", $in_day div.floor 60, $in_day mod.floor 60);
    if $day eq 0:
        ret give $text;
    end of if
    ret $text concat " +" concat to.stored.str.utf8($day);
end of fn clock

# A clock that hands out the next quarter hour each time it is asked, until it reaches its end.
rec Clock:
    var now: int.sign.32[min: 0, max: 2895];
    var stop: int.sign.32[min: 0, max: 2880];
end of rec Clock

proc Clock.next [lent.mut $c: Clock] -> optional[int.sign.32]:
    # the stop is never past the next day's end, so neither is the last time handed out
    let $t := $c.now;
    if $t at_least $c.stop:
        ret optional.none;
    end of if
    if $t at_least 2880:
        ret optional.none;
    end of if
    change $c.now to $t plus 15;
    ret optional.value($t);
end of proc Clock.next

# ---------------------------------------------------------------------------------------------------
# Fares, in exact decimals
# ---------------------------------------------------------------------------------------------------

fn distance_fare [let $km: int.sign.32] -> float.%deci.64:
    if $km at_most $near_km:
        ret $near_rate times to.float.%deci.64($km);
    end of if
    let $far := to.float.%deci.64($km minus $near_km);
    ret $near_rate times to.float.%deci.64($near_km) plus $far_rate times $far;
end of fn distance_fare

# The bed costs the same per compartment, shared by however many sleep in it.
fn supplement [lent $seat: Seat] -> float.%deci.64:
    match $seat:
        case seat:
            ret 0;
        end of case seat
        case couchette:
            ret 25.00;
        end of case couchette
        case sleeper $berths:
            ret 120.00 div.near to.float.%deci.64($berths);
        end of case sleeper
    end of match
end of fn supplement

fn discount [let $age: Age] -> int.sign.32[min: 0, max: 50]:
    let $percent: int.sign.32[min: 0, max: 50] := match $age:
        case adult:
            is 0;
        end of case adult
        case youth:
            is 25;
        end of case youth
        case senior:
            is 30;
        end of case senior
    end of match;
    ret $percent;
end of fn discount

# The fare before rounding can hold more than two places (30% of 0.15 is 0.045): it is rounded to the
# cent once, at the end, half to even, and said so.
fn fare [let $km: int.sign.32, lent $seat: Seat, let $age: Age] -> float.%deci.64:
    let $full := distance_fare($km) plus supplement(lend $seat);
    let $off := to.float.%deci.64(discount(copy $age));
    let $exact := $full minus ($full times $off) div.near 100;
    ret std::round.near($exact, places: 2);
end of fn fare

fn seat_name [lent $seat: Seat] -> str.utf8:
    match $seat:
        case seat:
            ret "seat";
        end of case seat
        case couchette:
            ret "couchette";
        end of case couchette
        case sleeper $berths:
            ret "sleeper for " concat to.stored.str.utf8($berths);
        end of case sleeper
    end of match
end of fn seat_name

fn age_name [let $age: Age] -> str.utf8:
    match $age:
        case adult:
            ret "adult";
        end of case adult
        case youth:
            ret "youth";
        end of case youth
        case senior:
            ret "senior";
        end of case senior
    end of match
end of fn age_name

# ---------------------------------------------------------------------------------------------------
# Reading requests: "Ana: Lisboa > Wien, sleeper 2, youth"
# ---------------------------------------------------------------------------------------------------

fn parse_seat [lent $text: str.utf8] -> Seat or fail[bad_request: str.utf8]:
    if $text eq "seat":
        ret Seat.seat;
    end of if
    if $text eq "couchette":
        ret Seat.couchette;
    end of if
    let $words := std::str.split(lend $text, at: " ");
    if std::ls.len(lend $words) not_eq 2 or.else $words[pos: 1] not_eq "sleeper":
        fail bad_request[copy $text];
    end of if
    let $berths: int.sign.32[min: 1, max: 4] := to.int.sign.32[min: 1, max: 4]($words[pos: 2])-
    otherwise:
        case not_a_number:
            fail bad_request[copy $text];
        end of case not_a_number
        case out_of_range:
            fail bad_request[copy $text];
        end of case out_of_range
    end of otherwise;
    ret Seat.sleeper($berths);
end of fn parse_seat

fn parse_age [lent $text: str.utf8] -> Age or fail[bad_request: str.utf8]:
    if $text eq "adult":
        ret Age.adult;
    end of if
    if $text eq "youth":
        ret Age.youth;
    end of if
    if $text eq "senior":
        ret Age.senior;
    end of if
    fail bad_request[copy $text];
end of fn parse_age

fn parse_request [lent $line: str.utf8] -> Request or fail[bad_request: str.utf8]:
    let $who := std::str.split(lend $line, at: ": ");
    if std::ls.len(lend $who) not_eq 2:
        fail bad_request[copy $line];
    end of if
    let $parts := std::str.split(lend $who[pos: 2], at: ", ");
    if std::ls.len(lend $parts) not_eq 3:
        fail bad_request[copy $line];
    end of if
    let $ends := std::str.split(lend $parts[pos: 1], at: " > ");
    if std::ls.len(lend $ends) not_eq 2:
        fail bad_request[copy $line];
    end of if
    let $seat := parse_seat(lend $parts[pos: 2]) otherwise fail;
    let $age := parse_age(lend $parts[pos: 3]) otherwise fail;
    ret Request[traveller = copy $who[pos: 1], origin = copy $ends[pos: 1], destination = copy $ends[pos: 2], seat = give $seat, age = give $age];
end of fn parse_request

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
end of fn todays_requests

# ---------------------------------------------------------------------------------------------------
# Booking: a journey, a bed of the kind asked for if one is left, and a fare
# ---------------------------------------------------------------------------------------------------

# How many of each kind of place are left on tonight's trains, by `seat_name`.
fn places_left [] -> std::map.ordered[str.utf8, int.sign.32]:
    var $left: std::map.ordered[str.utf8, int.sign.32] = std::map.empty;
    std::map.set(lend.mut $left, key: "seat", value: 40);
    std::map.set(lend.mut $left, key: "couchette", value: 6);
    std::map.set(lend.mut $left, key: "sleeper for 1", value: 1);
    std::map.set(lend.mut $left, key: "sleeper for 2", value: 4);
    std::map.set(lend.mut $left, key: "sleeper for 3", value: 3);
    std::map.set(lend.mut $left, key: "sleeper for 4", value: 8);
    ret give $left;
end of fn places_left

proc book [
    lent $net: Network,
    lent.mut $left: std::map.ordered[str.utf8, int.sign.32],
    lent $req: Request,
    let $ticket: int.sign.32
] -> Booking or fail[unknown_station: str.utf8, same_station, no_route, sold_out: str.utf8]:
    let $trip := journey(lend $net, lend $req.origin, lend $req.destination) otherwise fail;
    let $kind := seat_name(lend $req.seat);
    # a kind of place the trains do not carry has none left, said here rather than read as a zero
    let $free := std::map.get(lend $left, key: lend $kind)-
    otherwise:
        case missing:
            fail sold_out[give $kind];
        end of case missing
    end of otherwise;
    if $free at_most 0:
        fail sold_out[give $kind];
    end of if
    std::map.set(lend.mut $left, key: copy $kind, value: $free minus 1);
    let $price := fare($trip.km, lend $req.seat, copy $req.age);
    ret Booking[ticket = $ticket, traveller = copy $req.traveller, journey = give $trip, seat = copy $req.seat, fare = $price];
end of proc book

# A request that cannot be booked is explained the same way wherever it is tried: one handler, told
# whom it is explaining to. Its arms act as if written at the call, so `continue` moves on to the next
# request there (owner, 2026-10-25).
coll.handler booking_failed:
    member booking_failed.explain [let $who: str.utf8]:
        !case unknown_station $name:
            std::print.shortest("  ", $who, ": no station called ", $name, " on this network", newln);
            continue;
        !end of case unknown_station
        !case same_station:
            std::print.shortest("  ", $who, ": that journey starts where it ends", newln);
            continue;
        !end of case same_station
        !case no_route:
            std::print.shortest("  ", $who, ": no night train joins those two stations", newln);
            continue;
        !end of case no_route
        !case sold_out $kind:
            std::print.shortest("  ", $who, ": no ", $kind, " left tonight", newln);
            continue;
        !end of case sold_out
    end of member booking_failed.explain
end of coll.handler booking_failed

proc show_booking [lent $b: Booking, let $age: Age]:
    let $changes := std::ls.len(lend $b.journey.stops) minus 2;
    std::print.shortest("  #", $b.ticket, " ", $b.traveller, " (", age_name(copy $age), "), ", seat_name(lend $b.seat), newln);
    std::print.shortest("    ", std::str.join(lend $b.journey.stops, between: " → "), newln);
    std::print.format("    {} km, {} change(s), leaves {}, arrives {}, fare {places: 2}{newln}", $b.journey.km, $changes, clock($departure), clock($departure plus $b.journey.minutes), $b.fare);
end of proc show_booking

# ---------------------------------------------------------------------------------------------------
# The departure board: everywhere reachable from one station tonight, soonest first
# ---------------------------------------------------------------------------------------------------

rec Arrival:
    var station: str.utf8;
    var minutes: trip.minutes;
    var km: trip.km;
end of rec Arrival

# Soonest first: each arrival goes in before the first one later than it, so arrivals at the same time keep their
# order. The list is written out anew around each one, which needs no positions.
fn soonest_first [lent $xs: std::ls[Arrival]] -> std::ls[Arrival]:
    var $out: std::ls[Arrival] = std::ls.empty;
    for $x in lend $xs:
        var $next: std::ls[Arrival] = std::ls.empty;
        var $placed = false;
        for $y in lend $out:
            if not $placed and $x.minutes lt $y.minutes:
                std::ls.append(lend.mut $next, value: copy $x);
                change $placed to true;
            end of if
            std::ls.append(lend.mut $next, value: copy $y);
        end of for
        if not $placed:
            std::ls.append(lend.mut $next, value: copy $x);
        end of if
        change $out to give $next;
    end of for
    ret give $out;
end of fn soonest_first

proc board [lent $net: Network, lent $from: str.utf8]:
    var $out = std::print.writer(size: 4096 bytes);
    std::print.shortest(lend.mut $out, newln, "departures from ", $from, " at ", clock($departure), newln);
    std::print.format(lend.mut $out, "  {align: left, width: 12 cols} {align: left, width: 8 cols} {align: right, width: 6 cols}{newln}", "to", "arrives", "km");
    let $best := quickest_from(lend $net, lend $from);
    var $arrivals: std::ls[Arrival] = std::ls.empty;
    for $e in std::map.pairs(lend $best):
        if $e.key eq $from:
            continue;
        end of if
        std::ls.append(lend.mut $arrivals, value: Arrival[station = copy $e.key, minutes = $e.value.minutes, km = $e.value.km]);
    end of for
    let $sorted := soonest_first(lend $arrivals);
    for $a in lend $sorted:
        std::print.format(lend.mut $out, "  {align: left, width: 12 cols} {width: 8 cols} {width: 6 cols}{newln}", $a.station, clock($departure plus $a.minutes), $a.km);
    end of for
    std::print.flush(lend.mut $out);
end of proc board

# ---------------------------------------------------------------------------------------------------
# Counting at the same time: how many stations each hub reaches by eight in the morning, each hub
# worked out by its own task, which reports back over a channel
# ---------------------------------------------------------------------------------------------------

proc reached_by [lent $net: Network, lent $hub: str.utf8, let $deadline: int.sign.32] -> int.unsign.64:
    let $best := quickest_from(lend $net, lend $hub);
    var $reached: std::ls[str.utf8] = std::ls.empty;
    for $e in std::map.pairs(lend $best):
        if $e.key not_eq $hub and $departure plus $e.value.minutes at_most $deadline:
            std::ls.append(lend.mut $reached, value: copy $e.key);
        end of if
    end of for
    ret std::ls.len(lend $reached);
end of proc reached_by

proc compare_hubs [lent $net: Network]:
    let $paris_ch: %channel[int.unsign.64] := %channel.empty;
    let $koln_ch: %channel[int.unsign.64] := %channel.empty;
    let $wien_ch: %channel[int.unsign.64] := %channel.empty;
    let $paris := "Paris";
    let $koln := "Köln";
    let $wien := "Wien";
    %spawn.task with [lend $paris_ch, lend $net, lend $paris]:
        %channel.send(lend $paris_ch, value: reached_by(lend $net, lend $paris, $by_eight));
    end of %spawn.task
    %spawn.task with [lend $koln_ch, lend $net, lend $koln]:
        %channel.send(lend $koln_ch, value: reached_by(lend $net, lend $koln, $by_eight));
    end of %spawn.task
    %spawn.task with [lend $wien_ch, lend $net, lend $wien]:
        %channel.send(lend $wien_ch, value: reached_by(lend $net, lend $wien, $by_eight));
    end of %spawn.task
    let $p := %channel.take(lend $paris_ch) otherwise 0;
    let $k := %channel.take(lend $koln_ch) otherwise 0;
    let $w := %channel.take(lend $wien_ch) otherwise 0;
    std::print.shortest(newln, "stations reached by ", clock($by_eight), ": Paris ", $p, ", Köln ", $k, ", Wien ", $w, newln);
end of proc compare_hubs

# ---------------------------------------------------------------------------------------------------
# Looking things up: the stations whose names hold a given letter
# ---------------------------------------------------------------------------------------------------

fn stations_where [lent $net: Network, let $keep: fn[str.utf8] -> bool] -> std::ls[str.utf8]:
    var $out: std::ls[str.utf8] = std::ls.empty;
    for $s in lend $net.stations:
        if $keep(copy $s):
            std::ls.append(lend.mut $out, value: copy $s);
        end of if
    end of for
    ret give $out;
end of fn stations_where

proc show_letters [lent $net: Network]:
    std::print.shortest(newln, "stations by letter:", newln);
    let $letters := ["ö", "ü", "ó", "x"];
    for $part in lend $letters:
        let $found := stations_where(lend $net, fn [let $s] with [lend $part] = std::str.contains(lend $s, part: lend $part));
        if std::ls.len(lend $found) eq 0:
            std::print.shortest("  none with ", $part, newln);
        end of if
        else:
            std::print.shortest("  with ", $part, ": ", std::str.join(lend $found, between: ", "), newln);
        end of else
    end of for
end of proc show_letters

# ---------------------------------------------------------------------------------------------------
# The report
# ---------------------------------------------------------------------------------------------------

proc report [lent $bookings: std::ls[Booking][max: 500], lent $left: std::map.ordered[str.utf8, int.sign.32]]:
    var $out = std::print.writer(size: 4096 bytes);
    std::print.shortest(lend.mut $out, newln, "=== ", $network_name, ", tonight ===", newln);
    var $fares: std::ls[float.%deci.64][max: 500] = std::ls.empty;
    var $kms: std::ls[int.sign.32][max: 500] = std::ls.empty;
    var $longest = 0;
    var $longest_who = "nobody";
    for $b in lend $bookings:
        std::ls.append(lend.mut $fares, value: $b.fare);
        std::ls.append(lend.mut $kms, value: $b.journey.km);
        if $b.journey.km gt $longest:
            change $longest to $b.journey.km;
            change $longest_who to copy $b.traveller;
        end of if
    end of for
    let $count := std::ls.len(lend $bookings);
    let $takings := total_of(lend $fares);
    std::print.shortest(lend.mut $out, "tickets sold: ", $count, newln);
    std::print.format(lend.mut $out, "takings: {places: 2}{newln}", $takings);
    if $count gt 0:
        let $average := std::round.near($takings div.near to.float.%deci.64(to.int.sign.32($count)), places: 2);
        std::print.format(lend.mut $out, "average fare: {places: 2}{newln}", $average);
    end of if
    std::print.shortest(lend.mut $out, "km travelled: ", total_of(lend $kms), ", the longest ", $longest, " km (", $longest_who, ")", newln);
    std::print.shortest(lend.mut $out, "places left:", newln);
    for $e in std::map.pairs(lend $left):
        std::print.format(lend.mut $out, "  {align: left, width: 14 cols} {width: 3 cols}{newln}", $e.key, $e.value);
    end of for
    std::print.flush(lend.mut $out);
end of proc report

# Every request of the day, each with the next ticket number; the ones that cannot be read or booked are explained.
proc book_all [lent $net: Network, lent.mut $left: std::map.ordered[str.utf8, int.sign.32], lent $lines: std::ls[str.utf8]] -> std::ls[Booking][max: 500]:
    std::print.shortest(newln, "bookings:", newln);
    var $bookings: std::ls[Booking][max: 500] = std::ls.empty;
    var $ticket = 500;
    for $line in lend $lines:
        if std::ls.len(lend $bookings) at_least $most_bookings:
            std::print.shortest("  (tonight's trains are full)", newln);
            break;
        end of if
        let $req := parse_request(lend $line)-
        otherwise:
            !case bad_request $what:
                if $what eq $line:
                    std::print.shortest("  (cannot read \"", $line, "\")", newln);
                end of if
                else:
                    std::print.shortest("  (cannot read \"", $what, "\" in \"", $line, "\")", newln);
                end of else
                continue;
            !end of case bad_request
        end of otherwise;
        let $b := book(lend $net, lend.mut $left, lend $req, $ticket plus 1) otherwise booking_failed.!explain(who: copy $req.traveller);
        change $ticket to $ticket plus 1;
        show_booking(lend $b, copy $req.age);
        std::ls.append(lend.mut $bookings, value: give $b);
    end of for
    ret give $bookings;
end of proc book_all

# The night's quarter hours, from departure to midnight, walked by the clock's own iterator.
proc show_quarter_hours []:
    std::print.shortest(newln, "the last quarter hours before midnight:");
    var $quarters = Clock[now = $departure, stop = 1440];
    for $t in lend.mut $quarters:
        std::print.shortest(" ", clock($t));
    end of for
    std::print.shortest(newln);
end of proc show_quarter_hours

# Exact decimals: why a fare is a `float.%deci.64`. A fare of 0.10 three times is 0.30 exactly.
proc show_exact_money []:
    let $dime: float.%deci.64 := 0.10;
    let $three := $dime plus $dime plus $dime;
    let $float: float.bin.64 := 0.1 plus 0.1 plus 0.1;
    std::print.shortest("three fares of 0.10 as float.deci.64: ");
    std::print.stored($three, newln);
    std::print.shortest("the same in float.bin.64: ");
    std::print.stored($float, newln);
end of proc show_exact_money

# ---------------------------------------------------------------------------------------------------
# The night: read the timetable, then each part in turn
# ---------------------------------------------------------------------------------------------------

!MAIN:
    std::print.shortest("the ", $network_name, " night trains", newln, newln, "reading the timetable:", newln);
    let $rows := timetable();
    let $net := read_network(lend $rows);
    std::print.shortest("  ", std::ls.len(lend $net.links), " lines between ", std::ls.len(lend $net.stations), " stations", newln);
    board(lend $net, "Wien");
    var $left = places_left();
    let $requests := todays_requests();
    let $bookings := book_all(lend $net, lend.mut $left, lend $requests);
    report(lend $bookings, lend $left);
    compare_hubs(lend $net);
    show_letters(lend $net);
    show_quarter_hours();
    show_exact_money();
    std::print.shortest("good night from ", $network_name, newln);
!end of MAIN
