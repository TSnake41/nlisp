use alloc::boxed::Box;

use crate::{
    atom::{Atom, List},
    closure::Closure,
    vm::{NlispVm, VmError},
};

/// Resolve each upvalues.
fn resolve_upvalues<'a>(context: &Closure<'a>, list: &[Atom<'a>], recursively: bool) -> List<'a> {
    list.iter()
        .map(|atom| match atom {
            Atom::List(sublist) if recursively => {
                Atom::List(resolve_upvalues(context, sublist, true))
            }
            atom => context.resolve(atom.clone()),
        })
        .collect()
}

/// Resolve each atom of the paramters :
///  - resolve upvalues using current context
///  - resolve symbols using vm globals
fn resolve_classic<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
    evaluate_each: bool,
) -> List<'a> {
    param
        .iter()
        .map(|atom| match atom {
            Atom::List(list) if evaluate_each => {
                let list_resolved = resolve_classic(vm, context, list, true);
                vm.evaluate(context, &list_resolved)
                    .unwrap_or_else(|err| Atom::Error(err))
            }

            Atom::Upvalue(upvalue_ref) => context.resolve_ref(upvalue_ref),

            Atom::Symbol(symbol) => vm.resolve(symbol).unwrap_or_else(|| atom.clone()),
            atom => atom.clone(),
        })
        .collect()
}

/// Resolve or evaluate the symbol, depending on its type.
///  - if it is a list, evaluate the list
///  - if it is a symbol/upvalue, resolve the atom
///  - if it is something else, return it as-is
fn evaluate_atom<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    atom: &Atom<'a>,
) -> Result<Atom<'a>, VmError> {
    match atom {
        // Evaluate the passed list.
        Atom::List(list) => {
            let list_resolved = resolve_classic(vm, context, list, false);
            vm.evaluate(context, &list_resolved)
        }

        // Resolve the symbol.
        Atom::Symbol(symb) => Ok(vm.resolve(symb).unwrap_or(Atom::Symbol(symb))),

        // Resolve the upvalue.
        Atom::Upvalue(upvalue_ref) => Ok(context.resolve_ref(upvalue_ref)),

        // We don't need to do anything on it.
        atom => Ok(atom.clone()),
    }
}

pub fn if_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    // Need the first parameter.
    let cond_atom = match param.get(0) {
        Some(atom) => atom,
        None => return Err(VmError::InvalidUsage),
    };

    // Atom::Bool(false) and Atom::Nil are falsy, everything else is truthful.
    let cond_result = match evaluate_atom(vm, context, cond_atom)? {
        Atom::Bool(false) | Atom::Nil => false,
        _ => true,
    };

    let branch = if cond_result {
        param.get(1)
    } else {
        param.get(2)
    };

    // Execute branch (if exists)
    match branch {
        Some(branch) => evaluate_atom(vm, context, branch),
        None => Ok(Atom::Nil),
    }
}

pub fn printd_function<'a>(
    _: &mut NlispVm,
    _: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    println!("{param:#?}");

    Ok(Atom::Nil)
}

pub fn print_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    println!("{:#?}", resolve_classic(vm, context, param, true));

    Ok(Atom::Nil)
}

/// ```lisp
/// (quote ...)
/// ```
///
/// Returns its parameters as a [Atom::List] without resolving symbols and upvalues.
pub fn quote_function<'a>(
    _: &mut NlispVm<'a>,
    _: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    Ok(Atom::List(param.iter().cloned().collect()))
}

/// ```lisp
/// (lambda (upvalues...)
///     (source...))
/// ```
///
/// Create a new [Atom::Closure] with an upvalue list and a specified source.
pub fn lambda_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    let param = resolve_classic(vm, context, param, false);

    let Some(Atom::List(upvalues)) = param.get(0) else { return Err(VmError::InvalidUsage) };
    let Some(Atom::List(source)) = param.get(1) else { return Err(VmError::InvalidUsage) };

    // Check if all upvalues are symbols.
    if upvalues.iter().any(|atom| !matches!(atom, Atom::Symbol(_))) {
        // There is an object that is not a symbol.
        return Err(VmError::InvalidUsage);
    }

    // Build the list of upvalues.
    let upvalue_symbols: Box<[&'a str]> = upvalues
        .iter()
        .map(|atom| match atom {
            Atom::Symbol(symb) => *symb,
            _ => "(nil)",
        })
        .collect();

    Ok(Atom::Closure(Closure::compile(
        resolve_upvalues(context, source, true),
        &upvalue_symbols,
    )))
}

/// ```lisp
/// (eval
///     (expr1)
///     (expr2)
///     ...
///     (exprN))
/// ```
/// Evaluate each expression and return an [Atom::List] with each expression result.
pub fn eval_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    // Check if all parameters are lists.
    if param.iter().any(|atom| !matches!(atom, Atom::List(_))) {
        return Err(VmError::InvalidUsage);
    }

    Ok(Atom::List(
        param
            .iter()
            .map(|atom| match atom {
                // Evaluate each lists.
                Atom::List(list) => vm.evaluate(context, list),
                _ => Err(VmError::InvalidUsage),
            })
            .map(|res| match res {
                // Transform errors into Atom::Error
                Ok(atom) => atom,
                Err(vm_error) => Atom::Error(vm_error),
            })
            .collect(),
    ))
}

/// ```lisp
/// (type
///     val1
///     val2
///     ...
///     valN)
/// ```
/// Create an [Atom::List] that contains each value type as a [Atom::String].
pub fn type_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    Ok(Atom::List(
        resolve_classic(vm, context, param, false)
            .iter()
            .map(|atom| Atom::String(atom.get_type_str()))
            .collect(),
    ))
}

/// ```lisp
/// (global symbol value)
/// ```
/// Create or replace the global `symbol` with the value computed from `value`.
pub fn global_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    // Check and resolve if needed the symbol atom.
    let Some(symbol) = (match param.get(0) {
        // A symbol atom stays as is.
        Some(Atom::Symbol(symb)) => Some(*symb),

        // Resolve the upvalue into its symbol.
        Some(Atom::Upvalue(upvalue_ref)) => match context.resolve_ref(upvalue_ref) {
            Atom::Symbol(symb) => Some(symb),
            _ => None
        }

        _ => None
    }) else {
        return Err(VmError::NotASymbol);
    };

    let Some(atom) = param.get(1) else { return Err(VmError::InvalidUsage) };

    let result = evaluate_atom(vm, context, atom);

    match result {
        Ok(value) => {
            vm.add_symbol(symbol, value);
            Ok(Atom::Nil)
        }
        Err(e) => Err(e),
    }
}

pub fn resolve_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    Ok(Atom::List(resolve_classic(vm, context, param, false)))
}

/// ```lisp
/// (neg num)
/// ```
///
/// Return the opposite of its parameter if it is a [Atom::Number].
/// If no parameter is given, returns [Atom::Nil].
pub fn neg_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    let param = evaluate_atom(vm, context, param.get(0).unwrap_or(&Atom::Nil));

    match param {
        Ok(atom) => match atom {
            Atom::Number(n) => Ok(Atom::Number(-n)),
            atom => Ok(atom),
        },
        Err(err) => Err(err),
    }
}

/// ```lisp
/// (+ num1 num2 ... numN)
/// ```
///
/// Return the sum of its [Atom::Number] parameters.
pub fn sum_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    Ok(Atom::Number(
        resolve_classic(vm, context, param, true)
            .iter()
            .map(|atom| match atom {
                Atom::Number(n) => *n,
                _ => 0f32,
            })
            .fold(0f32, |a, b| a + b),
    ))
}

/// ```lisp
/// (* num1 num2 ... numN)
/// ```
///
/// Return the product of its [Atom::Number] parameters.
pub fn product_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    Ok(Atom::Number(
        resolve_classic(vm, context, param, true)
            .iter()
            .map(|atom| {
                if let Atom::List(list) = atom {
                    vm.evaluate(context, list).unwrap_or(Atom::Nil)
                } else {
                    atom.clone()
                }
            })
            .map(|atom| match atom {
                Atom::Number(n) => n,
                _ => 0f32,
            })
            .fold(0f32, |a, b| a * b),
    ))
}

/// ```lisp
/// (= param1 param2 ... paramN)
/// ```
///
/// Return an [Atom::Bool] that indicates whether all params are the same.
/// If no parameter is given, returns true.
/// If an error occurs in
pub fn eq_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    let param = resolve_classic(vm, context, param, true);

    let mut iter = param.iter();
    let Some(first) = iter.next() else { /* no value */ return Ok(Atom::Bool(true)) };

    for elem in iter {
        if first != elem {
            return Ok(Atom::Bool(false));
        }
    }

    Ok(Atom::Bool(true))
}

/// ```lisp
/// (map func list)
/// ```
///
/// Apply func to each element of list
pub fn map_function<'a>(
    vm: &mut NlispVm<'a>,
    context: &mut Closure<'a>,
    param: &[Atom<'a>],
) -> Result<Atom<'a>, VmError> {
    Ok(Atom::Nil)
}
