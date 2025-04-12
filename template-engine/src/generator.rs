use crate::parser::{
    get_index_for_symbol, Conditional, ContentType, ExpressionData, OperationType, TagType,
};
use std::collections::HashMap;

/// Generates HTML code for a template var token
pub fn generate_html_template_var<'a>(
    content: &'a mut ExpressionData,
    context: &HashMap<String, Vec<String>>,
) -> &'a mut ExpressionData {
    content.gen_html = content.expression.clone();

    for var in &content.var_map {
        let i = get_index_for_symbol(&var, '{').unwrap();
        let k = get_index_for_symbol(&var, '}').unwrap();
        let var_without_braces = &var[(i + 2)..k];

        let val = &context.get(var_without_braces).unwrap()[0];

        content.gen_html = content.gen_html.replace(var, val)
    }

    content
}

/// Generates HTML code for a if or for tag tokens
pub fn generate_html_tag(
    content: &mut Conditional,
    context: &HashMap<String, Vec<String>>,
) -> String {
    let mut html = String::new();

    match &content.condition.operation {
        OperationType::Equal => {
            let right_operand: Vec<&str> = content.condition.right_operand.split(" ").collect();

            let left_operand: &Vec<String> = match context.get(&content.condition.left_operand) {
                Some(v) => v,
                None => return " ".to_string(),
            };

            if right_operand == *left_operand {
                match &mut *content.expression {
                    ContentType::Literal(text) => html.push_str(&text),
                    ContentType::Tag(tag) => match tag {
                        TagType::IfTag(data) => {
                            html.push_str(&generate_html_tag(&mut *data, context))
                        }
                        TagType::ForTag(data) => {
                            html.push_str(&generate_html_tag(&mut *data, context))
                        }
                    },
                    ContentType::TemplateVariable(data) => {
                        html.push_str(&generate_html_template_var(data, context).gen_html)
                    }
                    ContentType::Unrecognized => html.push_str(""),
                }
            }
        }
        OperationType::In => {
            let right_operand: &Vec<String> = match context.get(&content.condition.right_operand) {
                Some(v) => v,
                None => return " ".to_string(),
            };

            for element in right_operand {
                match *content.expression {
                    ContentType::Literal(ref text) => {
                        html.push_str(&text);
                    }
                    ContentType::TemplateVariable(ref mut data) => {
                        data.gen_html = data.expression.clone();
                        data.gen_html = data.gen_html.replace(&data.var_map[0], &element);

                        html.push_str(&data.gen_html);
                    }
                    _ => {}
                }
                html.push_str("\n");
            }
        }
        OperationType::Nosoported(e) => return e.to_string(),
    }

    html
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::parser::{get_conditional_data, ConditionData};

    use super::*;

    #[test]
    fn check_literals() {
        let mut context: HashMap<String, Vec<String>> = HashMap::new();

        context.insert("name".to_string(), vec!["Bob".to_string()]);
        context.insert("city".to_string(), vec!["Boston".to_string()]);

        assert_eq!(
            generate_html_template_var(
                &mut ExpressionData {
                    expression: "{{name}}".to_string(),
                    var_map: vec!["{{name}}".to_string()],
                    gen_html: "".to_string(),
                },
                &context
            )
            .gen_html,
            "Bob".to_string()
        )
    }

    #[test]
    fn check_if_tag() {
        let mut context: HashMap<String, Vec<String>> = HashMap::new();

        context.insert("name".to_string(), vec!["Bob".to_string()]);
        context.insert("city".to_string(), vec!["Boston".to_string()]);

        assert_eq!(
            get_conditional_data("{% if name = Bob %} <h1> hello Bob </h1> {% endif %}")
                .expect("Input for test"),
            Conditional {
                condition: ConditionData {
                    left_operand: "name".to_string(),
                    operation: OperationType::Equal,
                    right_operand: "Bob".to_string(),
                },
                expression: Box::new(ContentType::Literal("<h1> hello Bob </h1>".to_string()))
            }
        );

        assert_eq!(
            generate_html_tag(
                &mut get_conditional_data("{% if name = Bob %} <h1> hello Bob </h1> {% endif %}")
                    .expect("Input for test"),
                &context
            ),
            "<h1> hello Bob </h1>".to_string()
        )
    }

    #[test]
    fn check_if_tag_var() {
        let mut context: HashMap<String, Vec<String>> = HashMap::new();

        context.insert("name".to_string(), vec!["Bob".to_string()]);
        context.insert("city".to_string(), vec!["Boston".to_string()]);

        assert_eq!(
            get_conditional_data("{% if name = Bob %} <h1> hello {{name}} </h1> {% endif %}")
                .expect("Input for test"),
            Conditional {
                condition: ConditionData {
                    left_operand: "name".to_string(),
                    operation: OperationType::Equal,
                    right_operand: "Bob".to_string(),
                },
                expression: Box::new(ContentType::TemplateVariable(ExpressionData {
                    expression: "<h1> hello {{name}} </h1>".to_string(),
                    var_map: vec!["{{name}}".to_string()],
                    gen_html: "".into()
                }))
            }
        );

        assert_eq!(
            generate_html_tag(
                &mut get_conditional_data("{% if name = Bob %} <h1> hello Bob </h1> {% endif %}")
                    .expect("Input for test"),
                &context
            ),
            "<h1> hello Bob </h1>".to_string()
        )
    }

    #[test]
    fn check_for_tag_one() {
        let mut context: HashMap<String, Vec<String>> = HashMap::new();

        context.insert(
            "name".to_string(),
            vec!["Bob".to_string(), "Lisa".to_string()],
        );
        context.insert("city".to_string(), vec!["Boston".to_string()]);

        assert_eq!(
            generate_html_tag(
                &mut get_conditional_data(
                    "{% for costumer in name %} <li> {{customer}} </li> {% endfor %}"
                )
                .expect("Hardcoded input"),
                &context
            ),
            "<li> Bob </li>\n<li> Lisa </li>\n".to_string()
        )
    }
}
