# Turing Machine Executor

A Rust implementation of a Turing machine executor that allows users to define, execute, and test Turing machines.

## Features

- Execute Turing machines with custom input strings
- Determine if a machine **accepts** or **rejects** the input (holds)
- Report the **final state** reached by the machine
- Support for both interactive mode and file-based machine definitions
- Built-in example machines for testing
- JSON-based machine definition format

## What is a Turing Machine?

A Turing machine is a mathematical model of computation that consists of:
- **Tape**: An infinite sequence of cells, each containing a symbol
- **Head**: A read/write head that can move left or right on the tape
- **States**: A finite set of states the machine can be in
- **Transition Function**: Rules that determine the next state, what to write, and where to move based on the current state and symbol
- **Initial State**: The starting state
- **Accept States**: States that indicate the input is accepted
- **Reject States**: States that indicate the input is rejected

## Installation

1. Install Rust from [https://rustup.rs/](https://rustup.rs/) if you haven't already
2. Clone the repository:

```bash
git clone https://github.com/vibecoding-inc/turning-machine.git
cd turning-machine
```

3. Build the project:

```bash
cargo build --release
```

The compiled binary will be available at `./target/release/turning_machine`.

## Usage

#### Interactive Mode

Run the main program to use the interactive interface:

```bash
cargo run --release
# or if already built:
./target/release/turning_machine
```

The program offers several options:
1. **Run example machine** - Test with pre-built Turing machines
2. **Define custom machine** - Create your own machine using JSON
3. **Load machine from file** - Load a machine definition from a JSON file
4. **Help** - View format documentation
5. **Exit** - Close the program

#### Running Examples

You can run the example machines directly:

```bash
cargo run --release -- --examples
# or if already built:
./target/release/turning_machine --examples
```

#### Loading from File

Example machine definitions are provided in the `examples/` directory:

```bash
# Run the interactive program and select option 3
cargo run --release
# Then enter: examples/even_ones.json
```

## Machine Definition Format

Turing machines are defined using JSON with the following structure:

```json
{
    "states": ["q0", "q1", "accept", "reject"],
    "alphabet": ["0", "1"],
    "tape_alphabet": ["0", "1", "_"],
    "initial_state": "q0",
    "accept_states": ["accept"],
    "reject_states": ["reject"],
    "blank_symbol": "_",
    "transitions": {
        "q0,0": ["q0", "0", "R"],
        "q0,1": ["q1", "1", "R"],
        "q1,_": ["accept", "_", "R"]
    }
}
```

### Field Descriptions

- **states**: Array of all state names
- **alphabet**: Array of input symbols (symbols that can appear in input)
- **tape_alphabet**: Array of all symbols that can appear on tape (includes alphabet + blank + work symbols)
- **initial_state**: Name of the starting state
- **accept_states**: Array of accepting state names
- **reject_states**: Array of rejecting state names
- **blank_symbol**: Symbol representing empty tape cells (default: "_")
- **transitions**: Object mapping state-symbol pairs to [new_state, write_symbol, direction]
  - Key format: `"state,symbol"`
  - Value format: `["new_state", "write_symbol", "L or R"]`
  - Direction: `"L"` for left, `"R"` for right

## Example Machines

### 1. Even Number of 1s (`examples/even_ones.json`)

Accepts strings with an even number of 1s (including zero).

**Examples:**
- ✓ `""` → ACCEPTS
- ✓ `"0"` → ACCEPTS
- ✗ `"1"` → REJECTS
- ✓ `"11"` → ACCEPTS
- ✗ `"111"` → REJECTS
- ✓ `"0101"` → ACCEPTS

### 2. Accept All (`examples/accept_all.json`)

Accepts any string over the alphabet {0, 1}.

**Examples:**
- ✓ `""` → ACCEPTS
- ✓ `"0"` → ACCEPTS
- ✓ `"111"` → ACCEPTS
- ✓ `"01010"` → ACCEPTS

### 3. a⁺b⁺ (`examples/a_plus_b_plus.json`)

Accepts strings of one or more 'a's followed by one or more 'b's.

**Examples:**
- ✓ `"ab"` → ACCEPTS
- ✓ `"aabb"` → ACCEPTS
- ✓ `"aaabbb"` → ACCEPTS
- ✗ `"a"` → REJECTS
- ✗ `"ba"` → REJECTS
- ✗ `"aba"` → REJECTS

## Output Interpretation

When you execute a Turing machine, the program provides:

1. **Accepts/Rejects**: Whether the machine accepts or rejects the input
   - **ACCEPTS**: The machine reached an accept state (holds)
   - **REJECTS**: The machine reached a reject state or no transition is defined
   - **DID NOT HALT**: The machine exceeded the maximum step limit (possible infinite loop)

2. **Final State**: The state the machine was in when it halted (or when it exceeded the step limit)

3. **Steps**: Number of transitions executed

4. **Halted**: Whether the machine successfully halted or hit the step limit

## API Usage

You can use the Turing machine executor as a library in your own Rust code by creating a library crate structure (not currently set up in this repository).

## Requirements

- Rust 1.70+ (install from [https://rustup.rs/](https://rustup.rs/))
- No external runtime dependencies

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests.

## License

This project is open source and available for educational purposes.
