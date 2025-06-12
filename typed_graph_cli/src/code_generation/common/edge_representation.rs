use std::fmt::Display;

use build_script_lang::schema::{LowerBound, Quantifier};
use std::fmt::Write;

use crate::GenResult;

/// Sorted by most to least restrictive  
/// max correspond to the most inclusive representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeRepresentation {
    Result,
    Option,
    Iterator,
}

impl EdgeRepresentation {
    pub fn from_quantity<I>(quantity: &Quantifier<I>) -> EdgeRepresentation {
        EdgeRepresentation::from_bounds(&quantity.bounds)
    }
    
    pub fn from_bounds(bounds: &Option<(LowerBound, u32)>) -> EdgeRepresentation {
        match &bounds {
            Some((lower, upper)) => {
                if lower == &LowerBound::One && upper == &1 {
                    EdgeRepresentation::Result
                } else if upper == &1 {
                    EdgeRepresentation::Option
                } else {
                    EdgeRepresentation::Iterator
                }
            }
            None => EdgeRepresentation::Iterator,
        }
    }

    /// Which type is the return type
    pub fn get_return_type_rust(&self, edge_type: impl Display, node_type: impl Display, schema_name: impl Display) -> String {
        match self {
            EdgeRepresentation::Result => {
                format!("SchemaResult<({edge_type}, {node_type}), NK, EK, {schema_name}<NK, EK>>")
            }
            EdgeRepresentation::Option => {
                format!("SchemaResult<Option<({edge_type}, {node_type})>, NK, EK, {schema_name}<NK, EK>>")
            }
            EdgeRepresentation::Iterator => {
                format!("SchemaResult<impl Iterator<Item = ({edge_type}, {node_type})> + 'a, NK, EK, {schema_name}<NK, EK>>")
            }
        }
    }

    /// Given an iterator of (Edge, Node) collect them into the representation
    pub fn collect_results_rust(&self, edge_type: impl Display, s: &mut String) -> GenResult<()> {
        match self {
            EdgeRepresentation::Iterator => (),
            EdgeRepresentation::Option => writeln!(s, "           .next()")?,
            EdgeRepresentation::Result => {
                writeln!(s, "           .next()")?;
                writeln!(s, "           .ok_or_else(|| TypedError::InvalidLowerBound(self.get_id(), self.get_type(), \"[{edge_type}]\".to_string()))?")?;
            }
        };

        Ok(())
    }

    pub fn get_return_type_python(&self, edge_type: impl Display, node_type: impl Display) -> String {
        match self {
            EdgeRepresentation::Result => {
                format!("Tuple['{edge_type}', '{node_type}']")
            }
            EdgeRepresentation::Option => {
                format!("Optional[Tuple['{edge_type}', '{node_type}']]")
            }
            EdgeRepresentation::Iterator => {
                format!("Iterator[Tuple['{edge_type}', '{node_type}']]")
            }
        }
    }

    pub fn collect_results_python(&self, edge_type: impl Display, s: &mut String) -> GenResult<()> {
        match self {
            EdgeRepresentation::Iterator => (),
            EdgeRepresentation::Option => writeln!(s, "        nodes = next(nodes, None)")?,
            EdgeRepresentation::Result => {
                writeln!(s, "        nodes = next(nodes, None)")?;
                writeln!(s, "        if nodes is None:")?;
                writeln!(s, "            raise RecievedNoneValue(f'{{self.get_id()}}({{self.get_type()}})', '{edge_type}')")?;
            }
        }

        Ok(())
    }
}