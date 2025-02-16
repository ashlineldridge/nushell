use nu_engine::CallExt;
use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, SyntaxShape,
    Type, Value,
};
use regex::Regex;

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "split list"
    }

    fn signature(&self) -> Signature {
        Signature::build("split list")
            .input_output_types(vec![(
                Type::List(Box::new(Type::Any)),
                Type::List(Box::new(Type::List(Box::new(Type::Any)))),
            )])
            .required(
                "separator",
                SyntaxShape::Any,
                "the value that denotes what separates the list",
            )
            .switch(
                "regex", 
                "separator is a regular expression, matching values that can be coerced into a string", 
                Some('r'))
            .category(Category::Filters)
    }

    fn usage(&self) -> &str {
        "Split a list into multiple lists using a separator."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["separate", "divide", "regex"]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        split_list(engine_state, stack, call, input)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Split a list of chars into two lists",
                example: "[a, b, c, d, e, f, g] | split list d",
                result: Some(Value::list(
                    vec![
                        Value::list(
                            vec![
                                Value::test_string("a"),
                                Value::test_string("b"),
                                Value::test_string("c"),
                            ],
                            Span::test_data(),
                        ),
                        Value::list(
                            vec![
                                Value::test_string("e"),
                                Value::test_string("f"),
                                Value::test_string("g"),
                            ],
                            Span::test_data(),
                        ),
                    ],
                    Span::test_data(),
                )),
            },
            Example {
                description: "Split a list of lists into two lists of lists",
                example: "[[1,2], [2,3], [3,4]] | split list [2,3]",
                result: Some(Value::list(
                    vec![
                        Value::list(
                            vec![Value::list(
                                vec![Value::test_int(1), Value::test_int(2)],
                                Span::test_data(),
                            )],
                            Span::test_data(),
                        ),
                        Value::list(
                            vec![Value::list(
                                vec![Value::test_int(3), Value::test_int(4)],
                                Span::test_data(),
                            )],
                            Span::test_data(),
                        ),
                    ],
                    Span::test_data(),
                )),
            },
            Example {
                description: "Split a list of chars into two lists",
                example: "[a, b, c, d, a, e, f, g] | split list a",
                result: Some(Value::list(
                    vec![
                        Value::list(
                            vec![
                                Value::test_string("b"),
                                Value::test_string("c"),
                                Value::test_string("d"),
                            ],
                            Span::test_data(),
                        ),
                        Value::list(
                            vec![
                                Value::test_string("e"),
                                Value::test_string("f"),
                                Value::test_string("g"),
                            ],
                            Span::test_data(),
                        ),
                    ],
                    Span::test_data(),
                )),
            },
            Example {
                description: "Split a list of chars into lists based on multiple characters",
                example: r"[a, b, c, d, a, e, f, g] | split list -r '(b|e)'",
                result: Some(Value::list(
                    vec![
                        Value::list(vec![Value::test_string("a")], Span::test_data()),
                        Value::list(
                            vec![
                                Value::test_string("c"),
                                Value::test_string("d"),
                                Value::test_string("a"),
                            ],
                            Span::test_data(),
                        ),
                        Value::list(
                            vec![Value::test_string("f"), Value::test_string("g")],
                            Span::test_data(),
                        ),
                    ],
                    Span::test_data(),
                )),
            },
        ]
    }
}

enum Matcher {
    Regex(Regex),
    Direct(Value),
}

impl Matcher {
    pub fn new(regex: bool, lhs: Value) -> Result<Self, ShellError> {
        if regex {
            Ok(Matcher::Regex(Regex::new(&lhs.as_string()?).map_err(
                |err| {
                    ShellError::GenericError(
                        "Error with regular expression".into(),
                        err.to_string(),
                        match lhs {
                            Value::Error { .. } => None,
                            _ => Some(lhs.span()),
                        },
                        None,
                        Vec::new(),
                    )
                },
            )?))
        } else {
            Ok(Matcher::Direct(lhs))
        }
    }

    pub fn compare(&self, rhs: &Value) -> Result<bool, ShellError> {
        Ok(match self {
            Matcher::Regex(regex) => {
                if let Ok(rhs_str) = rhs.as_string() {
                    regex.is_match(&rhs_str)
                } else {
                    false
                }
            }
            Matcher::Direct(lhs) => rhs == lhs,
        })
    }
}

fn split_list(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let separator: Value = call.req(engine_state, stack, 0)?;
    let mut temp_list = Vec::new();
    let mut returned_list = Vec::new();

    let iter = input.into_interruptible_iter(engine_state.ctrlc.clone());
    let matcher = Matcher::new(call.has_flag("regex"), separator)?;
    for val in iter {
        if matcher.compare(&val)? {
            if !temp_list.is_empty() {
                returned_list.push(Value::list(temp_list.clone(), call.head));
                temp_list = Vec::new();
            }
        } else {
            temp_list.push(val);
        }
    }
    if !temp_list.is_empty() {
        returned_list.push(Value::list(temp_list.clone(), call.head));
    }
    Ok(Value::list(returned_list, call.head).into_pipeline_data())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
