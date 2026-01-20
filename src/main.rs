use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};

/// Represents the direction the Turing machine head can move
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Direction {
    L, // Left
    R, // Right
}

/// Result of executing a Turing machine
#[derive(Debug)]
struct ExecutionResult {
    accepts: Option<bool>, // True if accepts, False if rejects, None if didn't halt
    final_state: String,
    steps: usize,
    halted: bool,
    tape: String,
}

/// A Turing machine executor
#[derive(Debug)]
struct TuringMachine {
    states: HashSet<String>,
    alphabet: HashSet<char>,
    tape_alphabet: HashSet<char>,
    transitions: HashMap<(String, char), (String, char, Direction)>,
    initial_state: String,
    accept_states: HashSet<String>,
    reject_states: HashSet<String>,
    blank_symbol: char,
}

impl TuringMachine {
    /// Create a new Turing machine
    fn new(
        states: HashSet<String>,
        alphabet: HashSet<char>,
        tape_alphabet: HashSet<char>,
        transitions: HashMap<(String, char), (String, char, Direction)>,
        initial_state: String,
        accept_states: HashSet<String>,
        reject_states: HashSet<String>,
        blank_symbol: char,
    ) -> Result<Self, String> {
        // Validate input
        if !states.contains(&initial_state) {
            return Err(format!("Initial state {} not in states", initial_state));
        }
        if !accept_states.is_subset(&states) {
            return Err("Accept states must be subset of states".to_string());
        }
        if !reject_states.is_subset(&states) {
            return Err("Reject states must be subset of states".to_string());
        }
        if !accept_states.is_disjoint(&reject_states) {
            return Err("Accept and reject states must be disjoint".to_string());
        }
        if !tape_alphabet.contains(&blank_symbol) {
            return Err(format!("Blank symbol {} not in tape alphabet", blank_symbol));
        }

        Ok(TuringMachine {
            states,
            alphabet,
            tape_alphabet,
            transitions,
            initial_state,
            accept_states,
            reject_states,
            blank_symbol,
        })
    }

    /// Execute the Turing machine on the given input
    fn execute(&self, input_string: &str, max_steps: usize) -> Result<ExecutionResult, String> {
        // Initialize tape with input
        let mut tape: Vec<char> = if input_string.is_empty() {
            vec![]
        } else {
            input_string.chars().collect()
        };
        let mut head_position: i32 = 0;
        let mut current_state = self.initial_state.clone();
        let mut steps = 0;

        // Validate input symbols
        for symbol in input_string.chars() {
            if !self.alphabet.contains(&symbol) {
                return Err(format!("Invalid input symbol: {}", symbol));
            }
        }

        // Execute until halt or max steps
        while steps < max_steps {
            // Check if in halting state
            if self.accept_states.contains(&current_state) {
                return Ok(ExecutionResult {
                    accepts: Some(true),
                    final_state: current_state,
                    steps,
                    halted: true,
                    tape: tape.iter().collect(),
                });
            }

            if self.reject_states.contains(&current_state) {
                return Ok(ExecutionResult {
                    accepts: Some(false),
                    final_state: current_state,
                    steps,
                    halted: true,
                    tape: tape.iter().collect(),
                });
            }

            // Extend tape if needed
            if head_position < 0 {
                tape.insert(0, self.blank_symbol);
                head_position = 0;
            }
            if head_position >= tape.len() as i32 {
                tape.push(self.blank_symbol);
            }

            // Read current symbol
            let current_symbol = tape[head_position as usize];

            // Look up transition
            let transition_key = (current_state.clone(), current_symbol);
            if let Some((new_state, write_symbol, direction)) = self.transitions.get(&transition_key)
            {
                // Write symbol
                tape[head_position as usize] = *write_symbol;

                // Move head
                match direction {
                    Direction::L => head_position -= 1,
                    Direction::R => head_position += 1,
                }

                // Update state
                current_state = new_state.clone();
                steps += 1;
            } else {
                // No transition defined - implicit reject
                return Ok(ExecutionResult {
                    accepts: Some(false),
                    final_state: current_state,
                    steps,
                    halted: true,
                    tape: tape.iter().collect(),
                });
            }
        }

        // Max steps reached - likely infinite loop
        Ok(ExecutionResult {
            accepts: None,
            final_state: current_state,
            steps,
            halted: false,
            tape: tape.iter().collect(),
        })
    }
}

/// Helper struct for JSON deserialization
#[derive(Debug, Deserialize)]
struct MachineJson {
    states: Vec<String>,
    alphabet: Vec<String>,
    tape_alphabet: Vec<String>,
    initial_state: String,
    accept_states: Vec<String>,
    reject_states: Vec<String>,
    blank_symbol: Option<String>,
    transitions: HashMap<String, Vec<String>>,
}

/// Parse a Turing machine from JSON format
fn parse_machine_json(json_data: &MachineJson) -> Result<TuringMachine, String> {
    // Convert transitions from string keys to tuple keys
    let mut transitions = HashMap::new();
    for (key, value) in &json_data.transitions {
        let parts: Vec<&str> = key.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid transition key: {}", key));
        }
        let state = parts[0].to_string();
        let symbol = parts[1]
            .chars()
            .next()
            .ok_or_else(|| format!("Invalid symbol in transition key: {}", key))?;

        if value.len() != 3 {
            return Err(format!("Invalid transition value for key: {}", key));
        }
        let new_state = value[0].clone();
        let write_symbol = value[1]
            .chars()
            .next()
            .ok_or_else(|| format!("Invalid write symbol in transition: {}", key))?;
        let direction = match value[2].as_str() {
            "L" => Direction::L,
            "R" => Direction::R,
            _ => return Err(format!("Invalid direction: {}", value[2])),
        };

        transitions.insert((state, symbol), (new_state, write_symbol, direction));
    }

    let blank_symbol = json_data
        .blank_symbol
        .as_ref()
        .and_then(|s| s.chars().next())
        .unwrap_or('_');

    // Validate alphabet entries are single characters
    for entry in &json_data.alphabet {
        if entry.chars().count() != 1 {
            return Err(format!(
                "Alphabet entry '{}' must be a single character",
                entry
            ));
        }
    }

    // Validate tape_alphabet entries are single characters
    for entry in &json_data.tape_alphabet {
        if entry.chars().count() != 1 {
            return Err(format!(
                "Tape alphabet entry '{}' must be a single character",
                entry
            ));
        }
    }

    TuringMachine::new(
        json_data.states.iter().cloned().collect(),
        json_data.alphabet.iter().flat_map(|s| s.chars()).collect(),
        json_data
            .tape_alphabet
            .iter()
            .flat_map(|s| s.chars())
            .collect(),
        transitions,
        json_data.initial_state.clone(),
        json_data.accept_states.iter().cloned().collect(),
        json_data.reject_states.iter().cloned().collect(),
        blank_symbol,
    )
}

/// Create example Turing machines for testing
fn create_example_machines() -> HashMap<String, TuringMachine> {
    let mut examples = HashMap::new();

    // Machine 1: Accepts strings with even number of 1s
    let mut transitions = HashMap::new();
    transitions.insert(
        ("q0".to_string(), '0'),
        ("q0".to_string(), '0', Direction::R),
    );
    transitions.insert(
        ("q0".to_string(), '1'),
        ("q1".to_string(), '1', Direction::R),
    );
    transitions.insert(
        ("q0".to_string(), '_'),
        ("accept".to_string(), '_', Direction::R),
    );
    transitions.insert(
        ("q1".to_string(), '0'),
        ("q1".to_string(), '0', Direction::R),
    );
    transitions.insert(
        ("q1".to_string(), '1'),
        ("q0".to_string(), '1', Direction::R),
    );
    transitions.insert(
        ("q1".to_string(), '_'),
        ("reject".to_string(), '_', Direction::R),
    );

    let even_ones = TuringMachine::new(
        ["q0", "q1", "accept", "reject"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        ['0', '1'].iter().cloned().collect(),
        ['0', '1', '_'].iter().cloned().collect(),
        transitions,
        "q0".to_string(),
        ["accept"].iter().map(|s| s.to_string()).collect(),
        ["reject"].iter().map(|s| s.to_string()).collect(),
        '_',
    )
    .unwrap();
    examples.insert("even_ones".to_string(), even_ones);

    // Machine 2: Accept all strings
    let mut transitions = HashMap::new();
    transitions.insert(
        ("q0".to_string(), '0'),
        ("q0".to_string(), '0', Direction::R),
    );
    transitions.insert(
        ("q0".to_string(), '1'),
        ("q0".to_string(), '1', Direction::R),
    );
    transitions.insert(
        ("q0".to_string(), 'a'),
        ("q0".to_string(), 'a', Direction::R),
    );
    transitions.insert(
        ("q0".to_string(), 'b'),
        ("q0".to_string(), 'b', Direction::R),
    );
    transitions.insert(
        ("q0".to_string(), '_'),
        ("accept".to_string(), '_', Direction::R),
    );

    let accept_all = TuringMachine::new(
        ["q0", "accept"].iter().map(|s| s.to_string()).collect(),
        ['0', '1', 'a', 'b'].iter().cloned().collect(),
        ['0', '1', 'a', 'b', '_'].iter().cloned().collect(),
        transitions,
        "q0".to_string(),
        ["accept"].iter().map(|s| s.to_string()).collect(),
        HashSet::new(),
        '_',
    )
    .unwrap();
    examples.insert("accept_all".to_string(), accept_all);

    examples
}

/// Print the main menu
fn print_menu() {
    println!("\n{}", "=".repeat(60));
    println!("TURING MACHINE EXECUTOR");
    println!("{}", "=".repeat(60));
    println!("1. Run example machine");
    println!("2. Define custom machine (JSON format)");
    println!("3. Load machine from file");
    println!("4. Help");
    println!("5. Exit");
    println!("{}", "=".repeat(60));
}

/// Print help information
fn print_help() {
    println!("\n{}", "=".repeat(60));
    println!("HELP - Turing Machine Format");
    println!("{}", "=".repeat(60));
    println!(
        r#"
A Turing machine is defined using JSON with the following structure:

{{
    "states": ["q0", "q1", "accept", "reject"],
    "alphabet": ["0", "1"],
    "tape_alphabet": ["0", "1", "_"],
    "initial_state": "q0",
    "accept_states": ["accept"],
    "reject_states": ["reject"],
    "blank_symbol": "_",
    "transitions": {{
        "q0,0": ["q0", "0", "R"],
        "q0,1": ["q1", "1", "R"],
        "q1,_": ["accept", "_", "R"]
    }}
}}

Transition format: "state,symbol": [new_state, write_symbol, direction]
Direction: "L" (left), "R" (right)

The program will:
1. Execute the machine on your input string
2. Report if it ACCEPTS or REJECTS (halts)
3. Show the final state reached
"#
    );
}

/// Run one of the predefined example machines
fn run_example_machine() {
    let examples = create_example_machines();

    println!("\n{}", "=".repeat(60));
    println!("EXAMPLE MACHINES");
    println!("{}", "=".repeat(60));
    println!("1. Even number of 1s (accepts strings with even number of 1s)");
    println!("2. Accept all (accepts any string)");
    println!("{}", "=".repeat(60));

    print!("\nSelect example (1-2): ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    let (machine_key, machine_name) = match choice {
        "1" => ("even_ones", "Even number of 1s"),
        "2" => ("accept_all", "Accept all strings"),
        _ => {
            println!("Invalid choice!");
            return;
        }
    };

    let machine = examples.get(machine_key).unwrap();
    println!("\nSelected: {}", machine_name);
    println!("{}", "-".repeat(60));

    loop {
        print!("\nEnter input string (or 'back' to return): ");
        io::stdout().flush().unwrap();
        let mut input_str = String::new();
        io::stdin().read_line(&mut input_str).unwrap();
        let input_str = input_str.trim();

        if input_str.eq_ignore_ascii_case("back") {
            break;
        }

        match machine.execute(input_str, 10000) {
            Ok(result) => {
                println!("\n{}", "-".repeat(60));
                println!("EXECUTION RESULTS");
                println!("{}", "-".repeat(60));
                println!("Input string: '{}'", input_str);
                println!("Steps executed: {}", result.steps);
                println!("Final state: {}", result.final_state);
                println!("Machine halted: {}", result.halted);

                if let Some(true) = result.accepts {
                    println!(
                        "\n✓ RESULT: ACCEPTS (halts in state {})",
                        result.final_state
                    );
                } else if let Some(false) = result.accepts {
                    println!("\n✗ RESULT: REJECTS (final state: {})", result.final_state);
                } else {
                    println!("\n? RESULT: DID NOT HALT (possible infinite loop)");
                }
                println!("{}", "-".repeat(60));
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

/// Allow user to define a custom Turing machine via JSON
fn run_custom_machine() {
    println!("\n{}", "=".repeat(60));
    println!("DEFINE CUSTOM MACHINE (JSON)");
    println!("{}", "=".repeat(60));
    println!("Enter JSON definition (type 'help' for format, 'cancel' to abort):");
    println!("You can enter it as a single line or multiple lines (end with empty line)");
    println!("{}", "-".repeat(60));

    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        let line = line.trim();

        if line.eq_ignore_ascii_case("cancel") {
            return;
        }
        if line.eq_ignore_ascii_case("help") {
            print_help();
            println!("Continue entering JSON:");
            continue;
        }
        if line.is_empty() && !lines.is_empty() {
            break;
        }
        if !line.is_empty() {
            lines.push(line.to_string());
        }
    }

    let json_str = lines.join("\n");

    match serde_json::from_str::<MachineJson>(&json_str) {
        Ok(json_data) => match parse_machine_json(&json_data) {
            Ok(machine) => {
                println!("\n✓ Machine created successfully!");
                println!("States: {}", machine.states.len());
                println!("Transitions: {}", machine.transitions.len());

                loop {
                    print!("\nEnter input string (or 'back' to return): ");
                    io::stdout().flush().unwrap();
                    let mut input_str = String::new();
                    io::stdin().read_line(&mut input_str).unwrap();
                    let input_str = input_str.trim();

                    if input_str.eq_ignore_ascii_case("back") {
                        break;
                    }

                    match machine.execute(input_str, 10000) {
                        Ok(result) => {
                            println!("\n{}", "-".repeat(60));
                            println!("EXECUTION RESULTS");
                            println!("{}", "-".repeat(60));
                            println!("Input string: '{}'", input_str);
                            println!("Steps executed: {}", result.steps);
                            println!("Final state: {}", result.final_state);
                            println!("Machine halted: {}", result.halted);

                            if let Some(true) = result.accepts {
                                println!(
                                    "\n✓ RESULT: ACCEPTS (halts in state {})",
                                    result.final_state
                                );
                            } else if let Some(false) = result.accepts {
                                println!(
                                    "\n✗ RESULT: REJECTS (final state: {})",
                                    result.final_state
                                );
                            } else {
                                println!("\n? RESULT: DID NOT HALT (possible infinite loop)");
                            }
                            println!("{}", "-".repeat(60));
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
            Err(e) => println!("Error creating machine: {}", e),
        },
        Err(e) => println!("Invalid JSON: {}", e),
    }
}

/// Load a Turing machine definition from a JSON file
fn load_machine_from_file() {
    println!("\n{}", "=".repeat(60));
    println!("LOAD MACHINE FROM FILE");
    println!("{}", "=".repeat(60));

    print!("Enter filename (or 'cancel' to abort): ");
    io::stdout().flush().unwrap();
    let mut filename = String::new();
    io::stdin().read_line(&mut filename).unwrap();
    let filename = filename.trim();

    if filename.eq_ignore_ascii_case("cancel") {
        return;
    }

    match fs::read_to_string(filename) {
        Ok(json_str) => match serde_json::from_str::<MachineJson>(&json_str) {
            Ok(json_data) => match parse_machine_json(&json_data) {
                Ok(machine) => {
                    println!("\n✓ Machine loaded successfully!");
                    println!("States: {}", machine.states.len());
                    println!("Transitions: {}", machine.transitions.len());

                    loop {
                        print!("\nEnter input string (or 'back' to return): ");
                        io::stdout().flush().unwrap();
                        let mut input_str = String::new();
                        io::stdin().read_line(&mut input_str).unwrap();
                        let input_str = input_str.trim();

                        if input_str.eq_ignore_ascii_case("back") {
                            break;
                        }

                        match machine.execute(input_str, 10000) {
                            Ok(result) => {
                                println!("\n{}", "-".repeat(60));
                                println!("EXECUTION RESULTS");
                                println!("{}", "-".repeat(60));
                                println!("Input string: '{}'", input_str);
                                println!("Steps executed: {}", result.steps);
                                println!("Final state: {}", result.final_state);
                                println!("Machine halted: {}", result.halted);

                                if let Some(true) = result.accepts {
                                    println!(
                                        "\n✓ RESULT: ACCEPTS (halts in state {})",
                                        result.final_state
                                    );
                                } else if let Some(false) = result.accepts {
                                    println!(
                                        "\n✗ RESULT: REJECTS (final state: {})",
                                        result.final_state
                                    );
                                } else {
                                    println!("\n? RESULT: DID NOT HALT (possible infinite loop)");
                                }
                                println!("{}", "-".repeat(60));
                            }
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                }
                Err(e) => println!("Error creating machine: {}", e),
            },
            Err(e) => println!("Invalid JSON in file: {}", e),
        },
        Err(e) => println!("File error: {}", e),
    }
}

/// Run example machines for demonstration
fn run_examples() {
    println!("Turing Machine Executor - Examples\n");

    let examples = create_example_machines();

    // Test even ones machine
    println!("{}", "=".repeat(60));
    println!("Machine: Even number of 1s");
    println!("{}", "=".repeat(60));

    let machine = examples.get("even_ones").unwrap();
    let test_cases = ["", "0", "1", "11", "101", "111", "0101", "1111"];

    for test in &test_cases {
        let result = machine.execute(test, 10000).unwrap();
        print!("Input: '{}' -> ", test);
        if let Some(true) = result.accepts {
            println!(
                "ACCEPTS (state: {}, steps: {})",
                result.final_state, result.steps
            );
        } else {
            println!(
                "REJECTS (state: {}, steps: {})",
                result.final_state, result.steps
            );
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Machine: Accept all strings");
    println!("{}", "=".repeat(60));

    let machine = examples.get("accept_all").unwrap();
    let test_cases = ["", "ab", "01010", "111"];

    for test in &test_cases {
        let result = machine.execute(test, 10000).unwrap();
        print!("Input: '{}' -> ", test);
        if let Some(true) = result.accepts {
            println!(
                "ACCEPTS (state: {}, steps: {})",
                result.final_state, result.steps
            );
        } else {
            println!(
                "REJECTS (state: {}, steps: {})",
                result.final_state, result.steps
            );
        }
    }
}

fn main() {
    // Check if running in example mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--examples" {
        run_examples();
        return;
    }

    println!("\nWelcome to the Turing Machine Executor!");
    println!("This program allows you to execute Turing machines and determine:");
    println!("  1. If the machine accepts/rejects the input (halts)");
    println!("  2. The final state reached by the machine");

    loop {
        print_menu();
        print!("\nSelect option (1-5): ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => run_example_machine(),
            "2" => run_custom_machine(),
            "3" => load_machine_from_file(),
            "4" => print_help(),
            "5" => {
                println!("\nThank you for using the Turing Machine Executor!");
                break;
            }
            _ => println!("Invalid choice! Please select 1-5."),
        }
    }
}
