# Browser Session GNU linker-script provenance

Status: Draft source-level traceability for PR #317. This document does not claim hosted exact-head repository/security GREEN.

## Problem

The Browser Session Cargo execution-authority contract already fails closed on direct linker selection, GCC driver program-search replacement, opaque response files, dynamically loaded linker plugins, GCC specs files, and GCC driver wrappers. A separate GNU-compatible input-provenance surface remained: linker scripts selected through `-T` / `--script`.

Rust documents `-C link-arg` and `-C link-args` as arguments appended to the linker invocation. GCC documents `-T script` as selecting a linker script on systems using the GNU linker. GNU ld additionally defines `INPUT(file, ...)` and `GROUP(file, ...)` commands that introduce named files into the link as if they had appeared on the command line. A Git-owned Cargo flag can therefore select a repository or external linker script that adds object/archive inputs outside the reviewed Rust production-source closure without changing the nominal Cargo target or linker executable.

This is source/input provenance rather than a claim that the script itself is an executable program. It belongs in the same fail-closed compiler/linker authority boundary because the resulting binary can contain code selected by that script.

## RED

Commit `43375ef80b1bcf6a0421f900a2f9cbf519741849` adds a hostile repository fixture whose reviewed Cargo configuration selects `tools/review-bypass.ld`; the script contains `INPUT(tools/review-bypass-object.o)`. The fixture covers three encodings consumed by the existing shared parser:

- driver-level `-Ttools/review-bypass.ld`;
- forwarded GNU ld `-Wl,--script=tools/review-bypass.ld`;
- split `-Xlinker -T -Xlinker tools/review-bypass.ld` inside `link-args`.

The predecessor classifier did not recognize linker-script selection, so these cases were source-semantic RED. The fixture imports the canonical Browser Session Cargo compiler-authority contract rather than reproducing Cargo workspace/package discovery.

## Decision and repair

Commit `8975224e86ea5ef9821d168efeaafa20702ec659` extends only the existing linker-argument classifier. The guard now fails closed on:

- direct `-T`, joined `-T<path>`, `--script`, and `--script=<path>`;
- the same script-selection options forwarded through `-Wl,` or `--for-linker=`;
- script-selection option tokens passed through `-Xlinker`.

Existing executable-selection, response-file, specs, wrapper, plugin, and ordinary non-authorizing linker-argument behavior remains in the same parser. A normal `-pthread` control remains allowed.

The contract deliberately rejects script selection before trying to parse or allowlist script contents. Relaxation requires a same-tree immutable linker-script/input provenance contract that recursively proves every script-selected object/archive/search surface and remains valid for the exact qualified linker implementation. A path-only allowlist is insufficient because GNU ld scripts can introduce additional inputs and search behavior.

## Boundary and residual risk

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the shared Git-owned Cargo compiler/linker authority classifier.
- This repair does not widen the future BiDi adapter allowlist or authorize any linker script, object, archive, or external path.
- Build-script-generated linker arguments remain prohibited by the separate production build-surface contract until generated-source/build provenance is explicitly modeled.
- Non-GNU linker script/control-file mechanisms, implicit linker scripts supplied as ordinary input files, command-line `-L`/library selection, target/toolchain-supplied scripts, and external native dependencies remain separate review surfaces.

## Primary references

Free Software Foundation. (2026). *Using the GNU Compiler Collection (GCC): Link options*. https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html

Free Software Foundation. (2026). *The GNU linker*. https://sourceware.org/binutils/docs/ld.pdf

Rust Project Developers. (2026). *The rustc book: Codegen options*. https://doc.rust-lang.org/rustc/codegen-options/index.html
