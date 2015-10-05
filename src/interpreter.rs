use parser::Node;
use lexer::Str;
use std::collections::HashMap;
use std::cell::RefCell;

type Rib = HashMap<Str, Node>;

struct Envr {
    ribs: RefCell<Vec<Rib>>,
}

struct RibGuard<'a> {
    envr: &'a Envr,
}

impl<'a> Drop for RibGuard<'a> {
    fn drop(&mut self) {
        assert!(self.envr.ribs.borrow_mut().pop().is_some(), "No ribs to pop");
    }
}

impl Envr {
    fn new() -> Envr {
        Envr {
            ribs: RefCell::new(Vec::new()),
        }
    }

    fn with_value(name: &str, value: Node) -> Envr {
        let mut map = HashMap::new();
        map.insert(Str::new(name), value);
        Envr {
            ribs: RefCell::new(vec![map]),
        }
    }

    fn push_rib(&self) -> RibGuard {
        self.ribs.borrow_mut().push(HashMap::new());
        RibGuard {
            envr: self
        }
    }

    fn store(&self, name: &Str, value: Node) {
        let mut ribs = self.ribs.borrow_mut();
        assert!(ribs.len() > 0, "No ribs in environment");
        let len = ribs.len();
        let rib = &mut ribs[len - 1];
        assert!(!rib.contains_key(name), "Identifier already exists in rib: {}", name);
        rib.insert(name.clone(), value);
    }

    fn lookup(&self, name: &Str) -> Option<Node> {
        for rib in self.ribs.borrow().iter().rev() {
            if let Some(value) = rib.get(name) {
                return Some(value.clone());
            }
        }
        return None;
    }
}


pub fn run_program(input: &Node) -> Vec<Node> {
    let mut result = Vec::new();

    if let &Node::Program(ref ns) = input {
        for n in ns {
            let mut envr = Envr::new();
            result.push(run_node(n, &mut envr));
        }
    } else {
        panic!("Expected program, found: {:?}", input);
    }

    result
}

fn run_node(input: &Node, envr: &Envr) -> Node {
    match *input {
        ref t if t.is_value() => t.clone(),
        Node::S(ref ns) => {
            match ns[0] {
                Node::Print => {
                    let args = run_args(input, envr);
                    for a in &args {
                        println!("{}", a);
                    }
                    s!()
                }
                Node::Plus => {
                    let args = run_args(input, envr);
                    let result = args.iter().fold(0, |a, n| a + n.expect_lit_num());
                    lit_num!(result)
                }
                Node::Let => {
                    let len = ns.len();
                    let body = &ns[len - 1];
                    let args = &ns[1..len - 1];
                    assert!(args.len() % 2 == 0, "Argument without a value in `let`");
                    let _guard = envr.push_rib();
                    for i in 0..args.len() / 2 {
                        let arg_name = &args[i * 2].expect_ident();
                        let arg_value = run_node(&args[i * 2 + 1], envr);
                        envr.store(arg_name, arg_value);
                    }
                    run_node(body, envr)
                }
                Node::S(ref sub_ns) if sub_ns.len() > 0 && sub_ns[0] == Node::Fn => {
                    assert!(sub_ns.len() > 1, "No body for function: {:?}", &ns[0]);
                    let len = sub_ns.len();
                    let fun_body = &sub_ns[len - 1];
                    let formals: Vec<_> = sub_ns[1..len - 1].iter().map(|n| n.expect_ident()).collect();
                    let args = run_args(input, envr);
                    assert!(args.len() == formals.len(),
                            "Mismatch in number of function arguments. Expected: {}, found: {}",
                            formals.len(),
                            args.len());

                    let _guard = envr.push_rib();
                    for (ref formal, actual) in formals.iter().zip(args.iter()) {
                        envr.store(formal, actual.clone());
                    }
                    run_node(fun_body, envr)
                }
                ref n => {
                    let mut reduced_els = vec!(run_node(n, envr));
                    reduced_els.extend(ns[1..].iter().map(|n| n.clone()));
                    run_node(&Node::S(reduced_els), envr)
                }
            }
        }
        Node::Ident(ref s) => {
            if let Some(n) = envr.lookup(s) {
                return n;
            }
            panic!("Unknown identifier: {}", s);
        }
        _ => panic!("Unexpected node: {:?}", input),
    }
}

fn run_args(s: &Node, envr: &Envr) -> Vec<Node> {
    if let &Node::S(ref ns) = s {
        return ns[1..].iter().map(|n| run_node(n, envr)).collect();
    }

    panic!("Expcted S expression, found {:?}", s);
}

#[cfg(test)]
mod test {
    use super::{run_node, Envr};
    use super::*;
    use parser::Node;
    use lexer::Str;

    #[test]
    fn test_empty() {
        assert!(run_program(&program!()) == vec![]);
    }

    #[test]
    fn test_values() {
        assert!(run_program(&program!(lit_str!("foo"), lit_num!(42))) ==
                vec![lit_str!("foo"), lit_num!(42)]);
        let envr = &Envr::new();
        assert!(run_node(&lit_str!("foo"), envr) == lit_str!("foo"));
        let s = s!(Node::Fn, ident!("x"), s!(Node::Plus, ident!("x"), lit_num!(42)));
        assert!(run_node(&s, envr) == s);
        let s = s!();
        assert!(run_node(&s, envr) == s);
    }

    #[test]
    fn test_print() {
        let envr = &Envr::new();
        let s = s!(Node::Print, lit_str!("Hello world!"));
        assert!(run_node(&s, envr) == s!());
    }    

    #[test]
    fn test_plus() {
        let envr = &Envr::new();
        let s = s!(Node::Plus, lit_num!(3));
        assert!(run_node(&s, envr) == lit_num!(3));
        let s = s!(Node::Plus, lit_num!(3), lit_num!(1));
        assert!(run_node(&s, envr) == lit_num!(4));
        let s = s!(Node::Plus, lit_num!(3), lit_num!(1), lit_num!(1), lit_num!(1));
        assert!(run_node(&s, envr) == lit_num!(6));
    }    

    #[test]
    #[should_panic]
    fn test_plus_fail() {
        let envr = &Envr::new();
        let s = s!(Node::Plus, lit_num!(3), s!());
        run_node(&s, envr);
    }    

    #[test]
    fn test_ident() {
        let envr = &Envr::with_value("x", lit_num!(42));
        assert!(run_node(&ident!("x"), envr) == lit_num!(42));
    }

    #[test]
    #[should_panic]
    fn test_ident_fail() {
        let envr = &Envr::with_value("x", lit_num!(42));
        run_node(&ident!("y"), envr);
    }

    #[test]
    fn test_envr_push_pop() {
        let envr = &Envr::new();
        {
            let _guard = envr.push_rib();
            envr.store(&Str::new("x"), lit_num!(0));
            {
                let _guard = envr.push_rib();
                envr.store(&Str::new("x"), lit_num!(42));
                assert!(envr.lookup(&Str::new("x")) == Some(lit_num!(42)));
                assert!(envr.lookup(&Str::new("y")) == None);
            }
            assert!(envr.lookup(&Str::new("x")) == Some(lit_num!(0)));
            assert!(envr.lookup(&Str::new("y")) == None);
        }
        assert!(envr.lookup(&Str::new("x")) == None);
        assert!(envr.lookup(&Str::new("y")) == None);
    }

    #[test]
    #[should_panic]
    fn test_ident_dup_fail() {
        let envr = &Envr::with_value("x", lit_num!(42));
        envr.store(&Str::new("x"), lit_num!(42));
    }

    #[test]
    fn test_scoped_ident() {
        let envr = &Envr::with_value("x", lit_num!(0));
        let _guard = envr.push_rib();
        envr.store(&Str::new("x"), lit_num!(42));
        assert!(run_node(&ident!("x"), envr) == lit_num!(42));
    }

    #[test]
    fn test_let() {
        let envr = &Envr::new();
        // trivial
        assert!(run_node(&s!(Node::Let, s!()), envr) == s!());
        assert!(run_node(&s!(Node::Let, lit_num!(42)), envr) == lit_num!(42));
        // easy
        assert!(run_node(&s!(Node::Let, ident!("x"), lit_num!(42), ident!("x")), envr) ==
                lit_num!(42));
        assert!(run_node(&s!(Node::Let, ident!("x"), lit_num!(42),
                                        s!(Node::Plus, ident!("x"), lit_num!(42))), envr) ==
                lit_num!(84));
        // multiple
        assert!(run_node(&s!(Node::Let, ident!("x"), lit_num!(3),
                                        ident!("y"), lit_num!(4),
                                        s!(Node::Plus, ident!("x"), ident!("y"))), envr) ==
                lit_num!(7));
        // scoped
        assert!(run_node(&s!(Node::Let, ident!("x"), lit_num!(0),
                                        s!(Node::Let, ident!("x"), lit_num!(42),
                                                      ident!("x"))), envr) ==
                lit_num!(42));
        // uses earlier
        assert!(run_node(&s!(Node::Let, ident!("x"), lit_num!(3),
                                        ident!("y"), s!(Node::Plus, ident!("x"), lit_num!(1)),
                                        s!(Node::Plus, ident!("x"), ident!("y"))), envr) ==
                lit_num!(7));
    }

    #[test]
    #[should_panic]
    fn test_let_not_rec() {
        let envr = &Envr::new();
        run_node(&s!(Node::Let, ident!("x"), s!(Node::Plus, ident!("x"), lit_num!(0)), s!()), envr);
    }

    #[test]
    fn test_fn() {
        let envr = &Envr::new();
        // trivial
        assert!(run_node(&s!(s!(Node::Fn, s!())), envr) == s!());
        assert!(run_node(&s!(s!(Node::Fn, lit_num!(42))), envr) == lit_num!(42));
        // easy
        assert!(run_node(&s!(s!(Node::Fn, ident!("x"), ident!("x")), lit_num!(42)), envr) == lit_num!(42));
        assert!(run_node(&s!(s!(Node::Fn, ident!("x"), lit_num!(42)), lit_num!(0)), envr) == lit_num!(42));
        assert!(run_node(&s!(s!(Node::Fn, ident!("x"), s!(Node::Plus, ident!("x"), lit_num!(1))), lit_num!(42)), envr) == lit_num!(43));
        // multiple args
        assert!(run_node(&s!(s!(Node::Fn, ident!("x"), ident!("y"), ident!("x")), lit_num!(42), lit_num!(0)), envr) == lit_num!(42));
        assert!(run_node(&s!(s!(Node::Fn, ident!("x"), ident!("y"),
                                          s!(Node::Plus, ident!("x"), ident!("y"))),
                             lit_num!(42), lit_num!(1)), envr) == lit_num!(43));
        // scopes
        let f1 = s!(Node::Fn, ident!("x"), s!(Node::Plus, ident!("x"), lit_num!(1)));
        let f2 = s!(Node::Fn, ident!("x"), s!(f1, s!(Node::Plus, ident!("x"), lit_num!(4))));
        assert!(run_node(&s!(f2, lit_num!(2)), envr) == lit_num!(7));
        // higher order
        let f1 = s!(Node::Fn, ident!("x"), ident!("y"), s!(ident!("x"), s!(Node::Plus, ident!("y"), lit_num!(3))));
        let f2 = s!(Node::Fn, ident!("x"), s!(Node::Plus, ident!("x"), lit_num!(2)));
        assert!(run_node(&s!(f1, f2, lit_num!(5)), envr) == lit_num!(10));
    }

    #[test]
    fn test_fn_let() {
        let envr = &Envr::new();
        let f = s!(Node::Fn, ident!("x"), s!(Node::Plus, ident!("x"), lit_num!(1)));
        let l = s!(Node::Let, ident!("y"), f, s!(ident!("y"), lit_num!(42)));
        assert!(run_node(&l, envr) == lit_num!(43));
    }

    #[test]
    #[should_panic]
    fn test_fn_arg_mismatch() {
        let envr = &Envr::new();
        run_node(&s!(s!(Node::Fn, ident!("x"), ident!("x")), lit_num!(42), lit_num!(42)), envr);
    }
}
