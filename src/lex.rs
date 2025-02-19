use super::slice_iter::SliceIter;
use std::fmt::{self, Display, Formatter};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Token {
    Keyword(Keyword),
    Constant(Constant),
    Identifier(Identifier),
    OpenParen,
    CloseParen,
    OpenBrace,
    Semicolon,
    CloseBrace,
    Tilde,
    Decrement,
    Minus,
    Plus,

    PlusEqual,
    MinusEqual,
    TimesEqual,
    DivEqual,
    PercentEqual,
    BitAndEqual,
    BitOrEqual,
    BitXorEqual,

    Asterisk,
    Slash,
    Percent,
    Ampersand,
    Bar,
    Caret,
    Increment,
    LeftShift,
    LeftShiftEqual,
    RightShift,
    RightShiftEqual,
    Not,
    LogicalAnd,
    LogicalOr,
    EqualTo,
    NotEqual,
    LessThan,
    GreaterThan,
    Leq,
    Geq,
    Equals,

    QuestionMark,
    Colon,
}

pub fn tokenize(bytes: &[u8]) -> Result<Box<[Token]>, Error> {
    let mut iter = SliceIter::new(bytes);

    let mut tokens = Vec::new();
    while let Some(token) = lex_slice(&mut iter)? {
        tokens.push(token);
    }
    Ok(tokens.into())
}

fn lex_slice(iter: &mut SliceIter<u8>) -> Result<Option<Token>, Error> {
    match iter.as_slice() {
        [b'<', b'<', b'=', ..] => {
            iter.next();
            iter.next();
            iter.next();
            Ok(Some(Token::LeftShiftEqual))
        }
        [b'>', b'>', b'=', ..] => {
            iter.next();
            iter.next();
            iter.next();
            Ok(Some(Token::RightShiftEqual))
        }
        [b'-', b'-', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::Decrement))
        }
        [b'<', b'<', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::LeftShift))
        }
        [b'&', b'&', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::LogicalAnd))
        }
        [b'|', b'|', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::LogicalOr))
        }
        [b'!', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::NotEqual))
        }
        [b'=', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::EqualTo))
        }
        [b'>', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::Geq))
        }
        [b'<', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::Leq))
        }
        [b'>', b'>', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::RightShift))
        }
        [b'+', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::PlusEqual))
        }
        [b'+', b'+', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::Increment))
        }
        [b'-', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::MinusEqual))
        }
        [b'*', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::TimesEqual))
        }
        [b'/', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::DivEqual))
        }
        [b'%', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::PercentEqual))
        }
        [b'&', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::BitAndEqual))
        }
        [b'|', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::BitOrEqual))
        }
        [b'^', b'=', ..] => {
            iter.next();
            iter.next();
            Ok(Some(Token::BitXorEqual))
        }

        [a, ..] if !a.is_ascii() => error("Invalid Character (I Only Accept Ascii :[)"),
        [a, ..] if a.is_ascii_whitespace() => {
            iter.next();
            lex_slice(iter)
        }
        [a, ..] => {
            iter.next();
            Ok(Some(match a {
                b'(' => Token::OpenParen,
                b')' => Token::CloseParen,
                b'{' => Token::OpenBrace,
                b';' => Token::Semicolon,
                b'}' => Token::CloseBrace,
                b'~' => Token::Tilde,
                b'0'..=b'9' => {
                    let byte = AsciiDigit::from_int(*a).unwrap();
                    Token::Constant(constant_number(byte, iter)?)
                }
                b'-' => Token::Minus,
                b'+' => Token::Plus,
                b'*' => Token::Asterisk,
                b'/' => Token::Slash,
                b'%' => Token::Percent,
                b'&' => Token::Ampersand,
                b'|' => Token::Bar,
                b'^' => Token::Caret,
                b'!' => Token::Not,
                b'<' => Token::LessThan,
                b'>' => Token::GreaterThan,
                b'=' => Token::Equals,

                b'?' => Token::QuestionMark,
                b':' => Token::Colon,
                a => literal(*a, iter)?,
            }))
        }
        [] => Ok(None),
    }
}

impl AsciiDigit {
    const fn from_int(int: u8) -> Option<Self> {
        match int {
            b'0' => Some(AsciiDigit::Zero),
            b'1' => Some(AsciiDigit::One),
            b'2' => Some(AsciiDigit::Two),
            b'3' => Some(AsciiDigit::Three),
            b'4' => Some(AsciiDigit::Four),
            b'5' => Some(AsciiDigit::Five),
            b'6' => Some(AsciiDigit::Six),
            b'7' => Some(AsciiDigit::Seven),
            b'8' => Some(AsciiDigit::Eight),
            b'9' => Some(AsciiDigit::Nine),
            _ => None,
        }
    }
}

fn constant_number(start: AsciiDigit, iter: &mut SliceIter<u8>) -> Result<Constant, Error> {
    let mut bytes = vec![start];
    while let Some(constant) = next_if_number(iter) {
        bytes.push(constant);
    }
    if iter.peek().is_some_and(|byte| !word_character(byte)) {
        let number = parse_digit(&bytes);
        Ok(Constant::Integer(number))
    } else {
        Err(Error::InvalidConstant)
    }
}

fn literal(byte: u8, iter: &mut SliceIter<u8>) -> Result<Token, Error> {
    let mut bytes = vec![byte];
    while let Some(character) = next_if_word(iter) {
        bytes.push(character);
    }
    if iter.peek().is_some_and(|byte| !word_character(byte)) {
        Ok(match bytes.as_slice() {
            b"int" => Keyword::Int.into(),
            b"return" => Keyword::Return.into(),
            b"void" => Keyword::Void.into(),
            b"if" => Keyword::If.into(),
            b"else" => Keyword::Else.into(),
            b"goto" => Keyword::Goto.into(),
            b"do" => Keyword::Do.into(),
            b"while" => Keyword::While.into(),
            b"for" => Keyword::For.into(),
            b"break" => Keyword::Break.into(),
            b"continue" => Keyword::Continue.into(),
            b"switch" => Keyword::Switch.into(),
            b"case" => Keyword::Case.into(),
            b"default" => Keyword::Default.into(),
            _ => identifier(bytes.into())?.into(),
        })
    } else {
        Err(Error::InvalidLiteral)
    }
}

fn identifier(bytes: Box<[u8]>) -> Result<Identifier, Error> {
    if word_start(bytes[0]) && bytes[1..].iter().all(|&x| word_character(x)) {
        Ok(Identifier(bytes))
    } else {
        Err(Error::InvalidIdentifier)
    }
}

const fn _word_boundary(byte: u8) -> bool {
    !word_character(byte)
}

const fn word_start(byte: u8) -> bool {
    match byte {
        b if b.is_ascii_alphabetic() => true,
        b'_' => true,
        _ => false,
    }
}

const fn word_character(byte: u8) -> bool {
    match byte {
        b if b.is_ascii_alphanumeric() => true,
        b'_' => true,
        _ => false,
    }
}

fn next_if_number(iter: &mut SliceIter<u8>) -> Option<AsciiDigit> {
    iter.next_if_map(AsciiDigit::from_int)
}

#[derive(Clone, Copy)]
enum AsciiDigit {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
}

fn parse_digit(slice: &[AsciiDigit]) -> u64 {
    let mut cur = 0u64;
    for (place, digit) in slice.iter().map(|&x| u64::from(x as u8)).rev().enumerate() {
        cur += 10u64.pow(place as u32) * digit;
    }
    cur
}

impl Token {
    pub const fn identifier(&self) -> bool {
        matches!(self, Self::Identifier(_))
    }
    pub const fn constant(&self) -> bool {
        matches!(self, Self::Constant(_))
    }

    pub const fn keyword(&self) -> bool {
        matches!(self, Self::Keyword(_))
    }
}

fn next_if_word(iter: &mut SliceIter<u8>) -> Option<u8> {
    iter.next_if(word_character)
}

impl PartialEq<Token> for Keyword {
    fn eq(&self, other: &Token) -> bool {
        if let Token::Keyword(k) = other {
            k == self
        } else {
            false
        }
    }
}

impl PartialEq<Token> for Constant {
    fn eq(&self, other: &Token) -> bool {
        if let Token::Constant(c) = other {
            c == self
        } else {
            false
        }
    }
}

impl PartialEq<Token> for Identifier {
    fn eq(&self, other: &Token) -> bool {
        if let Token::Identifier(c) = other {
            c == self
        } else {
            false
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Identifier(pub Box<[u8]>);

impl Identifier {
    pub fn new(name: &[u8]) -> Self {
        Self(name.into())
    }

    pub fn new_rc(name: &[u8]) -> std::rc::Rc<Self> {
        Self(name.into()).into()
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", unsafe { std::str::from_utf8_unchecked(&self.0) })
    }
}

impl AsRef<[u8]> for Identifier {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Identifier> for Token {
    fn from(i: Identifier) -> Self {
        Self::Identifier(i)
    }
}

impl From<Keyword> for Token {
    fn from(k: Keyword) -> Self {
        Self::Keyword(k)
    }
}

impl From<Constant> for Token {
    fn from(c: Constant) -> Self {
        Self::Constant(c)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Keyword {
    Int,
    Void,
    Return,
    If,
    Else,
    Goto,
    Do,
    While,
    For,
    Break,
    Continue,
    Switch,
    Default,
    Case,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Constant {
    Integer(u64),
}

#[derive(Debug)]
pub enum Error {
    InvalidConstant,
    InvalidLiteral,
    InvalidIdentifier,
    NotAscii,
    Other(String),
}

fn error<T>(message: &str) -> Result<T, Error> {
    Err(Error::Other(message.into()))
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use std::str::from_utf8_unchecked;
        write!(f, "{}", unsafe { from_utf8_unchecked(&self.0) })
    }
}
