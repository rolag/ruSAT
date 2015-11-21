use cnf_system::{CNFClause, CNFSystem, ClauseType};
use std::collections::BTreeSet;

/// Applies unit propagation of a literal l to a system.
///     If a clause contains: l, then remove that entire clause
///     If a clause contains: not(l), then remove not(l) from the clause but keep the other
///     literals
/// Returns true if successful, false if a set contradicts another
fn basic_dpll_propagate(system: &mut CNFSystem, literal: isize) -> bool {
    let mut no_empty_clauses = true;
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
        // Check if successful because it could have been removed by the clauses_to_remove vector
        if system.remove_clause(&each_clause) {
            // Have to remove and then add because it's a hash
            each_clause.remove(&-literal);
            if no_empty_clauses == true {
                no_empty_clauses = !each_clause.is_empty();
            }
            system.add_clause(each_clause);
        }
    }
    no_empty_clauses
}

/// Returns a unit literal, i.e. a literal that appears in a clause that only contains that literal
fn basic_dpll_get_unit_literal(system: &CNFSystem) -> Option<isize> {
    for each_clause in system.clauses.iter() {
        let unit = each_clause.get_unit();
        if unit.is_some() {
            return unit.cloned();
        }
    }
    None
}

/// Takes in a system (without any tautologies, as they can be optimised out when parsed), and
/// return if it's Satisfiable or Unsatisfiable using the DPLL algorithm.
/// Assumes that there's at least one clause in the system
pub fn basic_dpll(system: &mut CNFSystem) -> (ClauseType, BTreeSet<isize>) {
    let mut interpretation: BTreeSet<isize> = BTreeSet::new();

    loop {
        match basic_dpll_get_unit_literal(system) {
            Some(literal) => {
                if !basic_dpll_propagate(system, literal) {
                    return (ClauseType::Unsatisfiable, interpretation)
                } else if system.len() == 0 {
                    interpretation.insert(literal);
                    return (ClauseType::Satisfiable, interpretation);
                }
            }
            // DPLL split: take some random literal and
            //                  Check DPLL on the system with a new clause with just that literal
            //                  Check DPLL on another system with a new clause with not(literal)
            None => {
                let other_system = &mut system.clone();
                // Get the first arbitrary literal in the first arbitrary clause (this assumes that
                // there's at least one clause in the set)
                let some_literal = system.clauses.iter()
                                                 .next().unwrap()
                                                 .get_literals()
                                                 .iter()
                                                 .next().unwrap()
                                                 .clone();
                let mut new_clause = CNFClause::new();
                new_clause.add(some_literal);
                let mut other_new_clause = CNFClause::new();
                other_new_clause.add(-some_literal);

                system.add_clause(new_clause);
                match basic_dpll(system) {
                    (ClauseType::Unsatisfiable, _) => {
                        other_system.add_clause(other_new_clause);
                        interpretation.insert(-some_literal);
                        // Must be Satisfiable or Unsatisfiable
                        match basic_dpll(other_system) {
                            (ClauseType::Satisfiable, new_interpretation) => {
                                interpretation.extend(new_interpretation.iter().cloned());
                                return (ClauseType::Satisfiable, interpretation);
                            },
                            (clause_type, new_interpretation) => {
                                return (clause_type, new_interpretation);
                            }
                        }
                    },
                    (clause_type, new_interpretation) => {
                        interpretation.insert(some_literal);
                        interpretation.extend(new_interpretation.iter().cloned());
                        return (clause_type, interpretation)
                    },
                }
            }
        }
    }

}
