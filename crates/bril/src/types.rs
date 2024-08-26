use eyre::eyre;
use serde::Deserialize;
use std::str::FromStr;

/// A block of instruction in a function.
/// A block doesn't contain any control flow
/// instructions like `br` or `jmp`.
pub type Block = Vec<Instruction>;

/// A variable in the program
pub type Var = String;

/// The arguments to the operation
pub type Args = Vec<Var>;

#[derive(Debug, Deserialize)]
pub struct BrilProgram {
    pub functions: Vec<Function>,
}

#[derive(Debug, Deserialize)]
pub struct Function {
    pub name: String,
    pub instrs: Vec<Instruction>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Instruction {
    pub op: Operation,
    pub args: Option<Args>,
    pub r#type: Option<Type>,
    pub value: Option<u32>,
    pub dest: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    #[default]
    Const,
    Add,
    Mul,
    Print,
    Alloc,
    Free,
    Br,
    Jmp,
}

impl FromStr for Operation {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "const" => Ok(Operation::Const),
            "add" => Ok(Operation::Add),
            "mul" => Ok(Operation::Mul),
            "print" => Ok(Operation::Print),
            "alloc" => Ok(Operation::Alloc),
            "free" => Ok(Operation::Free),
            "br" => Ok(Operation::Br),
            "jmp" => Ok(Operation::Jmp),
            val => Err(eyre!("incorrect operation, got {val}")),
        }
    }
}

impl Operation {
    pub fn is_assignment(&self) -> bool {
        self == &Operation::Const
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Bool,
}

impl FromStr for Type {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(Type::Int),
            "bool" => Ok(Type::Bool),
            val => Err(eyre!("incorrect type, got {val}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BrilProgram;

    #[test]
    fn test_deserialize() {
        let s = r#"
            {
              "functions": [
                {
                  "instrs": [
                    {
                      "dest": "v0",
                      "op": "const",
                      "type": "int",
                      "value": 1
                    },
                    {
                      "dest": "v1",
                      "op": "const",
                      "type": "int",
                      "value": 2
                    },
                    {
                      "args": [
                        "v0",
                        "v1"
                      ],
                      "dest": "v2",
                      "op": "add",
                      "type": "int"
                    },
                    {
                      "args": [
                        "v2"
                      ],
                      "op": "print"
                    }
                  ],
                  "name": "main"
                }
              ]
            }
        "#;

        let program: BrilProgram = serde_json::from_str(s).unwrap();

        assert_eq!(program.functions.len(), 1);
    }
}
