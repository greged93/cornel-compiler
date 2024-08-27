use crate::{all_none, all_some};
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
    pub args: Args,
    pub r#type: Option<Type>,
    pub value: Option<u32>,
    pub dest: Option<String>,
}

impl Instruction {
    /// Verifies if the instruction is a valid instruction
    pub fn is_valid(&self) -> bool {
        let count_args = self.args.len();
        let no_args = self.args.is_empty();
        let one_args = count_args == 1;
        let two_args = count_args == 2;
        let three_args = count_args == 3;
        match self.op {
            Operation::Const => {
                all_some!(self.value, self.dest) && all_none!(self.r#type) && no_args
            }
            Operation::Add => {
                all_some!(self.dest) && all_none!(self.value, self.r#type) && two_args
            }
            Operation::Mul => {
                all_some!(self.dest) && all_none!(self.value, self.r#type) && two_args
            }
            Operation::Id => all_some!(self.dest) && all_none!(self.value, self.r#type) && one_args,
            Operation::Print => all_none!(self.value, self.r#type, self.dest) && one_args,
            Operation::Br => all_none!(self.r#type, self.value, self.dest) && three_args,
            Operation::Jmp => all_none!(self.value, self.r#type, self.dest),
        }
    }

    /// Returns true if the instruction is a assignment (const operation)
    pub fn is_assignment(&self) -> bool {
        self.op == Operation::Const
    }
}

#[derive(Debug, Default, Hash, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    #[default]
    Const,
    Add,
    Mul,
    Id,
    Print,
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
            "id" => Ok(Operation::Id),
            "print" => Ok(Operation::Print),
            "br" => Ok(Operation::Br),
            "jmp" => Ok(Operation::Jmp),
            val => Err(eyre!("incorrect operation, got {val}")),
        }
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
                      "value": 1,
                      "args": []
                    },
                    {
                      "dest": "v1",
                      "op": "const",
                      "type": "int",
                      "value": 2,
                      "args": []
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
