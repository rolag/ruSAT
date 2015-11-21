use std::collections::{BTreeSet, HashSet};
use std::collections::btree_set::Iter;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ClauseType {
    Unknown,
    Tautology,      // Always true    -- all interpretations are models
    Satisfiable,    // Sometimes true -- has some models
    Unsatisfiable,  // Never true     -- no models
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
/// A clause in clausal normal form (CNF) i.e. a disjunction (∨) of literals
pub struct CNFClause {
    // Ordered set of literals, as order doesn't matter and the amount of times a literal occurs
    // doesn't matter: (a ∨ a) <=> (a)
    literals: BTreeSet<isize>,
}

impl CNFClause {

    pub fn new() -> CNFClause {
        CNFClause{ literals: BTreeSet::new() }
    }

    /// If the clause contains only one literal, return it as Some(Literal). Otherwise, return
    /// None.
    pub fn get_unit(&self) -> Option<&isize> {
        if self.len() == 1 {
            self.literals.iter().next()
        } else {
            None
        }
    }

    /// Add a new literal to the clause
    /// Returns true if value was not already present in the set
    pub fn add(&mut self, literal: isize) -> bool {
        match literal {
            0 => panic!("Can't insert a literal of value zero"),
            x => self.literals.insert(x),
        }
    }

    /// Removes a literal from the set, returning true if value was present in the set
    pub fn remove(&mut self, literal: &isize) -> bool {
        self.literals.remove(&literal)
    }

    /// Returns an iterator over the literals
    pub fn iter(&self) -> Iter<isize> {
        self.literals.iter()
    }

    /// Returns true if this clause contains the literal
    pub fn contains(&self, literal: isize) -> bool {
        self.literals.contains(&literal)
    }

    /// Returns the amount of elements in the set
    pub fn len(&self) -> usize {
        self.literals.len()
    }

    /// If the clause contains no literals, then it adds a literal zero
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

} // impl CNFClause

#[test]
fn test_cnf_clause() {
    let t1: isize = 1;
    let f1: isize = -1;
    let mut clause = CNFClause::new();
    clause.add(t1);
    clause.add(f1);

    // Assert n1,¬n1 is a tautology
    assert!(clause.is_tautology());

    clause.remove(&t1);
    let mut clause2 = CNFClause::new();
    clause2.add(f1);
    assert_eq!(clause, clause2);

    clause.remove(&-1);
    assert_eq!(clause, CNFClause::new());

    // Clear
    let mut clause  = CNFClause::new();
    let mut clause2 = CNFClause::new();

    clause.add(-5);
    clause.add(4);
    clause.add(2);

    // Add n2 and assert it contains() it
    clause2.add(2);
    assert!(clause2.contains(2));

    // Remove it and assert it doesn't contain it and it's length is zero
    clause2.remove(&2);
    assert!(!clause2.contains(2));
    assert_eq!(0, clause2.len());

    // Now make clause and clause2 have the equivalent literals in reverse order
    clause2.add(2);
    clause2.add(4);
    clause2.add(-5);

    // ...and assert they're still equal because it should be a sorted set
    assert_eq!(clause, clause2);

    // they shouldn't be tautologies yet...
    assert!(!clause.is_tautology());
    assert!(!clause2.is_tautology());

    // ...but now they should be!
    clause2.add(-2);
    assert!(clause != clause2);
    clause.add(-2);
    assert_eq!(clause, clause2);
    assert!(clause.is_tautology());

    // Assert that adding the same element means they're still equal (it's a set)
    clause2.add(2);
    assert_eq!(clause, clause2);
}



/// A conjunction (∧) of clauses
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct CNFSystem {
    pub clauses: HashSet<CNFClause>,
}

impl CNFSystem {
    pub fn new(initial_clauses: Option<HashSet<CNFClause>>) -> CNFSystem {
        match initial_clauses {
            Some(c) => CNFSystem{ clauses: c, },
            None    => CNFSystem{ clauses: HashSet::new(), },
        }
    }

    /// Checks if a clause contains a unit literal and removes the clause that contain just that
    /// literal. Returns an Option of the literal.
    pub fn take_unit_clause(&mut self) -> Option<isize> {
        let mut literal: isize = 0; // Can't have literal of value zero
        for each_clause in self.clauses.iter() {
            match each_clause.get_unit() {
                Some(l) => { literal = l.clone(); break; },
                None => continue,
            }
        }
        match literal {
            0 => None,
            x => {
                let mut clause = CNFClause::new();
                clause.add(x.clone());
                self.clauses.remove(&clause);
                Some(x)
            },
        }
    }

    /// Returns true if the system already contains a clause
    pub fn contains(&self, other_clause: &CNFClause) -> bool {
        self.clauses.contains(&other_clause)
    }

    /// Add a clause to the system. Returns false if the value was already in the system
    pub fn add_clause(&mut self, clause: CNFClause) -> bool {
        self.clauses.insert(clause)
    }

    /// Removes a clause from the system. Returns false if the value wasn't already in the system
    pub fn remove_clause(&mut self, clause: &CNFClause) -> bool {
        self.clauses.remove(clause)
    }

    /// Return the amount of clauses in the system
    pub fn len(&self) -> usize {
        self.clauses.len()
    }
}
