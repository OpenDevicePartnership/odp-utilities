//! cargo-stack-size
//!
//! A tool for inspecting stack size of a binary by function and crate
use cargo_metadata::{Message, camino::Utf8PathBuf};
use clap::{ArgAction, Parser};
use rustc_demangle::demangle;
use stack_sizes::analyze_executable;
use std::process::{Child, Command, Stdio};

#[derive(Parser, Debug)]
#[command(
    name = "cargo stack-size",
    version,
    about = "Shows stack size of binary by function and crate.\n\n\
    \
    `-Zemit-stack-sizes` MUST be set for stack size information to be emitted by the linker.\n\
    Additionally, depending on target, additional steps may need to be taken to ensure \
    that the stack size section is kept.\n\n\
    \
    For ease of use, this tool can be instructed to emit stack sizes with the `--emit-stack-sizes` flag.\n\
    Be warned however that this will override any RUSTFLAGS in .cargo/config.toml\n\n\
    \
    This ALWAYS builds with nightly as it is required for `-Zemit-stack-sizes`.\n\
    Which means stack sizes may differ slightly from a stable build."
)]
struct Args {
    #[arg(
        short,
        long,
        help = "Number of lines to show, 0 to show all",
        default_value_t = 20
    )]
    num: usize,

    #[arg(long, help = "Space separated list of features to activate")]
    features: Option<String>,

    #[arg(
        long = "crate",
        help = "Show only functions belonging to specified crate"
    )]
    crate_name: Option<String>,

    #[arg(long = "function", help = "Show only functions with specified name")]
    function_name: Option<String>,

    #[arg(
        long,
        allow_hyphen_values = true,
        help = "Set RUSTFLAGS (be warned this will override any previous flags)"
    )]
    rustflags: Option<String>,

    #[arg(long, action = ArgAction::SetTrue, help = "Shorthand for `--rustflags \"-Zemit-stack-sizes\"`")]
    emit_stack_sizes: bool,

    #[arg(long, action = ArgAction::SetTrue, help = "Filter anonymous closures")]
    filter_closures: bool,

    #[arg(long, action = ArgAction::SetTrue, help = "Build in release mode")]
    release: bool,
}

struct Function {
    name: String,
    stack_sz: u64,
}

fn spawn_cargo(
    release_mode: bool,
    features: Option<String>,
    rustflags: Option<String>,
    emit_stack_sizes: bool,
) -> std::io::Result<Child> {
    let mut cargo = Command::new("cargo");

    // Always build nightly since it's required for -Zemit-stack-sizes
    cargo.args(["+nightly", "build"]);

    if release_mode {
        cargo.arg("--release");
    }

    if let Some(features) = features {
        cargo.args(["--features", &features]);
    }

    // Only way to include -Zemit-stack-sizes here is to overwrite env::RUSTFLAGS
    // But don't want to overwrite flags set by user (such as in .cargo/config.toml)
    // Unfortunately don't see a way to append it
    // If we retrieve the current env and append, that still misses rustflags from config.toml
    // So let user decide if they want to override
    if emit_stack_sizes && let Some(rustflags) = rustflags {
        cargo.env("RUSTFLAGS", rustflags + " -Zemit-stack-sizes");
    } else if emit_stack_sizes {
        cargo.env("RUSTFLAGS", "-Zemit-stack-sizes");
    } else if let Some(rustflags) = rustflags {
        cargo.env("RUSTFLAGS", rustflags);
    }

    // First, build normally so user can see output if failed
    // If it did fail, just exit early here
    let status = cargo.status()?;
    if !status.success() {
        std::process::exit(1);
    }

    // Then build with output to json for message parsing
    // This won't trigger a full rebuild since nothing has changed
    cargo.arg("--message-format=json");
    cargo.stdout(Stdio::piped()).spawn()
}

fn bin_path_from_cargo(mut cargo: Child) -> Option<Utf8PathBuf> {
    let reader = std::io::BufReader::new(cargo.stdout.take()?);

    // Just get the binary executable produced by cargo build
    for message in Message::parse_stream(reader) {
        if let Ok(Message::CompilerArtifact(artifact)) = message
            && let Some(bin) = artifact.executable
        {
            cargo.wait().ok()?;
            return Some(bin);
        }
    }

    None
}

fn sorted_functions(
    bin: &[u8],
    crate_name: Option<String>,
    function_name: Option<String>,
    filter_closures: bool,
) -> Option<Vec<Function>> {
    let functions = analyze_executable(bin).ok()?;

    // No stack size info found in ELF
    if functions.defined.values().all(|f| f.stack().is_none()) {
        return None;
    }

    let mut functions: Vec<Function> = functions
        .defined
        .iter()
        .filter_map(|(_addr, f)| {
            // {:#} removes the hash from the demangled name
            let name = format!("{:#}", demangle(f.names().first()?));
            let stack_sz = f.stack()?;

            if filter_closures && name.ends_with("{{closure}}") {
                return None;
            }

            // This doesn't *truly* check if a function belongs to a crate like `cargo bloat``
            // Only checks if demgangled name begins with crate name which is good enough but not perfect
            // May revisit with some tips from `cargo bloat`` if necessary
            let keep = match (&crate_name, &function_name) {
                (Some(crate_name), Some(function_name)) => {
                    name.starts_with(crate_name) && name.contains(function_name)
                }
                (Some(crate_name), None) => name.starts_with(crate_name),
                (None, Some(function_name)) => name.contains(function_name),
                (None, None) => true,
            };

            if keep {
                Some(Function { name, stack_sz })
            } else {
                None
            }
        })
        .collect();

    functions.sort_by_key(|f| std::cmp::Reverse(f.stack_sz));
    Some(functions)
}

fn main() {
    // Needed otherwise Clap will think "stack-size" is an argument when called as `cargo stack-size`
    let mut args = std::env::args().collect::<Vec<_>>();
    if args.len() > 1 && args[1] == "stack-size" {
        args.remove(1);
    }
    let args = Args::parse_from(args);

    let cargo = spawn_cargo(
        args.release,
        args.features,
        args.rustflags,
        args.emit_stack_sizes,
    )
    .expect("Failed to build binary");
    let bin = bin_path_from_cargo(cargo).expect("Failed to locate binary");
    println!("Inspecting stack size of {}\n", bin.as_str());

    let bin = std::fs::read(bin).expect("Failed to read binary");
    let functions = sorted_functions(
        &bin,
        args.crate_name,
        args.function_name,
        args.filter_closures,
    )
    .expect(
        "No stack usage info found, \
        ensure -Zemit-stack-sizes is set in RUSTFLAGS and stack size section is kept by linker",
    );

    println!("Size\tName");
    let n = if args.num == 0 { usize::MAX } else { args.num };
    for f in functions.iter().take(n) {
        println!("{}\t{:#}", f.stack_sz, &f.name);
    }
}
