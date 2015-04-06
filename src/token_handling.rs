use syntax::parse::lexer::{
    StringReader,
    TokenAndSpan,
    Reader,
};
use syntax::parse::token;
use syntax::parse::token::Token;
use std::iter;

pub fn put_tokens_into_vec(lexer: &mut StringReader) -> Vec < Word > {
    let mut tokens = Vec::new();
    loop {
        match lexer.next_token() {
            TokenAndSpan {
                tok:token::Eof,
                sp:_,
            } => {
                tokens.push(Word::Eof);
                break;
            },
            TokenAndSpan {
                tok:token,
                sp:span,
            } => {
                let word = Word::from_token(token);
                match word {
                    Word::Nope => {},
                    Word::Whitespace(_) => {
                        let s = lexer.span_diagnostic.cm.span_to_snippet(span).unwrap();
                        if s.contains("\n\n") {
                            tokens.push(Word::LineBreakDouble);
                        } else if s.contains("\n") {
                            tokens.push(Word::LineBreak);
                        }
                    },
                    Word::Comment(_) => {
                        let s = lexer.span_diagnostic.cm.span_to_snippet(span).unwrap();
                        tokens.push(Word::Comment(s));
                    },
                    _ => tokens.push(word),
                }
            },
        }
    }
    tokens
}

// A word is a token reduced to all the information the pretty printing process requires. The
// different types of are not necessarily the same as the token types.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Word {
    // '+' '==' etc. - everything that must be set with one leading and one trailing space
    BinaryOperator(String),
    // '!' etc. - everything that must be set with one leading and no trailing space
    PrefixOperator(String),
    // '{' '(' ')' etc.
    // '.' '..' '...' etc.  - everything that must be set with no leading and no trailing space
    SlimInfix(String),
    // everything else, no need for disctiontion for the typesetting process.
    Other(String),
    Comment(String),
    LineBreak,
    LineBreakDouble,
    LineBreakIntentPlus,
    LineBreakIntentMinus,
    SemiColon,
    Comma,
    Colon,
    Whitespace(i32),
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Nope,
    Eof,
}

impl Word {
    pub fn to_string(self) -> String {
        match self {
            Word::BinaryOperator(s) => s,
            Word::PrefixOperator(s) => s,
            Word::OpenParen => "(".to_string(),
            Word::CloseParen => ")".to_string(),
            Word::OpenBrace => "{".to_string(),
            Word::CloseBrace => "}".to_string(),
            Word::OpenBracket => "[".to_string(),
            Word::CloseBracket => "]".to_string(),
            Word::SemiColon => ";".to_string(),
            Word::Comma => ",".to_string(),
            Word::Colon => ":".to_string(),
            Word::SlimInfix(s) => s,
            Word::Other(s) => s,
            Word::Comment(s) => convert_comment(s.as_ref()).to_string(),
            Word::LineBreak | Word::LineBreakIntentPlus | Word::LineBreakIntentMinus =>
            "\n".to_string(),
            Word::LineBreakDouble => "\n\n".to_string(),
            Word::Whitespace(i) =>
            if i > 0 {
                format!("{}", repeat(" ", i as usize))
            } else {
                "".to_string()
            },
            Word::Nope => "".to_string(),
            Word::Eof => "".to_string(),
        }
    }

    // This functions is mapping tokens to words.
    // Sorry for this gigantic monster of a match statement.
    fn from_token(token: Token) -> Word {
        match token {
            token::Eq => Word::BinaryOperator("=".to_string()),
            token::EqEq => Word::BinaryOperator("==".to_string()),
            token::Ne => Word::BinaryOperator("!=".to_string()),
            token::Ge => Word::BinaryOperator(">=".to_string()),
            token::Gt => Word::BinaryOperator(">".to_string()),
            token::Le => Word::BinaryOperator("<=".to_string()),
            token::Lt => Word::BinaryOperator("<".to_string()),
            token::AndAnd => Word::BinaryOperator("&&".to_string()),
            token::OrOr => Word::BinaryOperator("||".to_string()),
            token::Not => Word::PrefixOperator("!".to_string()),
            token::Tilde => Word::PrefixOperator("~".to_string()),
            token::BinOp(bin_op_token) =>
            match bin_op_token {
                token::Plus => Word::BinaryOperator("+".to_string()),
                token::Minus => Word::BinaryOperator("-".to_string()),
                token::Star => Word::BinaryOperator("*".to_string()),
                token::Slash => Word::BinaryOperator("/".to_string()),
                token::Percent => Word::BinaryOperator("%".to_string()),
                token::Caret => Word::BinaryOperator("^".to_string()),
                token::And => Word::PrefixOperator("&".to_string()),
                token::Or => Word::BinaryOperator("|".to_string()),
                token::Shl => Word::BinaryOperator("<<".to_string()),
                token::Shr => Word::BinaryOperator(">>".to_string()),
            },
            token::BinOpEq(bin_op_token) =>
            match bin_op_token {
                token::Plus => Word::BinaryOperator("+=".to_string()),
                token::Minus => Word::BinaryOperator("-=".to_string()),
                token::Star => Word::BinaryOperator("*=".to_string()),
                token::Slash => Word::BinaryOperator("/=".to_string()),
                token::Percent => Word::BinaryOperator("%=".to_string()),
                token::Caret => Word::BinaryOperator("^=".to_string()),
                token::And => Word::BinaryOperator("&=".to_string()),
                token::Or => Word::BinaryOperator("|=".to_string()),
                token::Shl => Word::BinaryOperator("<<=".to_string()),
                token::Shr => Word::BinaryOperator(">>=".to_string()),
            },
            token::At => Word::PrefixOperator("@".to_string()),
            token::Dot => Word::SlimInfix(".".to_string()),
            token::DotDot => Word::SlimInfix("..".to_string()),
            token::DotDotDot => Word::SlimInfix("...".to_string()),
            token::ModSep => Word::SlimInfix("::".to_string()),
            token::Comma => Word::Comma,
            token::Semi => Word::SemiColon,
            token::Colon => Word::Colon,

            token::RArrow => Word::BinaryOperator("->".to_string()),
            token::LArrow => Word::Other("<-".to_string()),
            token::FatArrow => Word::BinaryOperator("=>".to_string()),

            token::OpenDelim(token::Paren) => Word::OpenParen,
            token::CloseDelim(token::Paren) => Word::CloseParen,
            token::OpenDelim(token::Bracket) => Word::OpenBracket,
            token::CloseDelim(token::Bracket) => Word::CloseBracket,
            token::OpenDelim(token::Brace) => Word::OpenBrace,
            token::CloseDelim(token::Brace) => Word::CloseBrace,

            token::Pound => Word::PrefixOperator("#".to_string()),
            token::Dollar => Word::PrefixOperator("$".to_string()),
            token::Question => Word::PrefixOperator("?".to_string()),
            token::Literal(lit, suf) => {
                let mut out = match lit {
                    token::Byte(b) => format!("b'{}'", b.as_str()),
                    token::Char(c) => format!("'{}'", c.as_str()),
                    token::Float(c) => c.as_str().to_string(),
                    token::Integer(c) => c.as_str().to_string(),
                    token::Str_(s) => format!("\"{}\"", s.as_str()),
                    token::StrRaw(s, n) => format!("r{delim}\"{string}\"{delim}",
                        delim = repeat("#", n),
                        string = s.as_str()),
                    token::Binary(v) => format!("b\"{}\"", v.as_str()),
                    token::BinaryRaw(s, n) => format!("br{delim}\"{string}\"{delim}",
                        delim = repeat("#", n),
                        string = s.as_str()),
                };
                if let Some(s) = suf {
                    out.push_str(s.as_str())
                }
                Word::Other(out)
            },
            token::Ident(s, _) => {
                let s = token::get_ident(s).to_string();
                if s == "as" {
                    Word::BinaryOperator(s)
                } else {
                    Word::Other(s)
                }
            }
            token::Lifetime(s) => Word::Other(format!("{}", token::get_ident(s))),  // ???
            token::Underscore => Word::Other("_".to_string()),
            // not alle whitespaces are linebreaks, but we decide that in fn put_tokens_into_vec
            token::Whitespace => Word::Whitespace(0),
            token::DocComment(s) => Word::Comment(s.to_string()),
            token::Comment => Word::Comment("".to_string()),
            token::Eof => {
                unreachable!()
            },
            _ => Word::Nope,
        }
    }
}

// little helper functions
fn repeat(s: &str, n: usize) -> String {
    iter::repeat(s).take(n).collect()
}

// converts comments wrapped in /* */ to line comment
// does nothing right now... TODO: implement the thing
fn convert_comment(comment: &str) -> &str {
    if comment.starts_with("/*") {
        comment
    } else {
        comment
    }
}
