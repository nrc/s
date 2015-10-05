use parser::Node;
use lexer::Str;
use std::collections::HashMap;

pub fn fold(node: Node, fld: &mut Folder) -> Node {
    match node {
        Node::Program(ns) => Node::Program(ns.into_iter().map(|n| fold(n, fld)).collect()),
        Node::S(ns) => {
            match &ns[0] {
                &Node::Macro => {
                    fld.fold_macro(ns)
                }
                &Node::Ident(_) => {
                    fld.fold_ident(ns)
                }
                _ => {
                    Node::S(ns.into_iter().map(|n| fold(n, fld)).collect())
                }
            }
        }
        Node::Ident(_) |
        Node::Plus |
        Node::Fn |
        Node::Let |
        Node::Print |
        Node::LitNum(_) |
        Node::LitStr(_) => node,
        Node::Macro => panic!("noop fold of macro"),
    }
}

pub trait Folder {
    // Fold (id ...) ns includes id
    fn fold_ident(&mut self, ns: Vec<Node>) -> Node;
    // Fold (macro ...) ns includes macro
    fn fold_macro(&mut self, ns: Vec<Node>) -> Node;
}

pub struct NoopFolder;

impl Folder for NoopFolder {
    fn fold_ident(&mut self, ns: Vec<Node>) -> Node {
        Node::S(ns.into_iter().map(|n| fold(n, self)).collect())
    }    

    fn fold_macro(&mut self, ns: Vec<Node>) -> Node {
        Node::S(ns.into_iter().map(|n| fold(n, self)).collect())
    }    
}

pub struct Unhygienic {
    macros: HashMap<Str, (Vec<Str>, Node)>,
}

impl Unhygienic {
    pub fn new() -> Unhygienic {
        Unhygienic {
            macros: HashMap::new(),
        }
    }
}

impl Folder for Unhygienic {
    fn fold_ident(&mut self, ns: Vec<Node>) -> Node {
        {
            let name = ns[0].expect_ident();
            if self.macros.contains_key(name) {
                let &(ref args, ref body) = &self.macros[name];
                assert!(ns.len() - 1 == args.len());
                return body.subst(args, &ns[1..]);
            }
        }
        Node::S(ns)
    }    

    fn fold_macro(&mut self, ns: Vec<Node>) -> Node {
        let mut ns = ns;
        let name = ns[1].expect_ident().clone();
        let body = ns.pop().unwrap();
        // FIXME some kind of split would be more efficient.
        let args = ns[2..].iter().map(|n| n.expect_ident().clone()).collect();
        self.macros.insert(name, (args, body));
        s!()
    }    
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::Node;

    #[test]
    fn test_noop() {
        let noop = &mut NoopFolder;
        let p = program!();
        assert!(p.clone() == fold(p, noop));
        let p = program!(s!(Node::Print, lit_str!("Hello world!")));
        assert!(p.clone() == fold(p, noop));
        let p = program!(ident!("a"),
                         s!(Node::Let,
                            ident!("a"),
                            lit_num!(42),
                            s!(Node::Fn,
                               ident!("x"),
                               s!(Node::Plus, ident!("x"), ident!("a")))));
        assert!(p.clone() == fold(p, noop));
        let p = program!(s!(s!(Node::Fn, ident!("x"), s!(Node::Plus, ident!("x"), lit_num!(42))),
                            s!(Node::Plus, lit_num!(3), lit_str!("a string"))));
        assert!(p.clone() == fold(p, noop));
    }
}
