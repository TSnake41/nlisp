use alloc::collections::BTreeMap;

use crate::{
    atom::{Atom, List},
    closure::Closure,
    primitives,
};

/// Upper value (e.g parameter).
pub type Upvalue<'a> = Atom<'a>;

/// Reference to an upvalue.
#[derive(Debug, Clone, PartialEq)]
pub struct UpvalueRef<'a>(pub(crate) usize, pub(crate) &'a str);

pub type NativeFunction<'a> =
    &'a dyn Fn(&mut NlispVm<'a>, &mut Closure<'a>, &[Atom<'a>]) -> Result<Atom<'a>, VmError>;

pub struct NlispVm<'a> {
    /// A scope, basically a list of symbols, and a parent scope (if any).
    symbol_map: BTreeMap<&'a str, Atom<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
    NonEvaluable,
    NotAFunction,
    InvalidUsage,
    NotASymbol,
}

impl<'a> NlispVm<'a> {
    pub fn new() -> Self {
        let mut symbol_map = BTreeMap::new();

        symbol_map.insert("pi", Atom::Number(3.14159265));
        symbol_map.insert("true", Atom::Bool(true));
        symbol_map.insert("false", Atom::Bool(false));

        symbol_map.insert("print", Atom::NativeFunction(&primitives::print_function));
        symbol_map.insert("printd", Atom::NativeFunction(&primitives::printd_function));
        symbol_map.insert("if", Atom::NativeFunction(&primitives::if_function));
        symbol_map.insert("lambda", Atom::NativeFunction(&primitives::lambda_function));
        symbol_map.insert("quote", Atom::NativeFunction(&primitives::quote_function));
        symbol_map.insert("type", Atom::NativeFunction(&primitives::type_function));
        symbol_map.insert("global", Atom::NativeFunction(&primitives::global_function));
        symbol_map.insert(
            "resolve",
            Atom::NativeFunction(&primitives::resolve_function),
        );
        symbol_map.insert("eval", Atom::NativeFunction(&primitives::eval_function));

        symbol_map.insert("+", Atom::NativeFunction(&primitives::sum_function));
        symbol_map.insert("*", Atom::NativeFunction(&primitives::product_function));
        symbol_map.insert("=", Atom::NativeFunction(&primitives::eq_function));
        symbol_map.insert("neg", Atom::NativeFunction(&primitives::neg_function));

        NlispVm { symbol_map }
    }

    pub fn evaluate(
        &mut self,
        context: &mut Closure<'a>,
        list: &List<'a>,
    ) -> Result<Atom<'a>, VmError> {
        if let Some((first, param)) = list.clone().split_first_mut() {
            // Resolve symbol for first if needed.
            if let Atom::Symbol(symb) = first {
                if let Some(atom) = self.resolve(symb) {
                    *first = atom;
                }
            }

            match first {
                Atom::Closure(closure) => {
                    // Replace upvalues with parameters.
                    if let Some(upvalues) = &mut closure.upvalues {
                        upvalues.iter_mut().enumerate().for_each(|(i, upvalue)| {
                            if let Some(atom) = param.get(i) {
                                *upvalue = atom.clone()
                            }
                        });
                    }

                    self.evaluate(closure, &closure.code.clone())
                }
                Atom::NativeFunction(func) => func(self, context, param),
                _ => Err(VmError::NotAFunction),
            }
        } else {
            Err(VmError::NonEvaluable)
        }
    }

    pub fn add_symbol(&mut self, name: &'a str, value: Atom<'a>) {
        self.symbol_map.insert(name, value);
    }

    pub fn resolve(&self, symbol: &str) -> Option<Atom<'a>> {
        self.symbol_map.get(symbol).cloned()
    }
}

impl<'a> Default for NlispVm<'a> {
    fn default() -> Self {
        Self::new()
    }
}
