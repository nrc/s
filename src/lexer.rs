use std::fmt;
use std::ops::Deref;
use std::str::Chars;
use std::iter::{Iterator, Peekable};
use KEYWORDS;

// Token defintions.

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    Bra,
    Ket,
    Keyword(&'static str),
    Str(Str),
    Number(u32),
    Name(Str),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Bra => '('.fmt(f),
            Token::Ket => ')'.fmt(f),
            Token::Keyword(ref s) => s.fmt(f),
            Token::Str(ref s) => s.fmt(f),
            Token::Number(n) => n.fmt(f),
            Token::Name(ref s) => s.fmt(f),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Str(String);

impl Str {
    pub fn new(s: &str) -> Str {
        Str(s.to_owned())
    }
}

impl Deref for Str {
    type Target = str;

    fn deref(&self) -> &str {
        &&self.0
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}


// Lexing.

pub fn lex(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    lexer.lex()
}

struct Lexer<'a> {
    iter: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            iter: input.chars().peekable(),
        }
    }

    fn lex(&mut self) -> Vec<Token> {
        let mut result = vec![];

        while !self.done() {
            let t = self.next_token();
            result.extend(t.into_iter());
        }

        result
    }

    fn done(&mut self) -> bool {
        self.iter.peek().is_none()
    }

    fn next_token(&mut self) -> Option<Token> {
        self.eat_whitespace();
        let c = self.iter.peek().map(|c| *c);
        c.map(|c| {
            match c {
                '(' => {
                    self.iter.next();
                    Token::Bra
                }
                ')' => {
                    self.iter.next();
                    Token::Ket
                }
                '"' => self.lex_string(),
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => self.lex_number(),
                _ => self.lex_keyword_or_name(),
            }            
        })
    }

    fn eat_whitespace(&mut self) {
        while let Some(&c) = self.iter.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.iter.next();
        }
    }

    // Current char is "; returns Token::Str.
    fn lex_string(&mut self) -> Token {
        // eat "
        self.iter.next();

        let mut result = String::new();
        while let Some(&c) = self.iter.peek() {
            self.iter.next();
            if c == '"' {
                break;
            }
            result.push(c);
        }

        Token::Str(Str(result))
    }

    // Current char is a numeral; returns Token::Number.
    fn lex_number(&mut self) -> Token {
        let mut result = String::new();
        while let Some(&c) = self.iter.peek() {
            if !c.is_digit(10) {
                break;
            }
            self.iter.next();
            result.push(c);
        }
        
        Token::Number(result.parse().unwrap())
    }

    // Returns Token::Keyword or Token::Name.
    fn lex_keyword_or_name(&mut self) -> Token {
        let mut result = String::new();
        while let Some(&c) = self.iter.peek() {
            if c.is_whitespace() || c == '(' || c == ')' || c == '"' {
                break;
            }
            self.iter.next();
            result.push(c);
        }
        
        if let Ok(index) = KEYWORDS.binary_search(&&*result) {
            Token::Keyword(KEYWORDS[index])   
        } else {
            Token::Name(Str(result))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty() {
        assert!(lex("") == vec![]);
    }

    #[test]
    fn test_single() {
        assert!(lex("(") == vec![Token::Bra]);
        assert!(lex(")") == vec![Token::Ket]);
        assert!(lex("let") == vec![Token::Keyword("let")]);
        assert!(lex("print") == vec![Token::Keyword("print")]);
        assert!(lex("+") == vec![Token::Keyword("+")]);
        assert!(lex("\"\"") == vec![Token::Str(Str::new(""))]);
        assert!(lex("\"foo\"") == vec![Token::Str(Str::new("foo"))]);
        assert!(lex("\"foo + 3 + bar\"") == vec![Token::Str(Str::new("foo + 3 + bar"))]);
        assert!(lex("0") == vec![Token::Number(0)]);
        assert!(lex("1") == vec![Token::Number(1)]);
        assert!(lex("42") == vec![Token::Number(42)]);
        assert!(lex("foo") == vec![Token::Name(Str::new("foo"))]);
        assert!(lex("FOO") == vec![Token::Name(Str::new("FOO"))]);
        assert!(lex("Bar") == vec![Token::Name(Str::new("Bar"))]);
        assert!(lex("qux42") == vec![Token::Name(Str::new("qux42"))]);
    }

    #[test]
    fn test_single_ws() {
        assert!(lex("    \n  ") == vec![]);
        assert!(lex(" (") == vec![Token::Bra]);
        assert!(lex(") ") == vec![Token::Ket]);
        assert!(lex("let  ") == vec![Token::Keyword("let")]);
        assert!(lex("print\n") == vec![Token::Keyword("print")]);
        assert!(lex(" \n   +") == vec![Token::Keyword("+")]);
        assert!(lex("\n\n\"\"") == vec![Token::Str(Str::new(""))]);
        assert!(lex(" \"foo\" ") == vec![Token::Str(Str::new("foo"))]);
        assert!(lex("     \"foo + 3 + bar\"\n  \n ") == vec![Token::Str(Str::new("foo + 3 + bar"))]);
    }

    #[test]
    fn test_two() {
        assert!(lex("()") == vec![Token::Bra,Token::Ket]);
        assert!(lex("))") == vec![Token::Ket,Token::Ket]);
        assert!(lex("let(") == vec![Token::Keyword("let"), Token::Bra]);
        assert!(lex("print)") == vec![Token::Keyword("print"),Token::Ket]);
        assert!(lex("foo\"\"") == vec![Token::Name(Str::new("foo")),Token::Str(Str::new(""))]);
        assert!(lex("\"foo\"foo") == vec![Token::Str(Str::new("foo")), Token::Name(Str::new("foo"))]);
        assert!(lex("\"foo + 3 + bar\"+") == vec![Token::Str(Str::new("foo + 3 + bar")), Token::Keyword("+")]);
        assert!(lex("0foo") == vec![Token::Number(0),Token::Name(Str::new("foo"))]);
    }

    #[test]
    fn test_two_ws() {
        assert!(lex("( )") == vec![Token::Bra,Token::Ket]);
        assert!(lex(") )") == vec![Token::Ket,Token::Ket]);
        assert!(lex("let\n(") == vec![Token::Keyword("let"), Token::Bra]);
        assert!(lex("+\n+") == vec![Token::Keyword("+"), Token::Keyword("+")]);
        assert!(lex("print      )") == vec![Token::Keyword("print"),Token::Ket]);
        assert!(lex("foo \"\"") == vec![Token::Name(Str::new("foo")),Token::Str(Str::new(""))]);
        assert!(lex("\"foo\" foo") == vec![Token::Str(Str::new("foo")), Token::Name(Str::new("foo"))]);
        assert!(lex("\"foo + 3 + bar\" +") == vec![Token::Str(Str::new("foo + 3 + bar")), Token::Keyword("+")]);
        assert!(lex("0\n\nfoo") == vec![Token::Number(0),Token::Name(Str::new("foo"))]);
        assert!(lex("foo 42") == vec![Token::Name(Str::new("foo")), Token::Number(42)]);
    }

    #[test]
    fn test_realistic() {
        assert!(lex("(print \"Hello world!\")") == vec![Token::Bra,
                                                        Token::Keyword("print"),
                                                        Token::Str(Str::new("Hello world!")),
                                                        Token::Ket]);
        assert!(lex("a (let a 42 (fn x (+ x a)))") == vec![Token::Name(Str::new("a")),
                                                           Token::Bra,
                                                           Token::Keyword("let"),
                                                           Token::Name(Str::new("a")),
                                                           Token::Number(42),
                                                           Token::Bra,
                                                           Token::Keyword("fn"),
                                                           Token::Name(Str::new("x")),
                                                           Token::Bra,
                                                           Token::Keyword("+"),
                                                           Token::Name(Str::new("x")),
                                                           Token::Name(Str::new("a")),
                                                           Token::Ket,
                                                           Token::Ket,
                                                           Token::Ket]);
        assert!(lex("((fn x (+ x 42)) (+ 3 \"a string\"))") == vec![Token::Bra,
                                                                    Token::Bra,
                                                                    Token::Keyword("fn"),
                                                                    Token::Name(Str::new("x")),
                                                                    Token::Bra,
                                                                    Token::Keyword("+"),
                                                                    Token::Name(Str::new("x")),
                                                                    Token::Number(42),
                                                                    Token::Ket,
                                                                    Token::Ket,
                                                                    Token::Bra,
                                                                    Token::Keyword("+"),
                                                                    Token::Number(3),
                                                                    Token::Str(Str::new("a string")),
                                                                    Token::Ket,
                                                                    Token::Ket]);
    }
}
