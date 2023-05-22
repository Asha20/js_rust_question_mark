use std::{borrow::Cow, ops::Range};

use crate::tokenizer::Token;

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

    fn get_string_ops(&self, range: Range<usize>) -> Vec<StringOp> {
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
    pub operator: Modifier<'a>,
}

pub struct QuestionMarkOperator<'a> {
    config: Config<'a>,
}

impl<'a> QuestionMarkOperator<'a> {
    pub fn new(config: Config<'a>) -> Self {
        Self { config }
    }

    fn definition(&self) -> String {
        format!(
            r#"
            const EARLY_RETURN = Symbol();
            const __unwrap = x => {{
                if ({condition}) return {unwrap};
                throw {{ [EARLY_RETURN]: x }};
            }};
        "#,
            condition = self.config.value_check.get_string("x"),
            unwrap = self.config.unwrap.get_string("x")
        )
    }

    fn mechanism(&self, range: Range<usize>) -> Vec<StringOp> {
        vec![
            StringOp::insert(range.start, "try {"),
            StringOp::insert(
                range.end,
                "} catch (e) { if (EARLY_RETURN in e) return e[EARLY_RETURN]; throw e; }",
            ),
        ]
    }

    fn question_mark(&self, range: Range<usize>) -> Vec<StringOp> {
        self.config.operator.get_string_ops(range)
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
        modifier.add_token(&qm, token)
    }

    let result = modifier.modify(input);

    if let Cow::Owned(result) = result {
        Cow::Owned(qm.definition() + result.as_str())
    } else {
        result
    }
}
