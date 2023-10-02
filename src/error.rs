use crate::lexer::TokenType;
use crate::node::{Dtype, Node, NodeVariant};
use colored::Colorize;

#[derive(Debug)]
pub enum ErrorType<'a> {
    /// Token
    UnrecognizedToken(char),
    /// Received, expected
    UnexpectedToken(TokenType, TokenType),
    /// Vardef name
    VardefNoExpression(&'a str),
    /// Struct name, member
    NonexistentStructMember(&'a str, &'a str),
    /// To be used for str -> dtype.
    /// Dtype name
    InvalidDtypeFromStr(&'a str),
    /// Function name, arg count, param count
    FunctionArgParamMismatch(&'a str, usize, usize),
    /// Dest, src
    AssignTypeMismatch(Dtype, Dtype),
    /// Struct member variable must be an identifier.
    /// Received
    StructMemberVarNonId(&'a Node),
    /// Non-struct types don't have member variables
    /// Member access parent type
    PrimitiveMemberAccess(Dtype),
    /// Function name
    FunctionDeclDefMismatch(&'a str),
    /// Function name
    DuplicateFdef(&'a str),
    /// Struct name
    DuplicateSdef(&'a str),
    /// Function name
    NonexistentFunction(&'a str),
    /// Struct name
    NonexistentStruct(&'a str),
    /// Variable name
    NonexistentVariable(&'a str),
    /// Node variant of addressof target
    InvalidAddressof(&'a NodeVariant),
    /// Data type of deref target
    InvalidDeref(&'a NodeVariant),
}

impl<'a> ErrorType<'a> {
    pub fn message(&self) -> String {
        match self {
            ErrorType::UnrecognizedToken(tok) => format!("Unrecognized token '{}'.", tok),
            ErrorType::UnexpectedToken(recv, expect) => {
                format!("Expected {:?}, received {:?}.", expect, recv)
            }
            ErrorType::VardefNoExpression(name) => {
                format!("Definition of variable '{}' has no expression.", name)
            }
            ErrorType::NonexistentStructMember(sname, member) => {
                format!("Struct '{}' has no member '{}'.", sname, member)
            }
            ErrorType::InvalidDtypeFromStr(dtype) => {
                format!("'{}' is not a valid data type.", dtype)
            }
            ErrorType::FunctionArgParamMismatch(name, nargs, nparams) => format!(
                "Function '{}' takes in {} parameters but was passed {} arguments.",
                name, nparams, nargs
            ),
            ErrorType::AssignTypeMismatch(dest, src) => {
                format!("Attempting to assign type '{}' to type '{}'.", src, dest)
            }
            ErrorType::StructMemberVarNonId(node) => format!(
                "Struct member access must be an identifier; received '{:?}'.",
                node
            ),
            ErrorType::PrimitiveMemberAccess(stype) => format!(
                "Attempting to access member variable of non-struct type '{}'.",
                stype
            ),
            ErrorType::FunctionDeclDefMismatch(name) => format!(
                "Function declaration and definition of '{}' do not align.",
                name
            ),
            ErrorType::DuplicateFdef(name) => {
                format!("Duplicate definition of function '{}'.", name)
            }
            ErrorType::DuplicateSdef(name) => format!("Duplicate definition of struct '{}'.", name),
            ErrorType::NonexistentFunction(name) => format!("Function '{}' does not exist.", name),
            ErrorType::NonexistentStruct(name) => format!("Struct '{}' does not exist.", name),
            ErrorType::NonexistentVariable(name) => format!("Variable '{}' does not exist.", name),
            ErrorType::InvalidAddressof(dtype) => format!("Can't take address of '{:?}'.", dtype),
            ErrorType::InvalidDeref(dtype) => format!("Can't dereference '{:?}'.", dtype),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    message: String,
    line: usize,
}

impl Error {
    pub fn new(etype: ErrorType, line: usize) -> Self {
        Self {
            message: etype.message(),
            line,
        }
    }

    pub fn print(&self, prog: &str) {
        let split: Vec<&str> = prog.split('\n').collect();
        println!(
            "{}: Line {}: {}",
            "error".bright_red(),
            self.line,
            self.message
        );
        let longest: usize = *[if self.line == 0 { 0 } else { self.line - 1 }, self.line, self.line + 1]
            .map(|x| x.to_string().len())
            .iter()
            .max()
            .unwrap();
        for i in -1i32..=1 {
            if (self.line as i32 + i - 1) < 0 || (i + self.line as i32) < 0 || (i + self.line as i32) >= split.len() as i32 {
                continue;
            }

            let padding: usize = longest - (self.line as i32 + i).to_string().len();
            let format: String = format!(
                "  {}{} | {}",
                self.line as i32 + i,
                " ".repeat(padding),
                split[(self.line as i32 + i - 1) as usize]
            );
            println!(
                "{}",
                if i == 0 {
                    format.white().bold()
                } else {
                    format.truecolor(150, 150, 150)
                }
            );
        }
    }
}
