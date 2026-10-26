# common.sh: what every check shares; each script sources it. Paths come from where this folder sits, so the checks
# run in any clone. What they write, and the Unicode files they read, are in the clone's git-ignored .std-checks/
# (owner, 2026-10-25: scratch only for what's fine to lose; 2026-10-26: std's tests are committed, here).
CHECKS="$(cd "$(dirname "$0")" && pwd)"
R="$(cd "$CHECKS/../../.." && pwd)"
OUT="$R/.std-checks"
UCD="$OUT/ucd-17.0.0"
Y="$R/kayo/target/release/kayo"
O="$R/oracle/target/release/oracle"
STD_ID=fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936
STD_VER=2026-10-06T12:57:14+07:00
UNICODE_ID=55a2e2de2eca2e06e8a4b3aa2c5ef7245f3f55d4dfbfd872a3ddd9500f74dece
UNICODE_VER=2026-10-12T10:19:44+07:00
MONEY_ID=d7b62f4448a5b23d792a25bcdba89220e02691fb5e27cf18640ee491d049df0f
MONEY_VER=2026-10-07T20:22:24+07:00

# toml DIR NAME WORD…: DIR/CyJet.toml for a kit named NAME. Each WORD adds one thing:
#   std          this std/, by its path       builtin   the oracle's built-in std, by its id
#   stdtest=DIR  a copy of std/ as stdtest     unicode, money   that limb
#   vouch, raw_memory   that power
toml() {
    _dir="$1"; _name="$2"; shift 2
    _kits=""; _limbs=""; _powers=""
    for _w in "$@"; do
        case "$_w" in
            std) _kits="${_kits}std {\npath = $R/std\n}\n" ;;
            builtin) _kits="${_kits}std {\nid = $STD_ID\nver = $STD_VER\n}\n" ;;
            stdtest=*) _kits="${_kits}stdtest {\npath = ${_w#stdtest=}\n}\n" ;;
            unicode) _limbs="${_limbs}unicode {\nid = $UNICODE_ID\nver = $UNICODE_VER\n}\n" ;;
            money) _limbs="${_limbs}money {\nid = $MONEY_ID\nver = $MONEY_VER\n}\n" ;;
            vouch|raw_memory) _powers="${_powers}$_w {\n}\n" ;;
            *) echo "toml: no word $_w" >&2; return 1 ;;
        esac
    done
    {
        printf 'my details:\nname = %s\nprefix = %s\nver = 0.0.1\n\n' "$_name" "$_name"
        [ -n "$_kits" ] && printf "my kits:\n${_kits}\n"
        [ -n "$_limbs" ] && printf "my limbs:\n${_limbs}\n"
        [ -n "$_powers" ] && printf "my powers:\n${_powers}\n"
        printf 'my targets:\naarch64-macos {\n}\n'
    } > "$_dir/CyJet.toml"
}

# stdtest W: a copy of std/ in W/kits/std, named stdtest, beside the oracle's built-in std, for the pair checks.
stdtest() {
    mkdir -p "$1/kits/std"; cp "$R/std/"*.cy "$1/kits/std/"
    toml "$1/kits/std" stdtest builtin unicode money
}

# program DIR NAME FILE WORD…: a test kit whose root is FILE (a path, or - for what comes in).
program() {
    _pdir="$1"; _pname="$2"; _pfile="$3"; shift 3
    mkdir -p "$_pdir"
    if [ "$_pfile" = - ]; then cat > "$_pdir/kit.cy"; else cp "$_pfile" "$_pdir/kit.cy"; fi
    toml "$_pdir" "$_pname" "$@"
}

# first_line FILE: kayo's one-line build summary, after the program's path.
first_line() { head -1 "$1" | sed -E 's/^built .*\.(prog|final) //' | cut -c1-110; }
