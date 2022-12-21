//#![no_std]
extern crate alloc;

pub mod atom;
pub mod closure;
pub mod parser;
pub(crate) mod primitives;
pub mod vm;

use atom::{Atom, List};

fn main() {
    let code = r#"
    (global fn
        (lambda (name args definition)
            (global name (lambda args definition))))
        
    (fn - (a b)
        (+ a (neg b)))
    
    (fn fib (n fib)
        (if (= n 0)
            0
        (if (= n 1)
            1
        (+ (fib (- n 1)) (fib (- n 2))))))

    (fib 25 fib)
    "#;

    let list = parser::parse(code).unwrap();

    let mut vm = vm::NlispVm::new();

    let mut root_context = closure::Closure::compile_thin([].into());

    list.iter().for_each(|atom| match atom {
        Atom::List(l) => match vm.evaluate(&mut root_context, l) {
            Ok(a) => println!("{a:?}"),
            Err(err) => eprintln!("{err:?}"),
        },
        atom => println!("{atom:?}"),
    });

    //println!("{:?}", vm.evaluate(&mut closure, &mut code));
    //println!("{f:?}");
}
