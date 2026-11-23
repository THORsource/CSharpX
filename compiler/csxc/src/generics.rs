//! Specialize class templates before checking the object subset.
use crate::language::{temporal::TemporalType, Tok};
use std::collections::{BTreeMap, BTreeSet};

struct Parameter {
    name: String,
    default: Option<String>,
}

struct Template {
    parameters: Vec<Parameter>,
    declaration: Vec<Tok>,
    name_index: usize,
}

pub(super) struct Specialization {
    pub name: String,
    pub bindings: Vec<(String, String, Option<String>)>,
}

fn builtin_type(name: &str) -> Option<&str> {
    match name {
        "String" => Some("string"),
        "int" | "string" | "bool" => Some(name),
        name if TemporalType::from_name(name).is_some() => Some(name),
        _ => None,
    }
}

fn is_class_modifier(token: &Tok) -> bool {
    matches!(token, Tok::Ident(name) if matches!(name.as_str(), "public" | "private" | "protected" | "abstract" | "sealed"))
}

fn parameters(tokens: &[Tok]) -> Result<(Vec<Parameter>, usize), String> {
    let mut parameters = Vec::new();
    let mut names = BTreeSet::new();
    let mut offset = 0;
    let mut optional = false;
    loop {
        let Some(Tok::Ident(name)) = tokens.get(offset) else {
            return Err("CSX2401: expected a generic parameter name".into());
        };
        if builtin_type(name).is_some() || !names.insert(name) {
            return Err(format!(
                "CSX2401: invalid or duplicate generic parameter {name}"
            ));
        }
        offset += 1;
        let default = if tokens.get(offset) == Some(&Tok::Sym('=')) {
            let Some(Tok::Ident(ty)) = tokens.get(offset + 1) else {
                return Err("CSX2401: expected a default type after '='".into());
            };
            let ty = builtin_type(ty)
                .ok_or_else(|| format!("CSX2403: unsupported default type {ty}"))?;
            offset += 2;
            optional = true;
            Some(ty.to_owned())
        } else {
            if optional {
                return Err(
                    "CSX2401: required generic parameters must precede optional ones".into(),
                );
            }
            None
        };
        parameters.push(Parameter {
            name: name.clone(),
            default,
        });
        match tokens.get(offset) {
            Some(Tok::Sym('>')) => return Ok((parameters, offset + 1)),
            Some(Tok::Sym(',')) => offset += 1,
            _ => return Err("CSX2401: expected ',' or '>' in generic parameters".into()),
        }
    }
}

fn class_end(tokens: &[Tok], body: usize) -> Result<usize, String> {
    if tokens.get(body) != Some(&Tok::Sym('{')) {
        return Err("CSX2401: generic class templates require a body without a base clause".into());
    }
    let mut depth = 1;
    for (index, token) in tokens.iter().enumerate().skip(body + 1) {
        match token {
            Tok::Sym('{') => depth += 1,
            Tok::Sym('}') => {
                depth -= 1;
                if depth == 0 {
                    return Ok(index + 1);
                }
            }
            _ => {}
        }
    }
    Err("CSX2401: unclosed generic class template".into())
}

fn take_templates(tokens: &[Tok]) -> Result<(BTreeMap<String, Template>, Vec<Tok>), String> {
    let mut templates = BTreeMap::new();
    let mut remaining = Vec::new();
    let mut offset = 0;
    while offset < tokens.len() {
        if let [Tok::Ident(class), Tok::Ident(name), Tok::Sym('<'), tail @ ..] = &tokens[offset..] {
            if class == "class" {
                let (parameters, length) = parameters(tail)?;
                let body = offset + 3 + length;
                let end = class_end(tokens, body)?;
                let modifiers_start = remaining
                    .iter()
                    .rposition(|token| !is_class_modifier(token))
                    .map_or(0, |index| index + 1);
                let mut declaration = remaining.split_off(modifiers_start);
                let name_index = declaration.len() + 1;
                declaration.extend_from_slice(&tokens[offset..offset + 2]);
                declaration.extend_from_slice(&tokens[body..end]);
                let template = Template {
                    parameters,
                    declaration,
                    name_index,
                };
                if templates.insert(name.clone(), template).is_some() {
                    return Err(format!("CSX2402: duplicate generic class {name}"));
                }
                offset = end;
                continue;
            }
        }
        remaining.push(tokens[offset].clone());
        offset += 1;
    }
    Ok((templates, remaining))
}

fn arguments<'a>(
    tokens: &'a [Tok],
    template: &'a Template,
) -> Result<(Vec<&'a str>, usize), String> {
    let mut arguments = Vec::new();
    let mut offset = 0;
    if tokens.first() != Some(&Tok::Sym('>')) {
        loop {
            let Some(Tok::Ident(ty)) = tokens.get(offset) else {
                return Err("CSX2403: expected a concrete generic argument".into());
            };
            arguments.push(
                builtin_type(ty)
                    .ok_or_else(|| format!("CSX2403: unsupported type argument {ty}"))?,
            );
            offset += 1;
            match tokens.get(offset) {
                Some(Tok::Sym('>')) => break,
                Some(Tok::Sym(',')) => offset += 1,
                _ => return Err("CSX2403: expected ',' or '>' in generic arguments".into()),
            }
        }
    }
    if arguments.len() > template.parameters.len() {
        return Err("CSX2403: too many generic arguments".into());
    }
    for parameter in &template.parameters[arguments.len()..] {
        let default = parameter
            .default
            .as_deref()
            .ok_or_else(|| format!("CSX2403: missing generic argument {}", parameter.name))?;
        arguments.push(default);
    }
    Ok((arguments, offset + 1))
}

fn instantiate(template: &Template, name: &str, arguments: &[&str]) -> Vec<Tok> {
    let mut declaration = template.declaration.clone();
    declaration[template.name_index] = Tok::Ident(name.into());
    let bindings: BTreeMap<_, _> = template
        .parameters
        .iter()
        .map(|p| p.name.as_str())
        .zip(arguments.iter().copied())
        .collect();
    // The supported object subset uses type parameters as method return types.
    for index in template.name_index + 1..declaration.len().saturating_sub(2) {
        if let [Tok::Ident(ty), Tok::Ident(_), Tok::Sym('('), ..] = &declaration[index..] {
            if let Some(argument) = bindings.get(ty.as_str()) {
                declaration[index] = Tok::Ident((*argument).into());
            }
        }
    }
    declaration
}

pub(super) fn specialize(tokens: Vec<Tok>) -> Result<(Vec<Tok>, Vec<Specialization>), String> {
    let (templates, input) = take_templates(&tokens)?;
    if templates.is_empty() {
        return Ok((tokens, Vec::new()));
    }
    let identifiers: BTreeSet<_> = tokens
        .iter()
        .filter_map(|token| match token {
            Tok::Ident(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    let mut instances = BTreeMap::new();
    let mut output = Vec::new();
    let mut offset = 0;
    while offset < input.len() {
        if let [Tok::Ident(name), Tok::Sym('<'), tail @ ..] = &input[offset..] {
            if let Some(template) = templates.get(name) {
                let (arguments, length) = arguments(tail, template)?;
                let instance = format!("CsxGeneric_{}_{name}_{}", name.len(), arguments.join("_"));
                if identifiers.contains(instance.as_str()) {
                    return Err(format!(
                        "CSX2404: name reserved for generic specialization: {instance}"
                    ));
                }
                instances.entry(instance.clone()).or_insert_with(|| {
                    let declaration = instantiate(template, &instance, &arguments);
                    let bindings = template
                        .parameters
                        .iter()
                        .zip(arguments)
                        .map(|(parameter, argument)| {
                            (
                                parameter.name.clone(),
                                argument.into(),
                                parameter.default.clone(),
                            )
                        })
                        .collect();
                    (
                        declaration,
                        Specialization {
                            name: name.clone(),
                            bindings,
                        },
                    )
                });
                output.push(Tok::Ident(instance));
                offset += 2 + length;
                continue;
            }
        }
        output.push(input[offset].clone());
        offset += 1;
    }
    output.pop(); // Move EOF past the specialized class declarations.
    let mut report = Vec::new();
    for (declaration, specialization) in instances.into_values() {
        output.extend(declaration);
        report.push(specialization);
    }
    output.push(Tok::Eof);
    Ok((output, report))
}
