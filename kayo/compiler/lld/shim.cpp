// lld through its library (owner, 2026-10-08: "`a`", Zig's way): lld has
// no C interface, so this is the small C++ wrapper implementation-
// languages.md allows. `argv[0]` names the flavour (`ld64.lld`, `ld.lld`,
// `lld-link`, `wasm-ld`), as lld's own executable reads it. lld's library
// is not made to run twice in a process; CCA links once per run.

#include "lld/Common/Driver.h"
#include "llvm/Support/raw_ostream.h"

#include <cstring>
#include <string>

LLD_HAS_DRIVER(macho)
LLD_HAS_DRIVER(elf)
LLD_HAS_DRIVER(coff)
LLD_HAS_DRIVER(wasm)

// The link's exit code; what lld said, cut to `said_len`, into `said`.
extern "C" int cca_lld_link(int argc, const char **argv, char *said, size_t said_len) {
    std::string out, err;
    llvm::raw_string_ostream os(out), es(err);
    lld::Result r = lld::lldMain(llvm::ArrayRef<const char *>(argv, argc), os, es,
                                 {{lld::Darwin, &lld::macho::link},
                                  {lld::Gnu, &lld::elf::link},
                                  {lld::WinLink, &lld::coff::link},
                                  {lld::Wasm, &lld::wasm::link}});
    os.flush();
    es.flush();
    std::string all = out + err;
    if (said && said_len > 0) {
        size_t n = all.size() < said_len - 1 ? all.size() : said_len - 1;
        std::memcpy(said, all.data(), n);
        said[n] = '\0';
    }
    return r.retCode;
}
