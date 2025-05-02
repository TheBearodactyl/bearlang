use std::collections::HashMap;
use std::fs;
use clap::Parser as ClapParser;
use pest::Parser;
use pest_derive::Parser;

#[derive(clap::Parser)]
struct Cli {
    input: String,
}

#[derive(Parser)]
#[grammar = "bearl.pest"]
pub struct BearlangParser;

fn main() {
    let argv = Cli::parse();
    let unparsed_file = fs::read_to_string(&argv.input)
        .expect("cannot read file");

    let file = BearlangParser::parse(Rule::file, &unparsed_file)
        .expect("cannot parse file")
        .next()
        .unwrap();

    let mut properties: HashMap<&str, HashMap<&str, String>> = HashMap::new();
    let mut curr_section_name = "";

    // First pass: collect all properties
    for ln in file.into_inner() {
        match ln.as_rule() {
            Rule::EOI => {}
            Rule::section => {
                let mut inner_rules = ln.into_inner();
                curr_section_name = inner_rules.next().unwrap().as_str();
            }
            Rule::property => {
                let mut inner_rules = ln.into_inner();

                let name: &str = inner_rules.next().unwrap().as_str();
                let value = inner_rules.next().unwrap();

                let section = properties.entry(curr_section_name).or_default();

                match value.as_rule() {
                    Rule::variable => {
                        let var_parts: Vec<&str> = value.as_str()[1..value.as_str().len()-1]
                            .split('.')
                            .collect();
                        let var_value = format!("({}.{})", var_parts[0], var_parts[1]);
                        section.insert(name, var_value);
                    }
                    _ => {
                        section.insert(name, value.as_str().to_string());
                    }
                }
            }
            _ => unreachable!()
        }
    }

    // Second pass: resolve variables
    let mut resolved_properties: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (section_name, section_props) in &properties {
        let mut resolved_section = HashMap::new();
        for (key, value) in section_props {
            if value.starts_with('(') && value.ends_with(')') {
                let var_parts: Vec<&str> = value[1..value.len()-1].split('.').collect();
                if var_parts.len() == 2 {
                    if let Some(var_section) = properties.get(var_parts[0]) {
                        if let Some(var_value) = var_section.get(var_parts[1]) {
                            resolved_section.insert(key.to_string(), var_value.to_string());
                            continue;
                        }
                    }
                }
            }
            resolved_section.insert(key.to_string(), value.to_string());
        }
        resolved_properties.insert(section_name.to_string(), resolved_section);
    }

    println!("{:#?}", resolved_properties);
}