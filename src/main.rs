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
-c, --check-sat         Checks if the input system is a tautology, satisfiable
                        or unsatisfiable.
-i, --input-type TYPE   Input the system as using TYPE format. Possible values
                        are 'dimacs'. Default: dimacs.
-f, --file FILE         Read in the system from FILE. If FILE is ``-'', then
                        input is read from stdin. Default: ``-''.
-m, --models            Returns all models of the system or print unsatisfiable.
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

fn check_option(program_name: String, option: &String, option_values: Vec<&str>, option_name: &str) {
    let mut option_is_valid = false;
    for each_value in &option_values {
        if each_value == option {
            option_is_valid = true;
            break;
        }
    }
    if !option_is_valid {
        error_and_exit(program_name,
                       format!("invalid value for argument '{}': '{}'", option_name, option),
                       22);
    }
}

fn get_next_arg_or_err(program_name: String, args: &Vec<String>, current_index: usize) -> String {
    let arg_count = args.len();
    if current_index == arg_count - 1 {
        error_and_exit(program_name,
                       format!("necessary argument to {} not given", args[arg_count - 1]),
                       22);
    } else {
        args[current_index + 1].clone()
    }
}

fn error_and_exit(program_name: String, error_message: String, exit_code: i32) -> ! {
    println!("{}: {}", program_name, error_message);
    process::exit(exit_code);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();
    let arg_count: usize = args.len();
    //if arg_count == 1 {
    //    show_help(program_name);
    //    process::exit(0);
    //}

    // Set argument defaults
    let mut input_type = "dimacs".to_string();
    let input_types = vec!["dimacs"];

    let mut input_file = "-".to_string();

    let mut check_type = "sat-check".to_string();
    //let check_types = vec!["sat-check", "model-check"];

    // Loop through each argument, changing argument options when necessary
    let mut arg_index = 1;
    while arg_index < arg_count {
        match args[arg_index].as_ref() {
            "-c" | "--check-sat" | "--sat-check" => {
                check_type = "sat-check".to_string();
            },
            "-m" | "--models" | "--get-models" | "--get-all-models" => {
                check_type = "model-check".to_string();
            },
            option if (option == "-i") | (option == "--input-type") => {
                input_type = get_next_arg_or_err(program_name.clone(), &args, arg_index);
                check_option(program_name.clone(), &input_type, input_types.clone(), option );
                arg_index += 1;
            },
            "-f" | "--file" => {
                input_file = get_next_arg_or_err(program_name.clone(), &args, arg_index);
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
                error_and_exit(program_name,
                               format!("not and option: {}", bad_option),
                               22
                               );
            },
        }
        arg_index += 1;
    }

    // TODO: implement algorithm for getting all models in CNF form
    if check_type == "model-check" {
        println!("model check not implemented yet");
        process::exit(38);
    }

    let mut input;
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
        let current_line: String = lines.unwrap(); // expect string from iterator
        let words = current_line.split_whitespace().collect::<Vec<_>>();
        if words.len() == 0 {
            continue;
        }
        let first_char = words.iter().next().unwrap()   // get first word
                              .chars().next().unwrap(); // get first char (of first word)
        if first_char == 'c'  || first_char == 'p' {
            //println!("Ignoring: {:?}", words);
            continue;
        }
        // Now, insert the actual input into the system
        let mut current_clause = CNFClause::new();
        for each_word in words {
            // Convert word to integer
            let literal = match each_word.parse::<isize>() {
                Ok(word) => word,
                Err(_) => error_and_exit(program_name,
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
            error_and_exit(program_name, format!("you need to enter a system"), 22);
        }
    } else {
        //println!("System: {:?}", system);

        // Find if the system is satisfiable or unsatisfiable or tautology
        match concurrent_dpll(&mut system, units, 16) {
            (ClauseType::Tautology, _)     => println!("TAUTOLOGY"),
            (ClauseType::Satisfiable, interpretation) => println!("SATISFIABLE: {:?}", interpretation),
            (ClauseType::Unsatisfiable, _) => println!("UNSATISFIABLE"),
            (ClauseType::Unknown, _)       => println!("UNKNOWN"),
        }
    }
}
