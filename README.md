# CerisEmu: A capability machine emulator

Language-Based Security 2024

Group 10

Philipp Haas & Léo Gamboa dos Santos

## Running the project
- Make sure you have rust installed. We recommend installing it via [rustup](https://www.rust-lang.org/tools/install), and make sure you also install `cargo`.

- In the root of the project, run `cargo build --release`.
- Then to run the project, run `./target/release/cerisemu[.exe] [args]`

- Alternatively, you can run the project directly by running `cargo run --release -- [args]`

## CLI
- `cerisemu --help` should give you an idea of the possible commands.
- `cerisemu compile` will compile an assembly file into the internal representation. Make sure to check the help with `cerisemu compile --help`.
- `cerisemu emulate` will emulate a capability machine. Again, make sure to check the help with `cerisemu emulate --help`. If the `--compile` flag is not used, the input file must be valid ron file containing a Program, a ProgramConfig, or a MachineConfig.

### Running the OS example
Run `cargo run --release -- emulate -i config/os.ron --backtrace`.

## Directories
### Source directories
- `src/` contains all the sources of the project.
	- `src/main.rs` contains the CLI definitions.
	- `src/lib.rs` is the main library file and is used by the CLI to run either emulation or compilation.
	- `src/compiler/` contains all sources related to compilation.
	- `src/emulator/` contains all sources related to emulation.
- `tests/` contains all our unit/integration tests.

### Other directories
- `asm/` contains our uncompiled assembly files.
- `bin/` contains compiled programs.
- `config/` contains MachineConfigs that can be emulated directly.