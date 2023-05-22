use std::{borrow::Cow, ops::Range};

use crate::tokenizer::Token;

const MANGLE_ALPHABET: [char; 62] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9',
];

const EARLY_RETURN_SYMBOL_NAME: &str = "EARLY_RETURN";
const QUESTION_MARK_FUNC_NAME: &str = "__unwrap";

pub enum Modifier<'a> {
    Function(Cow<'a, str>),
    Property(Cow<'a, str>),
    Method(Cow<'a, str>),
}
impl<'a> Modifier<'a> {
    pub fn function<I: Into<Cow<'a, str>>>(s: I) -> Self {
        Self::Function(s.into())
    }

    pub fn property<I: Into<Cow<'a, str>>>(s: I) -> Self {
        Self::Property(s.into())
    }

    pub fn method<I: Into<Cow<'a, str>>>(s: I) -> Self {
        Self::Method(s.into())
    }

    fn get_string(&self, s: &str) -> String {
        match self {
            Self::Function(func) => format!("{func}({s})"),
            Self::Property(prop) => format!("{s}.{prop}"),
            Self::Method(method) => format!("{s}.{method}()"),
        }
    }

    fn get_string_ops(&self, range: Range<usize>) -> Vec<StringOp<'static>> {
        match self {
            Self::Function(func) => vec![
                StringOp::insert(range.start, format!("{func}(")),
                StringOp::insert(range.end, ")"),
            ],
            Self::Property(prop) => vec![StringOp::insert(range.end, format!(".{prop}"))],
            Self::Method(method) => vec![StringOp::insert(range.end, format!(".{method}()"))],
        }
    }
}

pub struct Config<'a> {
    pub value_check: Modifier<'a>,
    pub unwrap: Modifier<'a>,
    pub mangle: bool,
}

pub struct QuestionMarkOperator<'a> {
    config: Config<'a>,
    mangle_suffix: Option<String>,
}

impl<'a> QuestionMarkOperator<'a> {
    pub fn new(config: Config<'a>) -> Self {
        let mangle_suffix = if config.mangle {
            Some(nanoid::nanoid!(8, &MANGLE_ALPHABET))
        } else {
            None
        };

        Self {
            config,
            mangle_suffix,
        }
    }

    fn symbol_name(&self) -> Cow<'static, str> {
        if let Some(suffix) = &self.mangle_suffix {
            Cow::Owned(format!("{EARLY_RETURN_SYMBOL_NAME}_{}", suffix))
        } else {
            Cow::Borrowed(EARLY_RETURN_SYMBOL_NAME)
        }
    }

    fn unwrap_name(&self) -> Cow<'static, str> {
        if let Some(suffix) = &self.mangle_suffix {
            Cow::Owned(format!("{QUESTION_MARK_FUNC_NAME}_{}", suffix))
        } else {
            Cow::Borrowed(QUESTION_MARK_FUNC_NAME)
        }
    }

    fn definition(&self) -> String {
        format!(
            r#"
            const {symbol} = Symbol();
            const {unwrap_fn} = x => {{
                if ({condition}) return {unwrapped};
                throw {{ [{symbol}]: x }};
            }};
        "#,
            symbol = self.symbol_name(),
            unwrap_fn = self.unwrap_name(),
            condition = self.config.value_check.get_string("x"),
            unwrapped = self.config.unwrap.get_string("x"),
        )
    }

    fn mechanism(&self, range: Range<usize>) -> Vec<StringOp> {
        vec![
            StringOp::insert(range.start, "try {"),
            StringOp::insert(
                range.end,
                format!(
                    "}} catch (e) {{ if ({symbol} in e) return e[{symbol}]; throw e; }}",
                    symbol = self.symbol_name()
                ),
            ),
        ]
    }

    fn question_mark(&self, range: Range<usize>) -> Vec<StringOp<'static>> {
        let modifier = Modifier::Function(self.unwrap_name());
        modifier.get_string_ops(range)
    }
}

#[derive(Clone, Debug)]
enum StringOp<'a> {
    Insert(usize, Cow<'a, str>),
    Delete(Range<usize>),
}

impl<'a> StringOp<'a> {
    fn insert<I: Into<Cow<'a, str>>>(index: usize, s: I) -> Self {
        Self::Insert(index, s.into())
    }

    fn start(&self) -> usize {
        match self {
            Self::Insert(i, _) => *i,
            Self::Delete(range) => range.start,
        }
    }
}

struct StringModifier<'a> {
    ops: Vec<StringOp<'a>>,
    op_size: isize,
}

impl<'a> StringModifier<'a> {
    fn new() -> Self {
        Self {
            ops: Vec::new(),
            op_size: 0,
        }
    }

    fn add_operator(&mut self, op: StringOp<'a>) {
        let offset = match &op {
            StringOp::Insert(_, s) => s.len() as isize,
            StringOp::Delete(range) => range.start as isize - range.end as isize,
        };

        self.ops.push(op);
        self.op_size += offset;
    }

    fn add_token(&mut self, qm: &'a QuestionMarkOperator, token: Token) {
        match token {
            Token::PropertyAccess {
                object,
                dot,
                property,
            } => {
                for op in qm.question_mark(object) {
                    self.add_operator(op);
                }
                self.add_operator(StringOp::Delete(dot));
                self.add_operator(StringOp::Delete(property));
            }
            Token::Function { body } => {
                for op in qm.mechanism(body) {
                    self.add_operator(op);
                }
            }
        }
    }

    fn modify(mut self, source: &str) -> Cow<str> {
        if self.ops.is_empty() {
            return Cow::Borrowed(source);
        }

        let mut result = String::with_capacity((source.len() as isize + self.op_size) as usize);
        self.ops.sort_unstable_by_key(|x| x.start());

        let mut current = 0;
        for op in self.ops {
            match op {
                StringOp::Insert(index, s) => {
                    result.push_str(&source[current..index]);
                    result.push_str(&s);
                    current = index;
                }
                StringOp::Delete(range) => {
                    result.push_str(&source[current..range.start]);
                    current = range.end;
                }
            };
        }
        result.push_str(&source[current..]);

        Cow::Owned(result)
    }
}

pub fn process<'a>(input: &'a str, qm: &QuestionMarkOperator, tokens: Vec<Token>) -> Cow<'a, str> {
    let mut modifier = StringModifier::new();
    for token in tokens {
        modifier.add_token(qm, token)
    }

    let result = modifier.modify(input);

    if let Cow::Owned(result) = result {
        Cow::Owned(qm.definition() + result.as_str())
    } else {
        result
    }
}
