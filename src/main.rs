use std::{io, process, env};
use std::io::prelude::*;
use std::collections::HashSet;
mod cnf_system;
use cnf_system::{CNFSystem, CNFClause, ClauseType};
mod DPLL;
use DPLL::{concurrent_dpll};

// Show help and exit
fn show_help(program_name: String) {
    println!("Usage: {} [options]", program_name);
    println!("Mandatory arguments to long options are mandatory for short options too.");
    println!("
-f, --file FILE         Read in the system from FILE. If FILE is ``-'', then
                        input is read from stdin. Default: ``-''.
-v, --version           Output version and exit, regardless of other arguments.
-h, -?, --help          Output usage and exit, regardless of other arguments."
            );
}

fn show_version() {
    println!("ruSAT  Copyright (C) 2015");
    println!("This program comes with ABSOLUTELY NO WARRANTY");
    println!("License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>");
    println!("This is free software, and you are welcome to redistribute it");
    println!("under certain conditions.");
    println!("");
    println!("Home: https://github.com/rolag/ruSAT/");
}

fn get_next_arg_or_err<'a>(program_name: &str, args: &'a [String], current_index: usize) -> &'a str {
    let arg_count = args.len();
    if current_index == arg_count - 1 {
        error_and_exit(program_name,
                       format!("necessary argument to {} not given", args[arg_count - 1]),
                       22);
    } else {
        &args[current_index + 1]
    }
}

fn error_and_exit(program_name: &str, error_message: String, exit_code: i32) -> ! {
    println!("{}: {}", program_name, error_message);
    process::exit(exit_code);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();
    let arg_count: usize = args.len();
    if arg_count == 1 {
        show_help(program_name);
        process::exit(0);
    }

    // Set argument defaults
    let mut input_file = "-";

    // Loop through each argument, changing argument options when necessary
    let mut arg_index = 1;
    while arg_index < arg_count {
        match args[arg_index].as_ref() {
            "-f" | "--file" => {
                input_file = get_next_arg_or_err(&program_name, &args, arg_index);
                arg_index += 1;
            },
            "-v" | "--version" => {
                show_version();
                process::exit(0);
            },
            "-h" | "-?" | "--help" => {
                show_help(program_name);
                process::exit(0);
            },
            bad_option => {
                error_and_exit(&program_name,
                               format!("not and option: {}", bad_option),
                               22
                               );
            },
        }
        arg_index += 1;
    }

    let input;
    if input_file == "-" {
        // Read in CNF system from stdin in dimacs form, for now
        input = io::stdin();
    } else {
        // TODO: Read from file
        println!("reading from file not implemented yet");
        process::exit(38);
    }

    let mut system = CNFSystem::new(None);
    let mut contains_tautologies = false;
    // The units that exist in the input system, before any algorithm is applied to it
    let mut units = HashSet::new();

    // Skip all comment lines i.e. a line that begins with 'c' and the program line i.e. a line
    // like 'p VARIABLE_COUNT CLAUSE_COUNT'
    'next_line: for lines in input.lock().lines() {
        let current_line: String = lines.unwrap(); // expect valid string from stdin
        let words = current_line.split_whitespace().collect::<Vec<_>>();
        if let Some(first_word) = words.iter().next() {
            if let Some(first_char) = first_word.chars().next() {
                if first_char == 'c' || first_char == 'p' {
                    continue;
                }
            } else {
                continue;
            }
        } else {
            continue;
        }
        // Now, insert the actual input into the system
        let mut current_clause = CNFClause::new();
        for each_word in words {
            // Convert word to integer
            let literal = match each_word.parse::<isize>() {
                Ok(word) => word,
                Err(_) => error_and_exit(&program_name,
                                         format!("not a valid comment, program or input line: {}",
                                                 current_line),
                                         22),
            };
            // Check for tautologies
            if literal == 0 {
                break;
            }
            if current_clause.contains(-literal) {
                contains_tautologies = true;
                continue 'next_line;
            } else {
                current_clause.add(literal);
            }
        }
        if current_clause.len() > 0 {
            if current_clause.len() == 1 {
                units.insert(current_clause.iter().next().unwrap().clone());
            }
            system.add_clause(current_clause);
        }
    }

    if system.len() == 0 {
        if contains_tautologies {
            println!("TAUTOLOGY");
        } else {
            error_and_exit(&program_name, format!("you need to enter a system"), 22);
        }
    } else {
        //println!("System: {:?}", system);

        // Find if the system is satisfiable or unsatisfiable or tautology
        match concurrent_dpll(&mut system, units, 16) {
            (ClauseType::Tautology, _)     => println!("TAUTOLOGY"),
            (ClauseType::Satisfiable, interpretation) => println!("SATISFIABLE: {:?}", interpretation),
            (ClauseType::Unsatisfiable, _) => println!("UNSATISFIABLE"),
        }
    }
}
