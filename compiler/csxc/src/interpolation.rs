use super::{display_expression, emit_expr, lex_with_lines, Expr, Parser, Tok, Ty};
use crate::diagnostics::{split_position, Position, SourceChars};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralPart {
    Text(String),
    Hole(String),
}

#[derive(Clone, Debug)]
pub enum Part {
    Text(String),
    Value(Expr),
}

fn escape(input: &mut SourceChars<'_>) -> Result<char, String> {
    match input.next() {
        Some('n') => Ok('\n'),
        Some('r') => Ok('\r'),
        Some('t') => Ok('\t'),
        Some('0') => Ok('\0'),
        Some('"') => Ok('"'),
        Some('\\') => Ok('\\'),
        Some(_) => Err("CSX1004: invalid escape".into()),
        None => Err("CSX1003: unfinished escape".into()),
    }
}

fn hole(input: &mut SourceChars<'_>) -> Result<String, String> {
    let mut text = String::new();
    let mut depth = 0;
    while let Some(ch) = input.next() {
        if ch == '"' {
            let verbatim = text.ends_with('@') || text.ends_with("@$");
            text.push(ch);
            loop {
                let next = input
                    .next()
                    .ok_or("CSX1702: unterminated string in interpolation")?;
                text.push(next);
                if next == '"' {
                    if verbatim && input.peek() == Some(&'"') {
                        text.push(input.next().expect("peeked quote"));
                    } else {
                        break;
                    }
                } else if next == '\\' && !verbatim {
                    text.push(
                        input
                            .next()
                            .ok_or("CSX1702: unfinished escape in interpolation")?,
                    );
                }
            }
            continue;
        }
        match ch {
            '{' => depth += 1,
            '}' if depth == 0 => return Ok(text),
            '}' => depth -= 1,
            _ => {}
        }
        text.push(ch);
    }
    Err("CSX1702: unclosed interpolation".into())
}

pub(super) fn scan(first: char, input: &mut SourceChars<'_>) -> Result<Tok, String> {
    let (mut verbatim, mut interpolated) = (first == '@', first == '$');
    if first != '"' {
        match (first, input.peek()) {
            ('@', Some('$')) => {
                input.next();
                interpolated = true;
            }
            ('$', Some('@')) => {
                input.next();
                verbatim = true;
            }
            _ => {}
        }
        if input.next() != Some('"') {
            return Err("CSX1701: expected a quote after string prefix".into());
        }
    }

    let mut parts = Vec::new();
    let mut text = String::new();
    while let Some(ch) = input.next() {
        match ch {
            '"' if verbatim && input.peek() == Some(&'"') => {
                input.next();
                text.push('"');
            }
            '"' => {
                if !interpolated {
                    return Ok(Tok::Str(text));
                }
                if !text.is_empty() {
                    parts.push(LiteralPart::Text(text));
                }
                return Ok(Tok::Interpolated(parts));
            }
            '\\' if !verbatim => text.push(escape(input)?),
            '{' | '}' if interpolated && input.peek() == Some(&ch) => {
                input.next();
                text.push(ch);
            }
            '{' if interpolated => {
                if !text.is_empty() {
                    parts.push(LiteralPart::Text(std::mem::take(&mut text)));
                }
                parts.push(LiteralPart::Hole(hole(input)?));
            }
            '}' if interpolated => return Err("CSX1702: escape a literal '}' as '}}'".into()),
            _ => text.push(ch),
        }
    }
    Err("CSX1005: unterminated string".into())
}

pub(super) fn parse(parts: Vec<LiteralPart>, position: Position) -> Result<Expr, String> {
    let mut segments = Vec::new();
    for part in parts {
        segments.push(match part {
            LiteralPart::Text(text) => Part::Text(text),
            LiteralPart::Hole(text) => {
                let (tokens, _) = lex_with_lines(&text)
                    .map_err(|error| position.annotate(split_position(&error).1))?;
                let lines = vec![position; tokens.len()];
                let mut parser = Parser::new(tokens, lines);
                let expression = parser.expr(0)?;
                if parser.peek() != &Tok::Eof {
                    return Err("CSX1703: expected a single expression inside interpolation".into());
                }
                Part::Value(expression)
            }
        });
    }
    Ok(Expr::Interpolated(segments))
}

pub(super) fn emit(parts: &[Part], vars: &BTreeMap<String, Ty>) -> Result<(String, Ty), String> {
    if parts.is_empty() {
        return Ok(("String::new()".into(), Ty::Str));
    }
    let mut values = Vec::new();
    for part in parts {
        values.push(match part {
            Part::Text(text) => format!("{text:?}"),
            Part::Value(expression) => {
                let (value, ty) = emit_expr(expression, vars)?;
                if matches!(ty, Ty::Array(_)) {
                    return Err("CSX1704: interpolate an array element or Length".into());
                }
                display_expression(value, ty)
            }
        });
    }
    Ok((
        format!(
            "format!(\"{}\", {})",
            "{}".repeat(values.len()),
            values.join(", ")
        ),
        Ty::Str,
    ))
}

#[cfg(test)]
mod tests {
    use crate::language::{compile, lex, Tok};

    #[test]
    fn verbatim_preserves_backslashes_and_doubled_quotes() {
        let tokens = lex(r###"@"C:\Temp\Reports ""Hello""""###).unwrap();
        assert_eq!(tokens[0], Tok::Str(r#"C:\Temp\Reports "Hello""#.into()));
    }

    #[test]
    fn interpolates_values_and_reports_unescaped_closing_braces() {
        let code =
            compile(r#"void Main() { int n = 2; Console.WriteLine($"{{{n + 1}}}"); }"#).unwrap();
        assert!(code.contains("format!"));
        assert!(
            compile(r#"void Main() { Console.WriteLine($"invalid }"); }"#)
                .unwrap_err()
                .contains("CSX1702")
        );
    }
}
