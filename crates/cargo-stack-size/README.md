# cargo-stack-size
A tool for inspecting binary stack size by function and crate.

## Install
`cargo install --git https://github.com/OpenDevicePartnership/odp-utilities cargo-stack-size`

## Heads Up
This tool makes use of the unstable flag [`-Zemit-stack-sizes`](https://doc.rust-lang.org/beta/unstable-book/compiler-flags/emit-stack-sizes.html) which requires the `nightly` toolchain.

This flag may or may not work correctly depending on target, and is not guaranteed to work on all versions of `nightly`.
However it appears to work correctly on Linux and ARM Cortex-M ELF binaries.

Therefore, to use this tool, you must ensure the `-Zemit-stack-sizes` flag is set,
either in the `RUSTFLAGS` env variable or within `.cargo/config.toml`. You can also instruct this
tool to include the flag for you (see [Usage](#usage)) but be warned that would override any other flags set.

Lastly, as `nightly` is required, reported stack sizes may differ slightly from a `stable` build.

## Usage
Show full list of arguments and usage:  
`cargo stack-size --help`

Show top 10 functions by stack size in release build:  
`cargo stack-size -n 10 --release`

Show all functions by stack size and include `-Zemit-stack-sizes`:  
`cargo stack-size -n 0 --emit-stack-sizes`

Show top functions by stack size for given crate:  
`cargo stack-size --crate cargo_stack_size`

Show top functions by stack size matching given name:  
`cargo stack-size --function cargo_stack_size::main`  
**Note**: Functions may get inlined and thus won't show up here.
Try using `[#inline(never)]` for function you wish to inspect.

Show top 20 functions by stack size and filter anonymous closures:  
`cargo stack-size --filter-closures`

## Example Output
```
$ cargo stack-size --filter-closures --crate cargo_stack_size --emit-stack-sizes
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
Inspecting stack size of target/debug/cargo-stack-size

Size    Name
1352    cargo_stack_size::bin_path_from_cargo
1144    cargo_stack_size::main
824     cargo_stack_size::sorted_functions
536     cargo_stack_size::spawn_cargo
```

## Motivation
Being able to inspect stack size quickly and easily of a function or crate is a handy way to gain insight
into total stack footprint.

Currently, a fantastic tool exists called [`cargo-call-stack`](https://github.com/japaric/cargo-call-stack)
by [@japaric](https://github.com/japaric) which is great if you need to analyze the upper bound of stack
depth and visualize the call graph.

Unfortunately however, this tool does not currently work without some modifications.
A [fork](https://github.com/Dirbaio/cargo-call-stack) by [@Dirbaio](https://github.com/Dirbaio)
exists which does seem to work, and can be used to also provide a printed list of top functions by
stack size, but it's not as simple to use and lacks some features for organizing by crate and function
name.

This tool makes use of the awesome [stack-sizes](https://crates.io/crates/stack-sizes) library also written
by [@japaric](https://github.com/japaric). At one point this library also provided a binary version
with similar functionality as this tool but has been abandoned in favor of `cargo-call-stack`.

However, the value this tool provides over `cargo-call-stack` is ease of use and ability to inspect
individual functions and crates when you don't need full call stack analysis.

## Acknowledgements
- Inspired by [@japaric](https://github.com/japaric)'s work on [`cargo-call-stack`](https://github.com/japaric/cargo-call-stack) and makes use of his [stack-sizes](https://crates.io/crates/stack-sizes) crate.