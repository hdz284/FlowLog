/*
    DataType: number | string
    Attribute: <name>: <DataType>
    RelDecl: <name>(<Attribute>, <Attribute>, ...)
*/

use crate::parser::Lexeme;
use crate::Rule;
use pest::iterators::Pair;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DataType {
    Integer,
    String,
}

impl DataType {
    fn from_str(type_str: &str) -> Self {
        match type_str {
            "number" => Self::Integer,
            "string" => Self::String,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer => write!(f, "number"),   // f :: a formatter that can be used to write to a buffer
            Self::String => write!(f, "string"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    name: String,
    data_type: DataType,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.data_type)
    }
}

impl Attribute {
    fn from_str(name: &str, data_type: &str) -> Self {
        Self {
            name: name.to_string(),
            data_type: DataType::from_str(data_type),
        }
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }
}

#[derive(Debug, Clone)]
pub struct RelDecl {
    name: String,
    attributes: Vec<Attribute>,
    path: Option<String>,
}

impl fmt::Display for RelDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.attributes
                .iter()
                .map(|attr| attr.to_string()) // to_string() uses the Display impl for Attribute
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        if let Some(ref path) = self.path {
            write!(f, " read as {}", path)?;
        }
        Ok(())
    }
}


impl RelDecl {
    fn from_str(name: &str, attributes: Vec<Attribute>, path: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            attributes,
            path: path.map(|p| p.to_string()), 
        }
    }

    pub fn push_attr(&mut self, attr: Attribute) {
        self.attributes.push(attr);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    pub fn arity(&self) -> usize {
        self.attributes.len()
    }

    pub fn path(&self) -> Option<String> {
        self.path.clone()
    }
}

impl Lexeme for RelDecl {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let mut parsed_rule = parsed_rule.into_inner(); // into_inner() returns an iterator over the inner Pairs of a Pair
        /* parsing the relation name */
        let name = parsed_rule.next().unwrap().as_str(); // as_str() returns the original string of the input

        // debug!(".decl name = {:?}", name);
        // debug!("RelDecl attributes = {:?}", parsed_rule);

        /* parsing the relation attributes */
        let attributes = parsed_rule
            .next()
            .unwrap()
            .into_inner()
            .map(|attr| {
                // debug!(".decl attribute = {:?}", attr);
                let mut attr = attr.into_inner();
                let name = attr.next().unwrap().as_str();
                let data_type = attr.next().unwrap().as_str();
                Attribute::from_str(name, data_type)
            })
            .collect();

        // if parsed_rule has next, then a path is provided    
        let path = if let Some(path) = parsed_rule.next() {
            Some(path.into_inner().next().unwrap().as_str())
        } else {
            None
        };

        Self::from_str(name, attributes, path)
    }
}
