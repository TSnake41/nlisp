use core::{num::ParseFloatError, str::FromStr};
use alloc::vec::Vec;

use crate::atom::{Atom, List};

#[derive(Debug)]
pub enum ParseError {
    InvalidCharacter(usize),
    NumberError(ParseFloatError, usize),
    IncompleteString,
    IncompleteList,
}

enum ReadingState {
    /// Looking the read the next atom.
    None,

    /// Looking for a space character
    Symbol(usize),
    /// Looking for a space character
    Number(usize),
    /// Looking for an end of string "
    String(usize),

    /// Looking for a matching end parenthesis.
    List {
        /// The first character of the list we are reading.
        start: usize,

        /// The nesting level, the amount of non-closed parenthesis we actually consider.
        depth: usize,

        /// Whether we are in a string.
        in_string: bool,
    },
}

/// Parse a list from an input string.
pub fn parse(input: &str) -> Result<List<'_>, ParseError> {
    let mut atoms: Vec<Atom> = alloc::vec![];

    let iterator = input.chars().enumerate();
    let mut state = ReadingState::None;

    for (pos, c) in iterator {
        state = match state {
            // Symbol start: Alphabetic
            ReadingState::None
                if c.is_alphabetic() || (c.is_ascii_punctuation() && c != '(' && c != ')') =>
            {
                ReadingState::Symbol(pos)
            }

            // Number start: numeric or .
            ReadingState::None if c.is_numeric() || c == '.' => ReadingState::Number(pos),

            // List start: (
            ReadingState::None if c == '(' => ReadingState::List {
                start: pos,
                depth: 0,
                in_string: false,
            },

            // String start : '"'
            ReadingState::None if c == '"' => ReadingState::String(pos),

            // Whitespace
            ReadingState::None if c.is_whitespace() => ReadingState::None,

            // Something else unexpected
            ReadingState::None => return Err(ParseError::InvalidCharacter(pos)),

            // Symbol handling
            ReadingState::Symbol(start)
                if c.is_alphanumeric() || (c.is_ascii_punctuation() && c != '(' && c != ')') =>
            {
                ReadingState::Symbol(start)
            }

            ReadingState::Symbol(start) if c.is_whitespace() => {
                atoms.push(Atom::Symbol(&input[start..pos]));

                ReadingState::None
            }

            // Unexpected character
            ReadingState::Symbol(_) => return Err(ParseError::InvalidCharacter(pos)),

            // Reading number
            ReadingState::Number(start) if c.is_numeric() || c == '.' => {
                ReadingState::Number(start)
            }

            ReadingState::Number(start) if c.is_whitespace() => {
                let val = match f32::from_str(&input[start..pos]) {
                    Ok(v) => v,
                    Err(e) => return Err(ParseError::NumberError(e, pos)),
                };

                atoms.push(Atom::Number(val));

                ReadingState::None
            }

            ReadingState::Number(_) => return Err(ParseError::InvalidCharacter(pos)),

            ReadingState::String(start) if c == '"' => {
                atoms.push(Atom::String(&input[(start + 1)..pos]));

                ReadingState::None
            }

            ReadingState::String(start) => ReadingState::String(start),

            // List end
            ReadingState::List {
                start,
                depth,
                in_string,
            } if !in_string && depth == 0 && c == ')' => {
                let list = match parse(&input[(start + 1)..pos]) {
                    Ok(list) => list,
                    Err(e) => return Err(e),
                };

                atoms.push(Atom::List(list));

                ReadingState::None
            }

            // List parenthesis openning
            ReadingState::List {
                start,
                depth,
                in_string,
            } if !in_string && c == '(' => ReadingState::List {
                start,
                depth: depth + 1,
                in_string,
            },

            ReadingState::List {
                start,
                depth,
                in_string,
            } if !in_string && c == ')' => ReadingState::List {
                start,
                depth: depth - 1,
                in_string,
            },

            ReadingState::List {
                start,
                depth,
                in_string,
            } if c == '"' => ReadingState::List {
                start,
                depth,
                in_string: !in_string,
            },

            ReadingState::List {
                start,
                depth,
                in_string,
            } => ReadingState::List {
                start,
                depth,
                in_string,
            },
        }
    }

    // Parse the latest symbol, if possible.
    let pos = input.len();

    match state {
        ReadingState::Symbol(start) => {
            atoms.push(Atom::Symbol(&input[start..]));
        }
        ReadingState::Number(start) => {
            let val = match f32::from_str(&input[start..]) {
                Ok(v) => v,
                Err(e) => return Err(ParseError::NumberError(e, pos)),
            };

            atoms.push(Atom::Number(val));
        }
        ReadingState::String(_) => return Err(ParseError::IncompleteString),
        ReadingState::List {
            start: _,
            depth: _,
            in_string: _,
        } => return Err(ParseError::IncompleteList),

        ReadingState::None => (),
    };

    Ok(atoms.into_boxed_slice())
}
