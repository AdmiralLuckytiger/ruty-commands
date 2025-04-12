/// Each line in input can be one of following types
#[derive(Debug, PartialEq)]
pub enum ContentType {
    Literal(String),
    TemplateVariable(ExpressionData),
    Tag(TagType),
    Unrecognized,
}

/// Stores the result of the tokenization of the template string
/// 1. Allow for the parsing of more than one template variable per statement
/// 2. Allow for the parsing of more than two string literals in the input statement
#[derive(Debug, PartialEq)]
pub struct ExpressionData {
    pub expression: String,
    pub var_map: Vec<String>,
    pub gen_html: String,
}

/// Stores data from valid if tag expressions
#[derive(Debug, PartialEq)]
pub struct Conditional {
    pub condition: ConditionData,
    pub expression: Box<ContentType>,
}

/// Structurates data for evaluation purpuses
#[derive(Debug, PartialEq)]
pub struct ConditionData {
    pub left_operand: String,
    pub operation: OperationType,
    pub right_operand: String,
}

/// Valid operation for for and if tags
#[derive(Debug, PartialEq)]
pub enum OperationType {
    Equal,
    In,
    Nosoported(String),
}

/// Each Tag content corresponds to a for-tag or if-tag
#[derive(Debug, PartialEq)]
pub enum TagType {
    ForTag(Box<Conditional>),
    IfTag(Box<Conditional>),
}

/// Accepts an input statement and tokenizes it into one of an if tag, a for tag, or a template varaible.
/// Entry point for parser
pub fn get_content_type(input: &str) -> ContentType {
    let is_tag_expression = check_matching_pair(&input, "{%", "%}");

    let is_for_tag = (check_symbol_string(&input, "for")) && check_symbol_string(&input, "in")
        || check_symbol_string(&input, "endfor");

    let is_if_tag = check_symbol_string(&input, "if") || check_symbol_string(&input, "endif");

    let is_template_variable = check_matching_pair(&input, "{{", "}}");

    if is_tag_expression && is_for_tag {
        let content = get_conditional_data(&input);
        return ContentType::Tag(TagType::ForTag(Box::new(
            content.expect("Should panic if it is not right"),
        )));
    } else if is_tag_expression && is_if_tag {
        let content = get_conditional_data(&input);
        return ContentType::Tag(TagType::IfTag(Box::new(
            content.expect("Should panic if it is not right"),
        )));
    } else if is_template_variable {
        let content = get_expression_data(&input);
        return ContentType::TemplateVariable(content);
    } else if !is_tag_expression && !is_template_variable {
        return ContentType::Literal(input.to_string());
    } else {
        ContentType::Unrecognized
    }
}

/// Checks if a symbol is present within another string.
fn check_symbol_string(input: &str, pattern: &str) -> bool {
    input.contains(pattern)
}

/// Verify if a statement in a template file is syntactically correct.
fn check_matching_pair(input: &str, left_part: &str, right_pat: &str) -> bool {
    let count_left_pattern = input.matches(left_part).collect::<Vec<&str>>().len();
    let count_right_pattern = input.matches(right_pat).collect::<Vec<&str>>().len();

    count_left_pattern == count_right_pattern && count_left_pattern != 0
}

#[allow(dead_code)]
/// Returns the starting index of a substring within another string.
pub fn get_index_for_symbol(input: &str, symbol: char) -> Option<usize> {
    input.find(symbol)
}

/// Parses a template string into its constituent parts for a token of type TemplateString
fn get_expression_data(input: &str) -> ExpressionData {
    let expression_iter = input.split_whitespace();
    let mut template_var_map: Vec<String> = vec![];
    for word in expression_iter {
        if check_symbol_string(word, "{{") && check_symbol_string(word, "}}") {
            template_var_map.push(word.to_string());
        }
    }

    ExpressionData {
        expression: input.into(),
        var_map: template_var_map,
        gen_html: "".into(),
    }
}

#[allow(dead_code)]
/// Gets the type of evaluation that should be validated in if or for tags
fn get_operation_type(input: &str) -> OperationType {
    match input {
        "=" => OperationType::Equal,
        "in" => OperationType::In,
        _ => OperationType::Nosoported("Unrecognized operator".to_string()),
    }
}

#[allow(dead_code)]
/// Structurate expression to be evaluated
pub fn get_conditional_expression(input: &str) -> Result<ConditionData, String> {
    // Valid operators to compare
    let operators = [">", ">=", "=", "<=", "<", "in"];

    let input = input.trim();

    for operator in operators {
        if input.contains(operator) {
            let operants: Vec<&str> = input.split(operator).collect();

            if operants.len() != 2 {
                break;
            }

            return Ok(ConditionData {
                left_operand: operants[0].trim().to_string(),
                operation: get_operation_type(operator),
                right_operand: operants[1].trim().to_string(),
            });
        }
    }

    Err("Invalid format".to_string())
}

/// Structurate for and if tag expressions
pub fn get_conditional_data(input: &str) -> Result<Conditional, String> {
    // Checks input format
    if !input.ends_with("{% endif %}") & !input.ends_with("{% endfor %}") {
        return Err("Invalid input format".to_string());
    }

    let start_condition = match input.find("{% if ") {
        Some(i) => i + 6,
        None => {
            input
                .find("{% for ")
                .expect("If not a if expression is a for expression")
                + 7
        }
    };
    let end_condition = input.find(" %}").unwrap();
    let end_expr = match input.find("{% endif %}") {
        Some(i) => i,
        None => input
            .find("{% endfor %}")
            .expect("If not a if expression is a for expression"),
    };

    if start_condition >= end_condition {
        return Err("Invalid input format".to_string());
    }

    Ok(Conditional {
        condition: get_conditional_expression(&input[start_condition..end_condition])?,
        expression: Box::new(get_content_type(&input[end_condition + 3..end_expr].trim())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_literal_test() {
        let s = "<h1>Hello world</h1>";
        assert_eq!(ContentType::Literal(s.to_string()), get_content_type(s));
    }

    #[test]
    fn check_template_var_test() {
        let content = ExpressionData {
            expression: "Hi {{name}} ,welcome".to_string(),
            var_map: vec!["{{name}}".to_string()],
            gen_html: "".into(),
        };

        assert_eq!(
            ContentType::TemplateVariable(content),
            get_content_type("Hi {{name}} ,welcome")
        );
    }

    #[test]
    fn check_for_tag_test() {
        assert_eq!(
            ContentType::Tag(TagType::ForTag(Box::new(Conditional {
                condition: ConditionData {
                    left_operand: "name".to_string(),
                    operation: OperationType::In,
                    right_operand: "names".to_string(),
                },
                expression: Box::new(ContentType::TemplateVariable(ExpressionData {
                    expression: "<p> Welcome {{name}} !! </p>".to_string(),
                    var_map: vec!["{{name}}".to_string()],
                    gen_html: "".into(),
                })),
            }))),
            get_content_type("{% for name in names %} <p> Welcome {{name}} !! </p> {% endfor %}")
        )
    }

    #[test]
    fn check_if_tag_test() {
        assert_eq!(
            ContentType::Tag(TagType::IfTag(Box::new(Conditional {
                condition: ConditionData {
                    left_operand: "name".to_string(),
                    operation: OperationType::Equal,
                    right_operand: "Bob".to_string(),
                },
                expression: Box::new(ContentType::TemplateVariable(ExpressionData {
                    expression: "<p> Welcome {{name}} </p>".to_string(),
                    var_map: vec!["{{name}}".to_string()],
                    gen_html: "".into(),
                })),
            }))),
            get_content_type("{% if name = Bob %} <p> Welcome {{name}} </p> {% endif %}")
        )
    }

    #[test]
    fn check_symbol_string_test() {
        assert_eq!(true, check_symbol_string("{{Hello}}", "{{"))
    }

    #[test]
    fn check_symbol_pair_test() {
        assert_eq!(true, check_matching_pair("{{Hello}}", "{{", "}}"))
    }

    #[test]
    fn check_get_expression_data_test() {
        let expression_data = ExpressionData {
            expression: "Hi {{name}} ,welcome".to_string(),
            var_map: vec!["{{name}}".to_string()],
            gen_html: "".into(),
        };

        assert_eq!(expression_data, get_expression_data("Hi {{name}} ,welcome"));
    }

    #[test]
    fn check_get_index_for_symbol_test() {
        assert_eq!(
            3,
            get_index_for_symbol("Hi {{name}}, welcome", '{').unwrap()
        )
    }

    #[test]
    fn check_get_operation_type_test() {
        assert_eq!(get_operation_type("in"), OperationType::In)
    }

    #[test]
    fn fail_get_operation_type_test() {
        assert_eq!(
            get_operation_type("~"),
            OperationType::Nosoported("Unrecognized operator".to_string())
        )
    }

    #[test]
    fn check_get_conditional_expression() {
        assert_eq!(
            get_conditional_expression(" amount = 2000 ").unwrap(),
            ConditionData {
                left_operand: "amount".to_string(),
                operation: OperationType::Equal,
                right_operand: "2000".to_string()
            }
        )
    }

    #[test]
    fn check_get_conditional_data() {
        assert_eq!(
            get_conditional_data("{% if amount = 2000 %} <p> hola </p> {% endif %}").unwrap(),
            Conditional {
                condition: ConditionData {
                    left_operand: "amount".to_string(),
                    operation: OperationType::Equal,
                    right_operand: "2000".to_string(),
                },
                expression: Box::new(ContentType::Literal("<p> hola </p>".to_string())),
            }
        )
    }
}
