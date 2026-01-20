use colored::Colorize;
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

/// State snapshot during step-by-step execution
#[derive(Debug, Clone)]
struct ExecutionSnapshot {
    tape: Vec<char>,
    head_position: i32,
    current_state: String,
    step: usize,
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

    /// Execute the machine step-by-step, returning snapshots
    fn execute_step_by_step(
        &self,
        input_string: &str,
        max_steps: usize,
    ) -> Result<Vec<ExecutionSnapshot>, String> {
        let mut snapshots = Vec::new();

        // Initialize tape with input
        let mut tape: Vec<char> = if input_string.is_empty() {
            vec![]
        } else {
            input_string.chars().collect()
        };
        let mut head_position: i32 = 0;
        let mut current_state = self.initial_state.clone();
        let mut step = 0;

        // Validate input symbols
        for symbol in input_string.chars() {
            if !self.alphabet.contains(&symbol) {
                return Err(format!("Invalid input symbol: {}", symbol));
            }
        }

        // Save initial snapshot
        snapshots.push(ExecutionSnapshot {
            tape: tape.clone(),
            head_position,
            current_state: current_state.clone(),
            step,
        });

        // Execute until halt or max steps
        while step < max_steps {
            // Check if in halting state
            if self.accept_states.contains(&current_state)
                || self.reject_states.contains(&current_state)
            {
                break;
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
                step += 1;

                // Save snapshot after transition
                snapshots.push(ExecutionSnapshot {
                    tape: tape.clone(),
                    head_position,
                    current_state: current_state.clone(),
                    step,
                });
            } else {
                // No transition defined - halt
                break;
            }
        }

        Ok(snapshots)
    }

    /// Display the state diagram with transitions
    fn display_state_diagram(&self, current_state: Option<&str>, next_transition: Option<(char, &str, char, Direction)>) {
        println!("\n{}", "=".repeat(60));
        println!("{}", "STATE DIAGRAM".bold());
        println!("{}", "=".repeat(60));

        // Draw visual ASCII diagram
        self.draw_state_diagram(current_state, next_transition);

        // Display transitions grouped by state
        println!("\n{}:", "Transitions".bold());
        let mut transitions_by_state: HashMap<&String, Vec<(char, &String, char, Direction)>> =
            HashMap::new();

        for ((state, symbol), (new_state, write_symbol, direction)) in &self.transitions {
            transitions_by_state
                .entry(state)
                .or_insert_with(Vec::new)
                .push((*symbol, new_state, *write_symbol, *direction));
        }

        let mut sorted_states: Vec<_> = transitions_by_state.keys().collect();
        sorted_states.sort();

        for state in sorted_states {
            let mut state_header = format!("  {}:", state);
            if let Some(current) = current_state {
                if state.as_str() == current {
                    state_header = state_header.bold().yellow().to_string();
                }
            }
            println!("{}", state_header);

            let mut transitions = transitions_by_state.get(state).unwrap().clone();
            transitions.sort_by_key(|(s, _, _, _)| *s);

            for (symbol, new_state, write_symbol, direction) in transitions {
                let dir_str = match direction {
                    Direction::L => "←",
                    Direction::R => "→",
                };
                let transition_str = format!(
                    "    ({}) → write '{}', move {}, goto {}",
                    symbol, write_symbol, dir_str, new_state
                );

                // Highlight the next transition to be executed
                let is_next_transition = if let (Some(current), Some((next_sym, next_state, _, _))) = (current_state, next_transition) {
                    state.as_str() == current && symbol == next_sym && new_state.as_str() == next_state
                } else {
                    false
                };

                if is_next_transition {
                    println!("{}", format!("  ▶ {}", transition_str).bold().green());
                } else if let Some(current) = current_state {
                    if state.as_str() == current {
                        println!("{}", transition_str.yellow());
                    } else {
                        println!("{}", transition_str);
                    }
                } else {
                    println!("{}", transition_str);
                }
            }
        }
        println!();
    }

    /// Draw ASCII art diagram of state machine
    fn draw_state_diagram(&self, current_state: Option<&str>, next_transition: Option<(char, &str, char, Direction)>) {
        println!("\n{}:", "Visual Diagram".bold());
        
        // Sort states for consistent display
        let mut sorted_states: Vec<_> = self.states.iter().collect();
        sorted_states.sort();
        
        // Draw states with arrows connecting them
        // Create a simple horizontal layout with arrows
        for (i, state) in sorted_states.iter().enumerate() {
            // Draw state box
            let is_current = current_state.map(|c| c == state.as_str()).unwrap_or(false);
            let is_accept = self.accept_states.contains(*state);
            let is_reject = self.reject_states.contains(*state);
            
            // State box components
            let box_top = "┌──────────┐";
            let state_line = format!("│ {:^8} │", state.as_str());
            let type_line = if is_accept {
                "│ ✓ ACCEPT │"
            } else if is_reject {
                "│ ✗ REJECT │"
            } else {
                "│          │"
            };
            let box_bottom = "└──────────┘";
            
            // Print state box
            if is_current {
                println!("  {}", box_top.bold().yellow());
                println!("  {}", state_line.bold().yellow());
                if is_accept {
                    println!("  {}", type_line.green().bold().yellow());
                } else if is_reject {
                    println!("  {}", type_line.red().bold().yellow());
                } else {
                    println!("  {}", type_line.bold().yellow());
                }
                println!("  {}", box_bottom.bold().yellow());
            } else {
                println!("  {}", box_top);
                println!("  {}", state_line);
                if is_accept {
                    println!("  {}", type_line.green());
                } else if is_reject {
                    println!("  {}", type_line.red());
                } else {
                    println!("  {}", type_line);
                }
                println!("  {}", box_bottom);
            }
            
            // Draw transitions from this state
            let mut state_transitions = Vec::new();
            for ((from_state, symbol), (to_state, write_symbol, direction)) in &self.transitions {
                if from_state == *state {
                    state_transitions.push((symbol, to_state.as_str(), write_symbol, direction));
                }
            }
            
            if !state_transitions.is_empty() {
                state_transitions.sort_by_key(|(s, _, _, _)| *s);
                
                for (symbol, to_state, write_symbol, direction) in state_transitions {
                    let dir_arrow = match direction {
                        Direction::L => "←",
                        Direction::R => "→",
                    };
                    
                    // Check if this is the next transition
                    let is_next = if let (Some(current), Some((next_sym, next_state, _, _))) = (current_state, next_transition) {
                        state.as_str() == current && *symbol == next_sym && to_state == next_state
                    } else {
                        false
                    };
                    
                    let arrow = format!("      │ {} --[{}:{}{}]-->  {}", 
                        state.as_str(), symbol, write_symbol, dir_arrow, to_state);
                    
                    if is_next {
                        println!("{}", arrow.bold().green());
                    } else if is_current {
                        println!("{}", arrow.yellow());
                    } else {
                        println!("{}", arrow);
                    }
                }
                println!("      ↓");
            }
            
            if i < sorted_states.len() - 1 {
                println!();
            }
        }
        
        // Show next transition if available
        if let (Some(current), Some((symbol, next_state, write_symbol, direction))) = (current_state, next_transition) {
            println!("\n{}:", "Next Transition".bold().green());
            let dir_str = match direction {
                Direction::L => "←",
                Direction::R => "→",
            };
            println!("  {} --[read: '{}']-->", current.bold().yellow(), symbol.to_string().cyan());
            println!("    • Write: '{}'", write_symbol.to_string().cyan());
            println!("    • Move: {}", dir_str.cyan());
            println!("    • Goto: {}", next_state.bold().yellow());
        }
        
        println!();
    }

    /// Display the tape with head position
    fn display_tape(snapshot: &ExecutionSnapshot, blank_symbol: char) {
        println!("\n{}", "TAPE".bold());
        
        // Determine visible range around head
        let head_pos = snapshot.head_position;
        let tape_len = snapshot.tape.len() as i32;
        
        // Show at least 20 cells centered around head
        let visible_start = (head_pos - 10).max(0);
        let visible_end = (head_pos + 10).min(tape_len - 1).max(visible_start + 19);
        
        // Print tape cells
        print!("Tape:   ");
        for i in visible_start..=visible_end {
            if i >= 0 && i < tape_len {
                let cell = snapshot.tape[i as usize];
                let cell_str = if cell == blank_symbol {
                    format!("[_]")
                } else {
                    format!("[{}]", cell)
                };
                
                if i == head_pos {
                    print!("{}", cell_str.bold().green());
                } else {
                    print!("{}", cell_str);
                }
            } else {
                print!("[_]");
            }
        }
        println!();
        
        // Print head indicator
        print!("Head:   ");
        for i in visible_start..=visible_end {
            if i == head_pos {
                print!(" ^ ");
            } else {
                print!("   ");
            }
        }
        println!();
        
        // Print position numbers
        print!("Pos:    ");
        for i in visible_start..=visible_end {
            print!("{:>3}", i);
        }
        println!("\n");
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

/// Format a filename into a display name
fn format_display_name(filename: &str) -> String {
    filename
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Load example Turing machines from the examples folder
fn load_example_machines() -> HashMap<String, (TuringMachine, String)> {
    let mut examples = HashMap::new();
    
    // Try to load examples from the examples directory
    if let Ok(entries) = fs::read_dir("examples") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            
            let Some(filename) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            
            let Ok(json_str) = fs::read_to_string(&path) else {
                continue;
            };
            
            let Ok(json_data) = serde_json::from_str::<MachineJson>(&json_str) else {
                continue;
            };
            
            let Ok(machine) = parse_machine_json(&json_data) else {
                continue;
            };
            
            let display_name = format_display_name(filename);
            examples.insert(filename.to_string(), (machine, display_name));
        }
    }
    
    examples
}

/// Create example Turing machines for testing (fallback if no examples folder)
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
    // Try to load examples from the examples folder
    let loaded_examples = load_example_machines();
    
    // Prepare the examples list
    let examples_list: Vec<(String, String)> = if loaded_examples.is_empty() {
        let fallback = create_example_machines();
        let mut list: Vec<(String, String)> = fallback
            .keys()
            .map(|key| (key.clone(), format_display_name(key)))
            .collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    } else {
        let mut list: Vec<(String, String)> = loaded_examples
            .iter()
            .map(|(key, (_, display_name))| (key.clone(), display_name.clone()))
            .collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    };

    println!("\n{}", "=".repeat(60));
    println!("EXAMPLE MACHINES");
    println!("{}", "=".repeat(60));
    
    for (i, (_, display_name)) in examples_list.iter().enumerate() {
        println!("{}. {}", i + 1, display_name);
    }
    
    println!("{}", "=".repeat(60));

    print!("\nSelect example (1-{}): ", examples_list.len());
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    
    let choice_num = match choice.trim().parse::<usize>() {
        Ok(num) if num > 0 && num <= examples_list.len() => num,
        _ => {
            println!("Invalid choice!");
            return;
        }
    };

    let (machine_key, machine_name) = &examples_list[choice_num - 1];
    
    // Load the machine
    let machine = if !loaded_examples.is_empty() {
        match loaded_examples.get(machine_key) {
            Some((m, _)) => m,
            None => {
                println!("Machine '{}' not found!", machine_key);
                return;
            }
        }
    } else {
        let fallback = create_example_machines();
        if fallback.contains_key(machine_key) {
            // Reload to avoid lifetime issues
            return run_single_example(machine_key, machine_name);
        } else {
            println!("Machine '{}' not found!", machine_key);
            return;
        }
    };
    
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

        // Ask if user wants visual mode
        print!("Run in visual step-by-step mode? (y/n): ");
        io::stdout().flush().unwrap();
        let mut visual_mode = String::new();
        io::stdin().read_line(&mut visual_mode).unwrap();
        let visual_mode = visual_mode.trim().eq_ignore_ascii_case("y");

        if visual_mode {
            run_visual_mode(machine, input_str);
        } else {
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
}

/// Run a single example machine (helper for fallback case)
fn run_single_example(machine_key: &str, machine_name: &str) {
    let examples = create_example_machines();
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

        // Ask if user wants visual mode
        print!("Run in visual step-by-step mode? (y/n): ");
        io::stdout().flush().unwrap();
        let mut visual_mode = String::new();
        io::stdin().read_line(&mut visual_mode).unwrap();
        let visual_mode = visual_mode.trim().eq_ignore_ascii_case("y");

        if visual_mode {
            run_visual_mode(machine, input_str);
        } else {
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

                    // Ask if user wants visual mode
                    print!("Run in visual step-by-step mode? (y/n): ");
                    io::stdout().flush().unwrap();
                    let mut visual_mode = String::new();
                    io::stdin().read_line(&mut visual_mode).unwrap();
                    let visual_mode = visual_mode.trim().eq_ignore_ascii_case("y");

                    if visual_mode {
                        run_visual_mode(&machine, input_str);
                    } else {
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

                        // Ask if user wants visual mode
                        print!("Run in visual step-by-step mode? (y/n): ");
                        io::stdout().flush().unwrap();
                        let mut visual_mode = String::new();
                        io::stdin().read_line(&mut visual_mode).unwrap();
                        let visual_mode = visual_mode.trim().eq_ignore_ascii_case("y");

                        if visual_mode {
                            run_visual_mode(&machine, input_str);
                        } else {
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
                }
                Err(e) => println!("Error creating machine: {}", e),
            },
            Err(e) => println!("Invalid JSON in file: {}", e),
        },
        Err(e) => println!("File error: {}", e),
    }
}

/// Run visual step-by-step execution mode
fn run_visual_mode(machine: &TuringMachine, input_str: &str) {
    println!("\n{}", "=".repeat(60));
    println!("{}", "VISUAL STEP-BY-STEP MODE".bold().cyan());
    println!("{}", "=".repeat(60));
    println!("Input: '{}'", input_str);
    
    // Get all execution snapshots
    match machine.execute_step_by_step(input_str, 10000) {
        Ok(snapshots) => {
            if snapshots.is_empty() {
                println!("No snapshots generated.");
                return;
            }

            let mut current_step = 0;
            let max_step = snapshots.len() - 1;

            loop {
                // Clear screen (cross-platform approach)
                print!("\x1B[2J\x1B[1;1H");
                
                let snapshot = &snapshots[current_step];
                
                println!("\n{}", "=".repeat(60));
                println!("{}", "VISUAL STEP-BY-STEP MODE".bold().cyan());
                println!("{}", "=".repeat(60));
                println!("Input: '{}'", input_str);
                println!("Step: {}/{}", current_step, max_step);
                println!("Current State: {}", snapshot.current_state.bold().yellow());
                
                // Calculate next transition
                let next_transition = if !machine.accept_states.contains(&snapshot.current_state)
                    && !machine.reject_states.contains(&snapshot.current_state)
                {
                    let head_pos = snapshot.head_position as usize;
                    let current_symbol = if head_pos < snapshot.tape.len() {
                        snapshot.tape[head_pos]
                    } else {
                        machine.blank_symbol
                    };
                    
                    machine
                        .transitions
                        .get(&(snapshot.current_state.clone(), current_symbol))
                        .map(|(next_state, write_symbol, direction)| {
                            (current_symbol, next_state.as_str(), *write_symbol, *direction)
                        })
                } else {
                    None
                };
                
                // Display state diagram with current state highlighted and next transition
                machine.display_state_diagram(Some(&snapshot.current_state), next_transition);
                
                // Display tape
                TuringMachine::display_tape(snapshot, machine.blank_symbol);
                
                // Display status
                println!("{}", "STATUS".bold());
                if machine.accept_states.contains(&snapshot.current_state) {
                    println!("✓ Machine has {} - in ACCEPT state", "HALTED".green().bold());
                } else if machine.reject_states.contains(&snapshot.current_state) {
                    println!("✗ Machine has {} - in REJECT state", "HALTED".red().bold());
                } else if current_step == max_step {
                    // Check if there's a valid transition
                    let head_pos = snapshot.head_position as usize;
                    let current_symbol = if head_pos < snapshot.tape.len() {
                        snapshot.tape[head_pos]
                    } else {
                        machine.blank_symbol
                    };
                    
                    if machine
                        .transitions
                        .contains_key(&(snapshot.current_state.clone(), current_symbol))
                    {
                        println!("Machine is running...");
                    } else {
                        println!("✗ Machine has {} - no transition defined (implicit reject)", "HALTED".red().bold());
                    }
                } else {
                    println!("Machine is running...");
                }
                
                // Navigation controls
                println!("\n{}", "=".repeat(60));
                println!("{}", "CONTROLS".bold());
                print!("Commands: ");
                if current_step > 0 {
                    print!("[{}] Previous  ", "p".bold());
                }
                if current_step < max_step {
                    print!("[{}] Next  ", "n".bold());
                }
                print!("[{}] Jump to step  [{}] Quit", "j".bold(), "q".bold());
                println!("\n{}", "=".repeat(60));
                
                print!("\nEnter command: ");
                io::stdout().flush().unwrap();
                
                let mut command = String::new();
                io::stdin().read_line(&mut command).unwrap();
                let command = command.trim().to_lowercase();
                
                match command.as_str() {
                    "n" | "next" if current_step < max_step => {
                        current_step += 1;
                    }
                    "p" | "prev" | "previous" if current_step > 0 => {
                        current_step -= 1;
                    }
                    "j" | "jump" => {
                        print!("Enter step number (0-{}): ", max_step);
                        io::stdout().flush().unwrap();
                        let mut step_str = String::new();
                        io::stdin().read_line(&mut step_str).unwrap();
                        if let Ok(step) = step_str.trim().parse::<usize>() {
                            if step <= max_step {
                                current_step = step;
                            } else {
                                println!("Invalid step number. Press Enter to continue...");
                                let mut _dummy = String::new();
                                io::stdin().read_line(&mut _dummy).unwrap();
                            }
                        }
                    }
                    "q" | "quit" | "exit" | "back" => {
                        break;
                    }
                    "" if current_step < max_step => {
                        // Enter key defaults to next
                        current_step += 1;
                    }
                    _ => {
                        println!("Invalid command. Press Enter to continue...");
                        let mut _dummy = String::new();
                        io::stdin().read_line(&mut _dummy).unwrap();
                    }
                }
            }
        }
        Err(e) => println!("Error during execution: {}", e),
    }
}

/// Run example machines for demonstration
fn run_examples() {
    println!("Turing Machine Executor - Examples\n");

    // Try to load examples from the examples folder
    let loaded_examples = load_example_machines();
    
    if !loaded_examples.is_empty() {
        // Run all loaded examples
        for (key, (machine, display_name)) in &loaded_examples {
            println!("{}", "=".repeat(60));
            println!("Machine: {}", display_name);
            println!("File: examples/{}.json", key);
            println!("{}", "=".repeat(60));
            
            // Run the machine with empty input as a basic test
            match machine.execute("", 10000) {
                Ok(result) => {
                    print!("Input: '' -> ");
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
                Err(e) => println!("Error: {}", e),
            }
            println!();
        }
    } else {
        // Fallback to hardcoded examples
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
