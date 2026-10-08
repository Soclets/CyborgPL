<p align="center">
  <img src="brand/logo.svg" alt="" width="96">
</p>

<h3 align="center">Nerve™</h3>

<p align="center">Just a small hobby lang 😊<br>
Very slow at the development, fast &amp; safe at the final product (no warranty 🤣).</p>

> [!WARNING]
> **Not family friendly**.
> "Not family friendly" means (the definition used in this place) **NOT FUCKING FAMILY FRIENDLY**
>
> Nerve is unstable by design: any release may change or remove anything. Nothing said about its stability is a guarantee, and it comes with **no warranty**. See our licences: [`LICENSE`](LICENSE), [`LICENSE-CC-BY-4.0`](LICENSE-CC-BY-4.0).

- [`DECISIONS.md`](DECISIONS.md) — every settled design decision, dated, with what was proposed against it.
- [`design/`](design/) — the design documents, one topic per file.
- [`examples/`](examples/) — programs in settled syntax, each with its expected output.
- [`oracle/`](oracle/) — the reference interpreter: the executable spec.
- [`cca/`](cca/) — the compiler.
- [`brand/`](brand/) — the mark (a circular C wrapping `:=`). On a pure white or pure black background use the single-colour versions, never the purple one.
- [`LICENSE`](LICENSE) — Apache-2.0 WITH LLVM-exception. Programs you compile owe nothing; a fork keeps the notices and may not use the name.
- [`LICENSE-CC-BY-4.0`](LICENSE-CC-BY-4.0) — CC BY 4.0, for the website's text (`website/`): copy it, translate it, remix it, with credit. Credit it exactly as:
  `Original: Nerve™ (product name) at (one of the product's locations) nerve-lang.com (official website)`
  The parentheses and what is inside them are part of the credit, not placeholders. As the licence asks, also say the text is under CC BY 4.0, and whether you changed it. The site's code stays under `LICENSE`.

## Contributing

Pull requests and issues are accepted only when the owner wants them.

## Code of Conduct

The Code of Conduct depends on the owner, and likely won't be statically stated.

## Security

No security support. No private reports, no fixes promised, no supported versions. Use at your own risk.

## Credits

Nerve™ is designed by **Tankun Sriket**, who makes every decision in [`DECISIONS.md`](DECISIONS.md). Everything below helped build it.

**Claude**, by [Anthropic](https://www.anthropic.com), wrote most of the code and documents here under the owner's direction, as four [Claude Code](https://claude.com/claude-code) sessions: *design claude* (the design notes, the decision log, the oracle and the benchmarks in `bench/`), *compiler claude* (CCA), *check claude* (the program generator, differential testing and translation validation in `check/`) and *website claude* (`website/`). Two more work in their own repos: *synthia claude* on [Synthia](https://github.com/Cyborg-Assembly-Manufacturing-Site/Synthia), and *RRx2-TV claude* on RRx2's translation validator. Each signs its commits with its house name; the models behind them include Claude Opus 5 and Claude Opus 5.5.

**Languages and tools**
- [Rust](https://www.rust-lang.org) and Cargo: the oracle, CCA and the verification harness.
- [LLVM](https://llvm.org) 22: CCA's code generator, through [llvm-sys](https://crates.io/crates/llvm-sys) and [Inkwell](https://github.com/TheDan64/inkwell).
- [Z3](https://github.com/Z3Prover/z3) (Microsoft Research) and [cvc5](https://cvc5.github.io): the SMT solvers behind Precog™'s proofs, and the independent check of them.
- [Alive2](https://github.com/AliveToolkit/alive2): check claude's first translation validation, proving LLVM's rewrites of CCA's code.
- [Clang](https://clang.llvm.org): the C programs CCA is benchmarked against (`cca/bench/`, `bench/`), and the C side of `bench/gpu/`.
- Metal and Xcode's Metal Toolchain, by Apple: the GPU side of `bench/gpu/`.
- The compilers and runtimes of `bench/`'s other languages (26 with C): C++, Rust, Go, Zig, Swift, Nim, Fortran (gfortran), OCaml, Dart, Java, Kotlin, C# (.NET), JavaScript and TypeScript (Node), Julia, LuaJIT, PyPy, Elixir, Erlang, Lust, Python, Ruby, Lua, Perl and R.
- [Python](https://www.python.org): the benchmark runners.
- [Node.js](https://nodejs.org) and npm: the website's build.
- [Homebrew](https://brew.sh): LLVM, Z3, cvc5, Alive2 and most of the benchmark's toolchains on the build machine.
- [Git](https://git-scm.com), [GitHub](https://github.com) and the [GitHub CLI](https://cli.github.com): hosting and history.
- [Cloudflare](https://www.cloudflare.com) Workers and Wrangler: nerve-lang.com (and the old cyborgpl.dev).
- macOS and Xcode's command-line tools, by Apple.

**Data**
- The [Unicode Character Database](https://www.unicode.org/ucd/) 17.0.0 and [UAX #29](https://www.unicode.org/reports/tr29/): grapheme clusters in `str` and `char`.

**Rust crates** (CCA, from `cca/Cargo.lock`)

`anyhow` 1.0.104, `bitflags` 2.13.2, `cc` 1.4.7, `find-msvc-tools` 0.1.13, `inkwell` 0.10.0, `inkwell_internals` 0.15.0, `lazy_static` 1.5.0, `libc` 0.2.189, `llvm-sys` 221.1.0, `proc-macro2` 1.0.107, `quote` 1.0.47, `regex-lite` 0.1.9, `semver` 1.0.28, `shlex` 2.0.1, `syn` 3.0.6, `thiserror` 2.0.20, `thiserror-impl` 2.0.20, `unicode-ident` 1.0.26, `unicode-segmentation` 1.13.3.

The oracle and the verification harness use no crates.

**Website packages** (`website/site/package.json`)

[React](https://react.dev), [React Router](https://reactrouter.com), [Motion](https://motion.dev), [isbot](https://github.com/omrilotan/isbot), [Vite](https://vite.dev), [Shiki](https://shiki.style) (with its TextMate grammar engine), [resvg-js](https://github.com/thx/resvg-js), and Cloudflare's [Wrangler](https://developers.cloudflare.com/workers/wrangler/).

<details>
<summary>Every package in <code>package-lock.json</code> (250)</summary>

`@babel/code-frame` 7.29.7, `@babel/compat-data` 7.29.7, `@babel/core` 7.29.7, `@babel/generator` 7.29.8, `@babel/helper-compilation-targets` 7.29.7, `@babel/helper-globals` 7.29.7, `@babel/helper-module-imports` 7.29.7, `@babel/helper-module-transforms` 7.29.7, `@babel/helper-string-parser` 7.29.7, `@babel/helper-validator-identifier` 7.29.7, `@babel/helper-validator-option` 7.29.7, `@babel/helpers` 7.29.7, `@babel/parser` 7.29.9, `@babel/template` 7.29.7, `@babel/traverse` 7.29.8, `@babel/types` 7.29.8, `@cloudflare/kv-asset-handler` 0.5.0, `@cloudflare/unenv-preset` 2.16.2, `@cloudflare/workerd-darwin-64` 1.20260923.1, `@cloudflare/workerd-darwin-arm64` 1.20260923.1, `@cloudflare/workerd-linux-64` 1.20260923.1, `@cloudflare/workerd-linux-arm64` 1.20260923.1, `@cloudflare/workerd-windows-64` 1.20260923.1, `@cspotcode/source-map-support` 0.8.1, `@emnapi/runtime` 1.11.3, `@esbuild/aix-ppc64` 0.28.1, `@esbuild/android-arm` 0.28.1, `@esbuild/android-arm64` 0.28.1, `@esbuild/android-x64` 0.28.1, `@esbuild/darwin-arm64` 0.28.1, `@esbuild/darwin-x64` 0.28.1, `@esbuild/freebsd-arm64` 0.28.1, `@esbuild/freebsd-x64` 0.28.1, `@esbuild/linux-arm` 0.28.1, `@esbuild/linux-arm64` 0.28.1, `@esbuild/linux-ia32` 0.28.1, `@esbuild/linux-loong64` 0.28.1, `@esbuild/linux-mips64el` 0.28.1, `@esbuild/linux-ppc64` 0.28.1, `@esbuild/linux-riscv64` 0.28.1, `@esbuild/linux-s390x` 0.28.1, `@esbuild/linux-x64` 0.28.1, `@esbuild/netbsd-arm64` 0.28.1, `@esbuild/netbsd-x64` 0.28.1, `@esbuild/openbsd-arm64` 0.28.1, `@esbuild/openbsd-x64` 0.28.1, `@esbuild/openharmony-arm64` 0.28.1, `@esbuild/sunos-x64` 0.28.1, `@esbuild/win32-arm64` 0.28.1, `@esbuild/win32-ia32` 0.28.1, `@esbuild/win32-x64` 0.28.1, `@img/colour` 1.1.0, `@img/sharp-darwin-arm64` 0.35.4, `@img/sharp-darwin-x64` 0.35.4, `@img/sharp-freebsd-wasm32` 0.35.4, `@img/sharp-libvips-darwin-arm64` 1.3.3, `@img/sharp-libvips-darwin-x64` 1.3.3, `@img/sharp-libvips-linux-arm` 1.3.3, `@img/sharp-libvips-linux-arm64` 1.3.3, `@img/sharp-libvips-linux-ppc64` 1.3.3, `@img/sharp-libvips-linux-riscv64` 1.3.3, `@img/sharp-libvips-linux-s390x` 1.3.3, `@img/sharp-libvips-linux-x64` 1.3.3, `@img/sharp-libvips-linuxmusl-arm64` 1.3.3, `@img/sharp-libvips-linuxmusl-x64` 1.3.3, `@img/sharp-linux-arm` 0.35.4, `@img/sharp-linux-arm64` 0.35.4, `@img/sharp-linux-ppc64` 0.35.4, `@img/sharp-linux-riscv64` 0.35.4, `@img/sharp-linux-s390x` 0.35.4, `@img/sharp-linux-x64` 0.35.4, `@img/sharp-linuxmusl-arm64` 0.35.4, `@img/sharp-linuxmusl-x64` 0.35.4, `@img/sharp-wasm32` 0.35.4, `@img/sharp-webcontainers-wasm32` 0.35.4, `@img/sharp-win32-arm64` 0.35.4, `@img/sharp-win32-ia32` 0.35.4, `@img/sharp-win32-x64` 0.35.4, `@jridgewell/gen-mapping` 0.3.13, `@jridgewell/remapping` 2.3.5, `@jridgewell/resolve-uri` 3.1.2, `@jridgewell/sourcemap-codec` 1.6.0, `@jridgewell/trace-mapping` 0.3.9, `@jridgewell/trace-mapping` 0.3.31, `@oxc-project/types` 0.151.0, `@poppinss/colors` 4.1.6, `@poppinss/dumper` 0.6.5, `@poppinss/exception` 1.2.3, `@react-router/dev` 8.4.0, `@react-router/node` 8.4.0, `@remix-run/node-fetch-server` 0.14.1, `@remix-run/route-pattern` 0.22.1, `@resvg/resvg-js` 2.6.2, `@resvg/resvg-js-android-arm-eabi` 2.6.2, `@resvg/resvg-js-android-arm64` 2.6.2, `@resvg/resvg-js-darwin-arm64` 2.6.2, `@resvg/resvg-js-darwin-x64` 2.6.2, `@resvg/resvg-js-linux-arm-gnueabihf` 2.6.2, `@resvg/resvg-js-linux-arm64-gnu` 2.6.2, `@resvg/resvg-js-linux-arm64-musl` 2.6.2, `@resvg/resvg-js-linux-x64-gnu` 2.6.2, `@resvg/resvg-js-linux-x64-musl` 2.6.2, `@resvg/resvg-js-win32-arm64-msvc` 2.6.2, `@resvg/resvg-js-win32-ia32-msvc` 2.6.2, `@resvg/resvg-js-win32-x64-msvc` 2.6.2, `@rolldown/binding-android-arm-eabi` 1.2.11, `@rolldown/binding-android-arm64` 1.2.11, `@rolldown/binding-darwin-arm64` 1.2.11, `@rolldown/binding-darwin-x64` 1.2.11, `@rolldown/binding-freebsd-x64` 1.2.11, `@rolldown/binding-linux-arm-gnueabihf` 1.2.11, `@rolldown/binding-linux-arm64-gnu` 1.2.11, `@rolldown/binding-linux-arm64-musl` 1.2.11, `@rolldown/binding-linux-ppc64-gnu` 1.2.11, `@rolldown/binding-linux-s390x-gnu` 1.2.11, `@rolldown/binding-linux-x64-gnu` 1.2.11, `@rolldown/binding-linux-x64-musl` 1.2.11, `@rolldown/binding-openharmony-arm64` 1.2.11, `@rolldown/binding-win32-arm64-msvc` 1.2.11, `@rolldown/binding-win32-x64-msvc` 1.2.11, `@rolldown/pluginutils` 1.0.1, `@shikijs/core` 4.4.3, `@shikijs/engine-javascript` 4.4.3, `@shikijs/engine-oniguruma` 4.4.3, `@shikijs/langs` 4.4.3, `@shikijs/primitive` 4.4.3, `@shikijs/themes` 4.4.3, `@shikijs/types` 4.4.3, `@shikijs/vscode-textmate` 10.0.2, `@sindresorhus/is` 7.2.0, `@speed-highlight/core` 1.2.24, `@types/hast` 3.0.5, `@types/mdast` 4.0.4, `@types/unist` 3.0.3, `@ungap/structured-clone` 1.4.0, `babel-dead-code-elimination` 1.0.12, `baseline-browser-mapping` 2.11.26, `blake3-wasm` 2.1.5, `browserslist` 4.29.1, `caniuse-lite` 1.0.30001812, `ccount` 2.0.1, `character-entities-html4` 2.1.0, `character-entities-legacy` 3.0.0, `chokidar` 5.0.0, `comma-separated-tokens` 2.0.3, `confbox` 0.3.1, `convert-source-map` 2.0.0, `cookie` 1.1.1, `cookie-es` 3.1.1, `debug` 4.4.3, `dedent` 1.7.2, `dequal` 2.0.3, `detect-libc` 2.1.2, `devlop` 1.1.0, `electron-to-chromium` 1.5.439, `error-stack-parser-es` 1.0.5, `es-module-lexer` 2.3.2, `esbuild` 0.28.1, `escalade` 3.2.0, `exit-hook` 5.1.0, `exsolve` 1.1.1, `fdir` 6.5.0, `framer-motion` 13.4.3, `fsevents` 2.3.3, `gensync` 1.0.0-beta.2, `hast-util-to-html` 9.0.5, `hast-util-whitespace` 3.0.0, `html-void-elements` 3.0.0, `isbot` 5.2.2, `js-tokens` 4.0.0, `jsesc` 3.1.0, `json5` 2.2.3, `kleur` 4.1.5, `lightningcss` 1.33.0, `lightningcss-android-arm64` 1.33.0, `lightningcss-darwin-arm64` 1.33.0, `lightningcss-darwin-x64` 1.33.0, `lightningcss-freebsd-x64` 1.33.0, `lightningcss-linux-arm-gnueabihf` 1.33.0, `lightningcss-linux-arm64-gnu` 1.33.0, `lightningcss-linux-arm64-musl` 1.33.0, `lightningcss-linux-x64-gnu` 1.33.0, `lightningcss-linux-x64-musl` 1.33.0, `lightningcss-win32-arm64-msvc` 1.33.0, `lightningcss-win32-x64-msvc` 1.33.0, `lodash` 4.18.1, `lru-cache` 5.1.1, `mdast-util-to-hast` 13.2.1, `micromark-util-character` 2.1.1, `micromark-util-encode` 2.0.1, `micromark-util-sanitize-uri` 2.0.1, `micromark-util-symbol` 2.0.1, `micromark-util-types` 2.0.2, `miniflare` 5.20260923.0-alpha, `motion` 13.4.3, `motion-dom` 13.4.2, `motion-utils` 13.3.0, `ms` 2.1.3, `nanoid` 3.3.19, `node-releases` 2.0.57, `oniguruma-parser` 0.12.2, `oniguruma-to-es` 4.3.6, `p-map` 7.0.8, `path-to-regexp` 6.3.0, `pathe` 2.0.3, `picocolors` 1.1.1, `picomatch` 4.0.7, `pkg-types` 2.3.3, `postcss` 8.5.28, `property-information` 7.2.0, `react` 19.3.0, `react-dom` 19.3.0, `react-refresh` 0.18.0, `react-router` 8.4.0, `readdirp` 5.1.1, `regex` 6.1.0, `regex-recursion` 6.0.2, `regex-utilities` 2.3.0, `rolldown` 1.2.11, `scheduler` 0.28.0, `semver` 6.3.1, `semver` 7.8.5, `sharp` 0.35.4, `shiki` 4.4.3, `source-map-js` 1.2.1, `space-separated-tokens` 2.0.2, `stringify-entities` 4.0.4, `supports-color` 10.2.2, `tinyglobby` 0.2.17, `trim-lines` 3.0.1, `tslib` 2.8.1, `undici` 7.29.0, `unenv` 2.0.0-rc.24, `unist-util-is` 6.0.1, `unist-util-position` 5.0.0, `unist-util-stringify-position` 4.0.0, `unist-util-visit` 5.1.0, `unist-util-visit-parents` 6.0.2, `update-browserslist-db` 1.3.3, `valibot` 1.5.0, `vfile` 6.0.3, `vfile-message` 4.0.3, `vite` 8.3.1, `workerd` 1.20260923.1, `wrangler` 4.139.0, `ws` 8.21.0, `yallist` 3.1.1, `youch` 4.1.0-beta.10, `youch-core` 0.3.3, `zwitch` 2.0.4.

</details>

**Ideas taken from other languages**
- **Rust**: ownership and borrowing, lifetime groups (`'a` there, `@x` here), interfaces and `<T: Ordered>` bounds (Rust's traits), diagnostics "helpful like Rust's", and `::` for stepping into another Limb (Rust's paths, which took it from C++).
- **Ada**: `and then` / `or else`, here `and.then` / `or.else`.
- **Swift**, **Zig** and **C**: points of comparison in the SIMD, wrapping-arithmetic and enum decisions.
- **IEEE 754**: `bin16`, `bin32` and `bin64` arithmetic and rounding.
- **Lua**: the name *Lunae*.
- **Kespar**, another of the owner's languages: Precog's design notes and its first provisional limits.
- **Xag**, the owner's C++ language: the reference for everything outside Precog, and the licence.
- **CyborgPL v1** ([CyborgPL-legacy](https://github.com/Artificial-IntelligenceAI/CyborgPL-legacy)): `x` and `xx`.

Copyright 2026 Tankun Sriket.
Nerve™ (formerly CyborgPL™), Precog™ and Limb™ are names of this project.
