use cnf_system::{CNFClause, CNFSystem, ClauseType};
use std::collections::{BTreeSet, HashSet};
use std::sync::mpsc;
use std::thread;

/// Applies unit propagation of a literal l to a system.
///     If a clause contains: l, then remove that entire clause
///     If a clause contains: not(l), then remove not(l) from the clause but keep the others
/// Returns (true, new_units) if successful, (false, _) if a set contradicts another
pub fn concurrent_dpll_propagate(system: &mut CNFSystem, literal: isize)
                                 -> Option<HashSet<isize>> {
    let mut new_units = HashSet::new();
    let mut clauses_to_remove: Vec<CNFClause> = vec![];
    let mut clauses_to_reduce: Vec<CNFClause> = vec![];

    for each_clause in system.clauses.iter().cloned() {
        if each_clause.contains(literal) {
            clauses_to_remove.push(each_clause);
        } else if each_clause.contains(-literal) {
            clauses_to_reduce.push(each_clause);
        }
    }

    for each_clause in clauses_to_remove {
        system.remove_clause(&each_clause);
    }

    for mut each_clause in clauses_to_reduce {
        // Have to remove and then add because it's a hash
        // Check if successful because it could have been removed by the clauses_to_remove vector
        if system.remove_clause(&each_clause) {
            each_clause.remove(&-literal);
            match each_clause.len() {
                0 => { return None; },
                1 => {
                    new_units.insert(each_clause.iter()          // get literals iterator
                                                .next().unwrap() // get first literal
                                                .clone()         // clone it and insert it
                                    );
                },
                _ => {},
            };
            system.add_clause(each_clause);
        }
    }
    Some(new_units)
}

/// Takes in a system (without any tautologies, as they can be optimised out when parsed), and
/// return if it's Satisfiable or Unsatisfiable using a concurrent version of the DPLL algorithm.
/// Assumes that there's at least one clause in the system
pub fn concurrent_dpll(mut system: CNFSystem, units: HashSet<isize>, thread_count: isize)
                      -> (ClauseType, BTreeSet<isize>) {
    let mut interpretation: BTreeSet<isize> = BTreeSet::new();
    let mut current_units = units;

    // Propagate units until you can't propagate anymore
    while current_units.len() > 0 {
        // The new units revealed by previous unit propagation
        let mut revealed_units = HashSet::new();
        for each_unit_literal in current_units {
            match concurrent_dpll_propagate(&mut system, each_unit_literal) {
                None            => { return (ClauseType::Unsatisfiable, interpretation); },
                Some(new_units) => {
                    revealed_units.extend(new_units);
                    interpretation.insert(each_unit_literal);
                    if system.len() == 0 {
                        return (ClauseType::Satisfiable, interpretation);
                    }
                }
            }
        }
        current_units = revealed_units;
    }

    // Now, pick a new random variable and work out if the system is satisfiable with variable and
    // not(variable).

    // Get the first arbitrary literal in the first arbitrary clause (this assumes that
    // there's at least one clause in the set)
    let some_literal = system.clauses.iter().next().unwrap().iter().next().unwrap().clone();

    // Create two new units hashes to send to the next instances of concurrent_dpll
    let mut positive_clause = HashSet::new();
    let mut negative_clause = HashSet::new();
    positive_clause.insert( some_literal);
    negative_clause.insert(-some_literal);

    // Create a new system for the new branch
    let system2 = system.clone();

    // Create a channel to send messages between the new threads
    let (sender, receiver) = mpsc::channel();
    let sender1 = sender.clone();
    let sender2 = sender.clone();

    // Spawn threads for each system. We can call unwrap() on the join() methods because DPLL is
    // sound and the only way for this unwrap to panic is for the spawned concurrent_dpll() to
    // panic
    if thread_count >= 2 {
        thread::spawn(move || {
            sender1.send(concurrent_dpll(system, positive_clause, thread_count - 2)).unwrap();
        }).join().unwrap();
        thread::spawn(move || {
            let system = system2;
            sender2.send(concurrent_dpll(system, negative_clause, thread_count - 2)).unwrap();
        }).join().unwrap();
    } else {
        thread::spawn(move || {
            sender1.send(concurrent_dpll(system, positive_clause, 0)).unwrap();
        }).join().unwrap();
        sender2.send(concurrent_dpll(system2, negative_clause, 0)).unwrap();
    }

    // Now, wait for one (or both) of the threads to come back with a result
    let mut thread_result_iterator = receiver.iter();
    match thread_result_iterator.next().unwrap() {
        (ClauseType::Unsatisfiable, _) => {
            // Wait for other result
            match thread_result_iterator.next().expect("sent two messages but only received one") {
                (ClauseType::Unsatisfiable, new_interpretation) => {
                    (ClauseType::Unsatisfiable, new_interpretation)
                },
                (clause_type, new_interpretation) => {
                    interpretation.extend(new_interpretation);
                    (clause_type, interpretation)
                },
            }
        },
        (clause_type, new_interpretation) => {
            interpretation.extend(new_interpretation);
            (clause_type, interpretation)
        }
    }
}
