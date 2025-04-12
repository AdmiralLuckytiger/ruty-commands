use std::{collections::HashMap, io, io::BufRead};

mod generator;
use crate::generator::{generate_html_tag, generate_html_template_var};

mod parser;
use crate::parser::{get_content_type, ContentType, TagType};

fn main() -> () {
    let mut context: HashMap<String, Vec<String>> = HashMap::new();

    context.insert("name".to_string(), vec!["Bob".to_string()]);
    context.insert("city".to_string(), vec!["Boston".to_string()]);

    for line in io::stdin().lock().lines() {
        match get_content_type(&line.unwrap().clone()) {
            ContentType::TemplateVariable(mut content) => {
                let html = generate_html_template_var(&mut content, &context)
                    .gen_html
                    .clone();
                println!("{}", html);
            }
            ContentType::Literal(text) => println!("{}", text),
            ContentType::Tag(TagType::ForTag(ref mut content)) => {
                let html = generate_html_tag(&mut *content, &context);
                println!("{}", html);
            }
            ContentType::Tag(TagType::IfTag(ref mut content)) => {
                let html = generate_html_tag(&mut *content, &context);
                println!("{}", html);
            }
            ContentType::Unrecognized => println!("Unrecognized input"),
        }
    }
}
