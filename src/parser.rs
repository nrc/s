use lexer::{Token, Str};
use std::fmt;

// AST
// A Program is basically an s expression without parentheses, it only occurs at
// the top level of the program. An expression may not be empty, it must start
// with either a node, followed by any number of nodes, except keywords.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    Program(Vec<Node>),
    S(Vec<Node>),
    Plus,
    Fn,
    Let,
    Print,
    Ident(Str),
    LitNum(u32),
    LitStr(Str),
}

impl Node {
    fn push(&mut self, n: Node) {
        match *self {
            Node::Program(ref mut ns) | Node::S(ref mut ns) => ns.push(n),
            _ => panic!("Can't push to {:?}", self),
        }
    }

    pub fn is_keyword(&self) -> bool {
        match *self {
            Node::Plus | Node::Fn | Node::Let | Node::Print => true,
            _ => false,
        }
    }

    pub fn is_value(&self) -> bool {
        match *self {
            Node::LitStr(_) | Node::LitNum(_) => true,
            Node::S(ref ns) => ns.len() == 0 || &ns[0] == &Node::Fn,
            _ => false,
        }

    }

    pub fn expect_lit_num(&self) -> u32 {
        if let &Node::LitNum(n) = self {
            n
        } else {
            panic!("expected LitNum, found {:?}", self)
        }
    }

    pub fn expect_ident(&self) -> &Str {
        if let &Node::Ident(ref s) = self {
            s
        } else {
            panic!("expected Ident, found {:?}", self)
        }
    }
}

// Builder macros
macro_rules! program {
    ($($ns: expr),*) => (::parser::Node::Program(vec![$($ns),*]))
}
macro_rules! s {
    ($($ns: expr),*) => (::parser::Node::S(vec![$($ns),*]))
}
macro_rules! ident {
    ($s: expr) => (::parser::Node::Ident(::lexer::Str::new($s)))
}
macro_rules! lit_num {
    ($n: expr) => (::parser::Node::LitNum($n))
}
macro_rules! lit_str {
    ($s: expr) => (::parser::Node::LitStr(::lexer::Str::new($s)))
}

// FIXME could pretty print, if I could be bothered.
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn write_node_list(ns: &[Node], f: &mut fmt::Formatter) -> fmt::Result {
            let mut first = true;
            for n in ns {
                if first {
                    first = false;
                } else {
                    try!(write!(f, " "));
                }
                try!(n.fmt(f));
            }

            Ok(())
        }

        match *self {
            Node::Program(ref ns) => {
                try!(write_node_list(ns, f));
            }
            Node::S(ref ns) => {
                try!(write!(f, "("));
                try!(write_node_list(ns, f));
                try!(write!(f, ")"));
            }
            Node::Plus => try!(write!(f, "+")),
            Node::Fn => try!(write!(f, "fn")),
            Node::Let => try!(write!(f, "let")),
            Node::Print => try!(write!(f, "print")),
            Node::Ident(ref s) => try!(write!(f, "{}", s)),
            Node::LitNum(n) => try!(write!(f, "{}", n)),
            Node::LitStr(ref s) => try!(write!(f, "{}", s)),
        }

        Ok(())
    }
}

pub fn parse(input: &[Token]) -> Node {
    let mut expr_stack = Vec::new();
    let mut cur_node = Node::Program(Vec::new());
    let mut i = 0;
    loop {
        if i >= input.len() {
            break;
        }

        match input[i] {
            Token::Bra => {
                expr_stack.push(cur_node);
                cur_node = Node::S(Vec::new());
            }
            Token::Ket => {
                match cur_node {
                    Node::S(..) => {
                        let old_cur = cur_node;
                        cur_node = expr_stack.pop().unwrap();
                        cur_node.push(old_cur);
                    }
                    _ => panic!("Unexpected `)`"),
                }
            }

            Token::Keyword("+") => cur_node.push(Node::Plus),
            Token::Keyword("fn") => cur_node.push(Node::Fn),
            Token::Keyword("let") => cur_node.push(Node::Let),
            Token::Keyword("print") => cur_node.push(Node::Print),
            Token::Name(ref s) => cur_node.push(Node::Ident(s.clone())),
            Token::Number(n) => cur_node.push(Node::LitNum(n)),
            Token::Str(ref s) => cur_node.push(Node::LitStr(s.clone())),
            _ => unreachable!(),
        }

        i += 1;
    }

    debug!("parsed: {:?}", expr_stack);
    assert!(expr_stack.is_empty(), "Unexpected EOF");

    match cur_node {
        Node::Program(..) => cur_node,
        _ => panic!("Expected Program, found {:?}", cur_node),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lexer::lex;

    #[test]
    fn test_empty() {
        assert!(parse(&lex("")) == Node::Program(Vec::new()));
    }

    #[test]
    fn test_simple() {
        assert!(parse(&lex("fn")) == program!(Node::Fn));
        assert!(parse(&lex("let")) == program!(Node::Let));
        assert!(parse(&lex("+")) == program!(Node::Plus));
        assert!(parse(&lex("42")) == program!(lit_num!(42)));
        assert!(parse(&lex("foo")) == program!(ident!("foo")));
        assert!(parse(&lex("a b")) == program!(ident!("a"), ident!("b")));
        assert!(parse(&lex("(+ 1 2)")) == program!(s!(Node::Plus, lit_num!(1), lit_num!(2))));
    }


    #[test]
    fn test_realistic() {
        assert!(parse(&lex("(print \"Hello world!\")")) ==
                program!(s!(Node::Print, lit_str!("Hello world!"))));
        assert!(parse(&lex("a (let a 42 (fn x (+ x a)))")) ==
                program!(ident!("a"),
                         s!(Node::Let,
                            ident!("a"),
                            lit_num!(42),
                            s!(Node::Fn,
                               ident!("x"),
                               s!(Node::Plus, ident!("x"), ident!("a"))))));
        assert!(parse(&lex("((fn x (+ x 42))(+ 3 \"a string\"))")) ==
                program!(s!(s!(Node::Fn, ident!("x"), s!(Node::Plus, ident!("x"), lit_num!(42))),
                            s!(Node::Plus, lit_num!(3), lit_str!("a string")))));
    }

    #[test]
    #[should_panic]
    fn test_fail_unclosed() {
        parse(&lex("("));
    }

    #[test]
    #[should_panic]
    fn test_fail_unclosed2() {
        parse(&lex("((fn x a) baz 42 "));
    }

    #[test]
    #[should_panic]
    fn test_fail_too_closed() {
        parse(&lex("(baz 42 ))"));
    }

    #[test]
    #[should_panic]
    fn test_fail_too_closed2() {
        parse(&lex(")"));
    }
}
