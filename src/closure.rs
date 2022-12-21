use alloc::boxed::Box;

use crate::{
    atom::{Atom, List},
    vm::{Upvalue, UpvalueRef},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Closure<'a> {
    pub(crate) upvalues: Option<Box<[Upvalue<'a>]>>,
    pub(crate) code: List<'a>,
}

/// Make an [`UpvalueRef`] each [`Atom::Symbol`] that matches a an upvalue symbol.
fn upvalueize_symbols<'a>(code: &[Atom<'a>], upvalue_symbols: &[&'a str]) -> List<'a> {
    // Take each atom of the source, and replace each upvalue symbol or already defined upvalue to an UpvalueRef.
    code.iter()
        .map(|atom| match atom {
            Atom::Symbol(symb) => {
                // Check if the symbol of upvalue matches one in upvalue_symbols.
                if let Some((i, symb)) = upvalue_symbols
                    .iter()
                    .enumerate()
                    .find(|(_, upval)| *upval == symb)
                {
                    // Override symbol with an upvalue symbol
                    Atom::Upvalue(UpvalueRef(i, symb))
                } else {
                    atom.clone()
                }
            }
            Atom::List(list) => Atom::List(upvalueize_symbols(list, upvalue_symbols)),
            atom => atom.clone(),
        })
        .collect()
}

impl<'a> Closure<'a> {
    /// Build a [`Closure`] from a [`List`] code and a list of upvalue symbol.
    pub fn compile(code: List<'a>, upvalue_symbols: &[&'a str]) -> Self {
        // Shortcut for functions that compile_functionhave no upvalue.
        if upvalue_symbols.is_empty() {
            return Self::compile_thin(code);
        }

        Closure {
            // Consider upvalues as Symbol by default.
            upvalues: Some(
                upvalue_symbols
                    .iter()
                    .map(|symb| Atom::Symbol(symb))
                    .collect(),
            ),

            code: upvalueize_symbols(&code, upvalue_symbols),
        }
    }

    /// Create a thin [`Closure`] with no upvalue.
    pub fn compile_thin(code: List<'a>) -> Self {
        Closure {
            upvalues: None,
            code,
        }
    }

    /// Resolve an [`Atom`] transforming [`Atom::Upvalue`] references into their underlying [`Atom`].
    pub fn resolve(&self, atom: Atom<'a>) -> Atom<'a> {
        match atom {
            Atom::Upvalue(upvalue_ref) => self.resolve_ref(&upvalue_ref),
            _ => atom,
        }
    }

    pub fn resolve_ref(&self, upvalue_ref: &UpvalueRef<'a>) -> Atom<'a> {
        if let Some(upvalues) = &self.upvalues {
            if let Some(upvalue) = upvalues.get(upvalue_ref.0) {
                return upvalue.clone();
            }
        }

        Atom::Nil
    }
}
