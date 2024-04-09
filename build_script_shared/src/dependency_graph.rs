use nom::Err;
use std::collections::{HashMap, HashSet};

use crate::error::{ParserError, ParserErrorKind, ParserSlimResult};
use crate::parsers::Ident;

#[derive(Debug)]
pub struct DependencyGraph<'a, I> {
    graph: HashMap<&'a Ident<I>, Vec<&'a Ident<I>>>,
}

impl<'a, I> DependencyGraph<'a, I> {
    pub fn new() -> Self {
        Self {
            graph: Default::default(),
        }
    }

    pub fn contains(&self, key: &Ident<I>) -> bool {
        self.graph.contains_key(key)
    }

    pub fn add_type(&mut self, type_name: &'a Ident<I>) {
        self.graph.insert(type_name, Vec::new());
    }

    pub fn print(&self) {
        dbg!("----");
        for (k, v) in &self.graph {
            dbg!(k.to_string(), v.len());
        }
        dbg!("----");
    }

    pub fn add_dependency(
        &mut self,
        type_name: &Ident<I>,
        inner_type: &'a Ident<I>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        let outgoing = self.graph.get_mut(type_name).unwrap();
        outgoing.push(inner_type);

        // Check if the refference creates a cyclic dependency
        if self.is_cyclic_directed_graph() {
            Err(Err::Failure(ParserError::new_at(
                inner_type,
                ParserErrorKind::CyclicReference,
            )))
        } else {
            Ok(())
        }
    }

    /// Cycle detection in a directed graph using DFS
    fn is_cyclic_directed_graph(&self) -> bool {
        // set is used to mark visited vertices
        let mut visited = HashSet::new();
        // set is used to keep track the ancestor vertices in recursive stack.
        let mut ancestors = HashSet::new();

        // call recur for all vertices
        for u in self.graph.keys() {
            // Don't recur for u if it is already visited
            if !visited.contains(u) && self.is_cyclic_recur(u, &mut visited, &mut ancestors) {
                return true;
            }
        }

        false
    }

    fn is_cyclic_recur(
        &self,
        current_vertex: &'a Ident<I>,
        visited: &mut HashSet<&'a Ident<I>>,
        ancestors: &mut HashSet<&'a Ident<I>>,
    ) -> bool {
        // mark it visited
        visited.insert(current_vertex);
        // add it to ancestor vertices
        ancestors.insert(current_vertex);

        // Recur for all the vertices adjacent to current_vertex
        for v in &self.graph[current_vertex] {
            // If the vertex is not visited then recurse on it
            if !visited.contains(v) {
                if self.is_cyclic_recur(v, visited, ancestors) {
                    return true;
                }
            } else if ancestors.contains(v) {
                // found a back edge, so there is a cycle
                return true;
            }
        }

        // remove from the ancestor vertices
        ancestors.remove(current_vertex);

        false
    }
}
