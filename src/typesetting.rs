use token_handling::{
    put_tokens_into_vec,
    Word,
};
use syntax::parse;
use syntax::parse::lexer;

static SPACES_PER_TAP:i32 = 4;
static MAX_LINE_LENGTH:i32 = 100;
static MAX_INTENT:i32 = 80;

pub struct Typesetter {
    words:Box < Vec < Word >>
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum Context {
    CodeBlock,  // {...}
    CurlyList,  // {foo, bar}
    List,  // (...), [...] or <...>
    ListExploded  // same as list, but formated like a codeblock
}

impl Typesetter {
    pub fn new(source: &str) -> Typesetter {
        let session = parse::new_parse_sess();
        let filemap = parse::string_to_filemap(&session, source.to_string(), "<stdin>".to_string());
        let mut lexer = lexer::StringReader::new(&session.span_diagnostic, filemap);
        let mut typesetter = Typesetter {
            words:Box::new(put_tokens_into_vec(&mut lexer)),
        };
        if typesetter.words.len() > 2 {
            typesetter.filter_linebreaks();
            typesetter.sort_out_ambiguities();
            typesetter.format();
            // typesetter.handle_overlong_lines();
        }
        typesetter
    }

    fn filter_linebreaks(&mut self) {
        let mut result = Vec::new();
        let mut index = 1usize;

        if self.words.len() > 0 {
            result.push(self.words [0].clone())
        }
        loop {
            if index >= self.words.len() - 1 {
                break;
            }
            let prev = self.words[index - 1].clone();
            let word = self.words[index].clone();
            let peek = self.words[index + 1].clone();

            match (prev, word, peek) {
                (_, Word::LineBreak, Word::CloseBrace)
                | (_, Word::LineBreakDouble, Word::CloseBrace)
                | (_, Word::LineBreak, Word::CloseBracket)
                | (_, Word::LineBreakDouble, Word::CloseBracket)
                | (_, Word::LineBreak, Word::CloseParen)
                | (_, Word::LineBreakDouble, Word::CloseParen)
                | (Word::OpenBrace, Word::LineBreak, _)
                | (Word::OpenBrace, Word::LineBreakDouble, _)
                | (_, Word::LineBreak, Word::Comma)
                | (_, Word::LineBreak, Word::SemiColon)
                | (_, Word::LineBreakDouble, Word::Comma)
                | (_, Word::LineBreakDouble, Word::SemiColon)
                | (_, Word::LineBreak, Word::OpenBrace)
                | (_, Word::LineBreakDouble, Word::OpenBrace)
                | (_, Word::LineBreak, Word::OpenBracket)
                | (_, Word::LineBreakDouble, Word::OpenBracket)
                | (_, Word::LineBreak, Word::OpenParen)
                | (_, Word::LineBreakDouble, Word::OpenParen)
                | (_, Word::LineBreak, Word::SlimInfix(_))
                | (_, Word::LineBreakDouble, Word::SlimInfix(_))
                | (Word::SlimInfix(_), Word::LineBreak, _)
                | (Word::SlimInfix(_), Word::LineBreakDouble, _) => {}
                (Word::OpenBracket, 
                    Word::LineBreak, 
                    _)
                | (Word::OpenBracket, Word::LineBreakDouble, _)
                | (Word::OpenParen, Word::LineBreak, _)
                | (Word::OpenParen, Word::LineBreakDouble, _) => {
                    // exploded list
                    result.push(Word::LineBreakIntentPlus);
                }
                (Word::CloseBrace, 
                    Word::LineBreak, 
                    Word::Other(s))
                | (Word::CloseBrace, Word::LineBreakDouble, Word::Other(s)) => {
                    // put } else on one line
                    if s != "else" {
                        result.push(self.words [index].clone());
                    }
                }
                (Word::Other(s1), 
                    Word::LineBreak, 
                    Word::Other(s2))
                | (Word::Other(s1), Word::LineBreakDouble, Word::Other(s2)) => {
                    // always put else and if on one line
                    if !(s1 == "else" && s2 == "if") {
                        result.push(self.words [index].clone());
                    }
                }
                _ => result.push(self.words [index].clone()),
            }
            index += 1;
        }
        if self.words.len() > 0 {
            result.push(self.words [index].clone())
        }
        self.words = Box::new(result);
    }

    fn sort_out_ambiguities(&mut self) {
        let mut result = Vec::new();
        let mut index = 1usize;
        //let mut is_expr = true;

        if self.words.len() > 0 {
            result.push(self.words [0].clone())
        }

        loop {
            if index >= self.words.len() {
                break;
            }
            let prev = self.words[index - 1].clone();
            let word = self.words[index].clone();

            /*if word == Word::Colon
            || word == Word::BinaryOperator("->".to_string())
            || word == Word::Other("fn".to_string())
            || word == Word::Other("enum".to_string())
            || word == Word::Other("struct".to_string())
            || word == Word::Other("impl".to_string()) {
                is_expr = false;
            } else if word == Word::SemiColon || word == Word::OpenBrace {
                is_expr = true;
            }*/

            // heuristic for decision: dereference operator or muliplication
            match (prev, word) {
                (Word::BinaryOperator(_), Word::BinaryOperator(s))
                | (Word::OpenBracket, Word::BinaryOperator(s))
                | (Word::OpenParen, Word::BinaryOperator(s))
                | (Word::OpenBrace, Word::BinaryOperator(s))
                | (Word::LineBreak, Word::BinaryOperator(s))
                | (Word::LineBreakDouble, Word::BinaryOperator(s))
                | (Word::SemiColon, Word::BinaryOperator(s))
                | (Word::Comma, Word::BinaryOperator(s)) => {
                    if s == "*" {
                        result.push(Word::PrefixOperator("*".to_string()));
                    } else {
                        result.push(self.words [index].clone());
                    }
                }
                (Word::Other(w), 
                    Word::PrefixOperator(p)) => {
                    if w != "if" && p == "!" {
                        result.push(Word::SlimInfix("!".to_string()));
                    } else {
                        result.push(self.words [index].clone());
                    }
                }
                (Word::Other(w), 
                    Word::BinaryOperator(b)) => {
                    if (w == "match" || w == "for" || w == "if" || w == "in" || w == "as")
                    && b == "*" {
                        result.push(Word::PrefixOperator("*".to_string()));
                        // } else if (b == ">" || b == "<") && !is_expr {
                        //     result.push(Word::SlimInfix(b));
                    } else {
                        result.push(self.words [index].clone());
                    }
                }
                (Word::PrefixOperator(p1), 
                    Word::PrefixOperator(p2)) => {
                    if p1 == "#" && p2 == "!" {
                        result.push(Word::SlimInfix(p2));
                    } else {
                        result.push(self.words [index].clone());
                    }
                }

                _ => result.push(self.words [index].clone()),
            }
            index += 1;
        }
        self.words = Box::new(result);
    }

    // The idea is that all formating decisions can be decided with context information and one
    // token lookahead.
    // The context is encoded in the enum 'Context'. Everytime a open delimiter is encountered, a
    // new context is pushed onto the context stack. Everytime a close delimiter is encountered
    // the context get poped.
    fn format(&mut self) {
        let mut result = Vec::new();
        let mut index = 0usize;
        let mut context_stack:Vec < Context > = Vec::with_capacity(10);
        let base_context = Context::CodeBlock;

        loop {
            if index >= self.words.len() - 1 {
                break;
            }

            let word = self.words[index].clone();
            let peek = self.words[index + 1].clone();

            result.push(word.clone());

            // switch to exploded List
            if *context_stack.last().unwrap_or(&base_context) == Context::List
            && (word == Word::OpenBracket || word == Word::OpenParen)
            && peek == Word::LineBreakIntentPlus {
                context_stack.pop();
                context_stack.push(Context::ListExploded);
            }

            // insert semicolon after return
            if word == Word::Other("return".to_string()) && peek == Word::CloseBrace {
                result.push(Word::SemiColon);
            }

            match decide_whitespace(context_stack.last().unwrap_or(&base_context), &word, &peek) {
                Some(whitespace) => {
                    result.push(whitespace);
                }
                None => {}
            }

            // switch to curly list, when a codeblock seems to be not a codeblock
            if *context_stack.last().unwrap_or(&base_context) == Context::CodeBlock &&
            word == Word::Comma &&
            context_stack.len() > 0 {
                context_stack.pop();
                context_stack.push(Context::CurlyList);
            }

            // decide context changes
            match peek {
                Word::OpenBracket
                | Word::OpenParen => context_stack.push(Context::List),
                Word::OpenBrace => context_stack.push(Context::CodeBlock),
                Word::SlimInfix(s) => {
                    if s == "<" {
                        context_stack.push(Context::List)
                    } else if s == ">" {
                        context_stack.pop();
                    }
                }
                Word::CloseBrace
                | Word::CloseBracket
                | Word::CloseParen => {
                    context_stack.pop();
                }
                _ => {},
            }
            index += 1;
        }
        result.push(Word::Eof);
        self.words = Box::new(result);
    }
    //'
    fn handle_overlong_lines(&mut self) {
        let mut result = Vec::new();
        let mut index = 0usize;
        let mut column = 0i32;
        let mut intent = 0i32;
        let mut index_at_last_ws = 0usize;

        loop {
            if index >= self.words.len() {
                break;
            }

            let word = self.words[index].clone();

            intent += match word {
                Word::LineBreakIntentPlus => SPACES_PER_TAP,
                Word::LineBreakIntentMinus => - SPACES_PER_TAP,
                _ => 0,
            };
            match word {
                Word::LineBreak
                | Word::LineBreakDouble
                | Word::LineBreakIntentPlus
                | Word::LineBreakIntentMinus => column = intent,
                _ => column += word.clone().to_string().len()as i32,
            }
            match word {
                Word::Whitespace(_) => index_at_last_ws = index,
                _ => {},
            }
            if column > MAX_LINE_LENGTH
            && word.clone().to_string().len() < (MAX_LINE_LENGTH - intent)as usize {
                for _ in 0..(index - index_at_last_ws) {
                    result.pop();
                }
                index = index_at_last_ws;
                column = intent + SPACES_PER_TAP;
                result.push(Word::LineBreak);
                result.push(Word::Whitespace(SPACES_PER_TAP));
            } else {
                result.push(word.clone());
            }
            index += 1;
        }

        result.push(Word::Eof);
        self.words = Box::new(result);
    }

    pub fn to_string(&self) -> String {
        let mut formated_source = "".to_string();
        let mut intent = 0i32;

        for word in self.words.iter() {
            intent += match *word {
                Word::LineBreakIntentPlus => SPACES_PER_TAP,
                Word::LineBreakIntentMinus => - SPACES_PER_TAP,
                _ => 0,
            };
            limit(&mut intent, 0, MAX_INTENT);
            match *word {
                Word::LineBreak | Word::LineBreakIntentPlus | Word::LineBreakIntentMinus => {
                    formated_source.push_str("\n");
                    for _ in 0..intent {
                        formated_source.push_str(" ");
                    }
                }
                Word::LineBreakDouble => {
                    formated_source.push_str("\n\n");
                    for _ in 0..intent {
                        formated_source.push_str(" ");
                    }
                }
                _ => formated_source.push_str(word.clone().to_string().as_ref()),
            }
        }
        formated_source
    }
}

fn limit(var: &mut i32, low: i32, upper: i32) {
    if *var < low {
        *var = low;
    } else if *var > upper {
        *var = upper
    }
}

fn decide_whitespace(context: &Context, word: &Word, peek: &Word) ->
Option < Word > {
    match *context {
        Context::CodeBlock | Context::CurlyList => {
            match (word, peek) {
                (&Word::SlimInfix(ref s), &Word::OpenBrace) => {
                    if *s == "::" {
                        None  // foo::{bar, ...}
                    } else {
                        Some(Word::Whitespace(1))  // cases like <'a, 'b> {
                    }
                }
                (&Word::Other(ref s), &Word::OpenParen) => {
                    if *s == "if" || *s == "match" || *s == "for" || *s == "let" {
                        Some(Word::Whitespace(1))
                    } else {
                        None
                    }
                }
                (&Word::OpenBrace, 
                    &Word::CloseBrace) => None,
                (&Word::OpenBrace, _) => Some(Word::LineBreakIntentPlus),
                (_, &Word::CloseBrace) => Some(Word::LineBreakIntentMinus),
                (_, &Word::LineBreakDouble)
                | (&Word::LineBreakDouble, _)
                | (_, &Word::LineBreak)
                | (&Word::LineBreak, _) => None,
                (&Word::Other(_), &Word::Other(_))
                | (&Word::BinaryOperator(_), _)
                | (_, &Word::BinaryOperator(_))
                | (_, &Word::OpenBrace) => Some(Word::Whitespace(1)),
                (&Word::CloseBrace, &Word::Other(ref s)) => {
                    match s.as_ref() {
                        "else" => Some(Word::Whitespace(1)),
                        _ => Some(Word::LineBreak),
                    }
                }
                (&Word::SemiColon, &Word::Comment(_))
                | (&Word::Comma, &Word::Comment(_)) => Some(Word::Whitespace(2)),
                (&Word::CloseBrace, &Word::SemiColon)
                | (&Word::CloseBrace, &Word::Comma) => None,
                (&Word::SemiColon, _)
                | (&Word::CloseBrace, &Word::Comment(_))
                | (&Word::Comma, _)
                | (&Word::Comment(_), _)
                | (&Word::CloseBrace, _) => Some(Word::LineBreak),
                (_, &Word::PrefixOperator(ref p)) => {
                    match p.as_ref() {
                        "#" => Some(Word::LineBreak),
                        _ => Some(Word::Whitespace(1)),
                    }
                }
                (_, 
                    &Word::Comment(_)) => Some(Word::Whitespace(2)),
                (_, _) => None,
            }
        }
        Context::List => {
            match (word, peek) {
                (&Word::LineBreak, _)
                | (&Word::LineBreakDouble, _) => Some(Word::Whitespace(SPACES_PER_TAP)),
                (&Word::Other(_), &Word::Other(_))
                | (&Word::BinaryOperator(_), _)
                | (_, &Word::BinaryOperator(_))
                | (&Word::Comma, _)
                | (&Word::Colon, _)
                | (&Word::SemiColon, _)
                | (_, &Word::OpenBrace)
                | (_, &Word::OpenBracket) => Some(Word::Whitespace(1)),
                (_, &Word::Comment(_)) => Some(Word::Whitespace(2)),
                (_, _) => None,
            }
        }
        Context::ListExploded => {
            match (word, peek) {
                (_, &Word::CloseBracket)
                | (_, &Word::CloseParen) => Some(Word::LineBreakIntentMinus),
                (_, &Word::LineBreakDouble)
                | (&Word::LineBreakDouble, _)
                | (_, &Word::LineBreak)
                | (&Word::LineBreak, _) => None,
                (&Word::Other(_), &Word::Other(_))
                | (&Word::BinaryOperator(_), _)
                | (_, &Word::BinaryOperator(_))
                | (_, &Word::OpenBrace)
                | (_, &Word::OpenBracket) => Some(Word::Whitespace(1)),
                (&Word::SemiColon, &Word::Comment(_))
                | (&Word::Comma, &Word::Comment(_)) => Some(Word::Whitespace(2)),
                (&Word::CloseBracket, &Word::Comma) => None,
                (&Word::CloseBracket, &Word::Comment(_))
                | (&Word::Comma, _)
                | (&Word::Comment(_), _)
                | (&Word::CloseBracket, _)
                | (&Word::CloseParen, _) => Some(Word::LineBreak),
                (_, &Word::Comment(_)) => Some(Word::Whitespace(2)),
                (_, _) => None,
            }
        }
    }
}
