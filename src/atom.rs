use alloc::boxed::Box;
use crate::{vm::{VmError, self}, closure};


#[derive(Clone)]
pub enum Atom<'a> {
    Symbol(&'a str),
    Number(f32),
    String(&'a str),
    List(List<'a>),

    // Internal atoms
    Bool(bool),
    Nil,
    Error(VmError),
    Upvalue(vm::UpvalueRef<'a>),
    Closure(closure::Closure<'a>),
    NativeFunction(vm::NativeFunction<'a>),
}

pub type List<'a> = Box<[Atom<'a>]>;

impl<'a> PartialEq for Atom<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Error(l0), Self::Error(r0)) => l0 == r0,
            (Self::Upvalue(l0), Self::Upvalue(r0)) => l0 == r0,
            (Self::Closure(l0), Self::Closure(r0)) => l0 == r0,
            (Self::NativeFunction(_), Self::NativeFunction(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<'a> Atom<'a> {
    pub fn get_type_str(&self) -> &'static str {
        match self {
            Atom::Symbol(_) => "Symbol",
            Atom::Number(_) => "Number",
            Atom::String(_) => "String",
            Atom::List(_) => "List",
            Atom::Bool(_) => "Bool",
            Atom::Nil => "Nil",
            Atom::Upvalue(_) => "Upvalue",
            Atom::Closure(_) => "Closure",
            Atom::NativeFunction(_) => "NativeFunction",
            Atom::Error(err_type) => match err_type {
                VmError::NonEvaluable => "Error:NonEvaluable",
                VmError::NotAFunction => "Error:NotAFunction",
                VmError::InvalidUsage => "Error:InvalidUsage",
                VmError::NotASymbol => "Error:NotASymbol",
            },
        }
    }
}

impl<'a> core::fmt::Debug for Atom<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Nil => f.debug_tuple("Nil").finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::Upvalue(arg0) => f.debug_tuple("Upvalue").field(arg0).finish(),
            Self::Closure(arg0) => f.debug_tuple("Closure").field(arg0).finish(),
            Self::NativeFunction(_) => f.debug_tuple("NativeFunction").finish(),
            Self::Error(arg0) => f.debug_tuple("Error").field(arg0).finish(),
        }
    }
}
