#![allow(dead_code)]

use std::iter::Peekable;
use std::str::CharIndices;

use tokens::{Token, TokenKind};

pub mod tokens;

#[derive(Clone)]
pub struct Lexer<'src> {
    content: &'src str,
    chars: Peekable<CharIndices<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn new(program: &'src str) -> Self {
        Lexer {
            content: program,
            chars: program.char_indices().peekable(),
        }
    }

    fn next_token(&mut self) -> Token {
        use tokens::TokenKind::*;

        self.skip_whitespace();

        let Some((cur_index, cur_char)) = self.peek() else {
            return Token::new(Eof, self.content.len(), 1);
        };

        match cur_char {
            '#' => {
                self.next();
                while let Some((_, c)) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.next();
                }
                self.next_token()
            }
            '+' => self.make_single_char_token(cur_index, Plus),
            '-' => {
                self.next();
                if self
                    .chars
                    .next_if(|(_, next_char)| *next_char == '>')
                    .is_some()
                {
                    Token::new(RArrow, cur_index, 2)
                } else {
                    Token::new(Minus, cur_index, 1)
                }
            }
            '/' => self.make_single_char_token(cur_index, Slash),
            '*' => self.make_single_char_token(cur_index, Asterisk),
            '&' => self.make_single_char_token(cur_index, BitAnd),
            '|' => self.make_single_char_token(cur_index, BitOr),
            '^' => self.make_single_char_token(cur_index, BitXor),
            '=' => {
                self.next();
                if self
                    .chars
                    .next_if(|(_, next_char)| *next_char == '=')
                    .is_some()
                {
                    Token::new(DoubleEqual, cur_index, 2)
                } else {
                    Token::new(SingleEqual, cur_index, 1)
                }
            }
            '!' => {
                self.next();
                if self
                    .chars
                    .next_if(|(_, next_char)| *next_char == '=')
                    .is_some()
                {
                    Token::new(NotEqual, cur_index, 2)
                } else {
                    Token::new(ExclamationMark, cur_index, 1)
                }
            }
            '<' => {
                self.next();
                if self
                    .chars
                    .next_if(|(_, next_char)| *next_char == '=')
                    .is_some()
                {
                    Token::new(LessThanOrEqual, cur_index, 2)
                } else {
                    Token::new(LessThan, cur_index, 1)
                }
            }
            '>' => {
                self.next();
                if self
                    .chars
                    .next_if(|(_, next_char)| *next_char == '=')
                    .is_some()
                {
                    Token::new(GreaterThanOrEqual, cur_index, 2)
                } else {
                    Token::new(GreaterThan, cur_index, 1)
                }
            }
            '(' => self.make_single_char_token(cur_index, LParen),
            ')' => self.make_single_char_token(cur_index, RParen),
            '{' => self.make_single_char_token(cur_index, LBrace),
            '}' => self.make_single_char_token(cur_index, RBrace),
            '~' => self.make_single_char_token(cur_index, Tilde),
            ':' => self.make_single_char_token(cur_index, Colon),
            ',' => self.make_single_char_token(cur_index, Comma),
            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = self.read_ident(cur_index);
                match ident {
                    "fun" => Token::new(Fun, cur_index, 3),
                    "take" => Token::new(Take, cur_index, 4),
                    "mut" => Token::new(Mut, cur_index, 3),
                    "read" => Token::new(Read, cur_index, 4),
                    "give" => Token::new(Give, cur_index, 4),
                    "break" => Token::new(Break, cur_index, 5),
                    "pass" => Token::new(Pass, cur_index, 4),
                    "ret" => Token::new(Return, cur_index, 3),
                    "proto" => Token::new(Proto, cur_index, 5),
                    "impl" => Token::new(Impl, cur_index, 4),
                    "type" => Token::new(Type, cur_index, 4),
                    "true" => Token::new(True, cur_index, 4),
                    "false" => Token::new(False, cur_index, 5),
                    s => Token::new(Identifier, cur_index, s.len()),
                }
            }
            '0'..='9' => {
                let size = self.read_int(cur_index);
                Token::new(IntLiteral, cur_index, size)
            }
            c => panic!("illegal token: {c}"),
        }
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.chars.peek().copied()
    }

    fn next(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn make_single_char_token(&mut self, position: usize, kind: TokenKind) -> Token {
        let tok = Token::new(kind, position, 1);
        self.next();
        tok
    }

    fn read_ident(&mut self, position: usize) -> &'src str {
        let mut last = position;
        while self
            .peek()
            .is_some_and(|(_, c)| c.is_ascii_alphanumeric() || c == '_')
        {
            let (l, _) = self.next().unwrap();
            last = l;
        }
        &self.content[position..last + 1]
    }

    fn read_int(&mut self, position: usize) -> usize {
        let mut last = position;
        while self.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
            let (l, _) = self.next().unwrap();
            last = l;
        }
        let number = &self.content[position..last + 1];
        number.len()
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|(_, c)| c.is_whitespace()) {
            self.next();
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.next_token();
        if tok.kind == TokenKind::Eof {
            None
        } else {
            Some(tok)
        }
    }
}
