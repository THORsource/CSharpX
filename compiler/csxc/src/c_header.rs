use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::language::{C_FILE_CALL_PREFIX, C_HEADER_USAGE_PREFIX};

pub(crate) fn identifier(token: &str) -> bool {
    let mut bytes = token.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn file_functions(header: &str) -> BTreeSet<String> {
    let mut declarations = String::new();
    let mut rest = header;
    while let Some((before, comment)) = rest.split_once("/*") {
        declarations.push_str(before);
        declarations.push(' ');
        rest = comment.split_once("*/").map_or("", |(_, after)| after);
    }
    declarations.push_str(rest);
    let declarations = declarations
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.split_once("//").map_or(line, |(before, _)| before))
        .collect::<Vec<_>>()
        .join(" ");
    let names = declarations
        .split(';')
        .filter_map(|declaration| {
            let tokens = declaration.split_inclusive(|ch: char| ch.is_whitespace() || "(*,)".contains(ch));
            let words: Vec<_> = tokens.flat_map(|token| {
                let token = token.trim();
                if let Some(last) = token.chars().last().filter(|ch| "(*,)".contains(*ch)) {
                    let end = token.len() - last.len_utf8();
                    [token[..end].trim(), &token[end..]]
                } else {
                    [token, ""]
                }
            }).filter(|token| !token.is_empty()).collect();
            match words.as_slice() {
                ["int", name, "(", "const", "char", "*", path, ",", "unsigned", "char",
                    "*", output, ",", "size_t", capacity, ")"]
                    if [name, path, output, capacity].into_iter().all(|token| identifier(token)) => Some(*name),
                _ => None,
            }
        }).collect::<BTreeSet<_>>();
    // The declaration buffer is temporary, so return names separately.
    names.into_iter().map(str::to_owned).collect()
}

fn scoped_entries<'a>(
    program: &'a str,
    prefix: &'static str,
) -> impl Iterator<Item = (&'a str, &'a str)> {
    program
        .split(prefix)
        .skip(1)
        .filter_map(|entry| entry.split_once("*/")?.0.split_once('|'))
}

pub(crate) fn binding_name(scope: &str, name: &str) -> String {
    format!("binding_{}_{scope}_{name}", scope.len())
}

pub(crate) struct Bridge {
    pub code: String,
    pub native: bool,
    pub managed_sources: BTreeSet<PathBuf>,
    pub managed_calls: Vec<(String, String)>,
}

pub(crate) fn make_bridge(
    program: &str,
    headers: &[&str],
    cs_sources: &[&str],
    rust_sources: &[&str],
) -> Result<Bridge, String> {
    let mut available = BTreeMap::<&str, BTreeSet<String>>::new();
    let mut managed_scopes = BTreeMap::new();
    let mut managed_sources = BTreeSet::new();
    let mut rust_scopes = BTreeMap::new();
    let mut rust_modules = BTreeMap::new();
    for (scope, name) in scoped_entries(program, C_HEADER_USAGE_PREFIX) {
        let extension = name.rsplit_once('.').map(|(_, ext)| ext);
        let managed = extension.is_some_and(|ext| ext.eq_ignore_ascii_case("cs"));
        let rust = extension.is_some_and(|ext| ext.eq_ignore_ascii_case("rs"));
        let candidates = if managed {
            cs_sources
        } else if rust {
            rust_sources
        } else {
            headers
        };
        let matches: BTreeSet<_> = candidates
            .iter()
            .map(Path::new)
            .filter(|path| {
                path.file_name()
                    .and_then(|filename| filename.to_str())
                    .is_some_and(|filename| filename.eq_ignore_ascii_case(name))
            })
            .collect();
        let path = match matches.len() {
            1 => *matches.first().expect("one matched C header"),
            0 => {
                return Err(format!(
                    "CSX1707: imported file {name} is not included in the project"
                ))
            }
            _ => {
                return Err(format!(
                    "CSX1707: imported file {name} matches multiple project files"
                ))
            }
        };
        if rust {
            if !path.is_file() {
                return Err(format!("CSX1810: Rust source {} was not found", path.display()));
            }
            let path = std::path::absolute(path)
                .map_err(|error| format!("CSX1810: invalid Rust source path: {error}"))?;
            let next_index = rust_modules.len();
            let index = *rust_modules.entry(path).or_insert(next_index);
            if rust_scopes.insert(scope, index).is_some() || managed_scopes.contains_key(scope) {
                return Err("CSX1810: use distinct aliases for multiple Rust or C# imports".into());
            }
            continue;
        }
        if managed {
            let class = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .filter(|stem| identifier(stem))
                .ok_or("CSX1801: a .cs import requires a filename that is a C# class identifier")?;
            if managed_scopes.insert(scope, class).is_some() || rust_scopes.contains_key(scope) {
                return Err(
                    "CSX1801: use distinct aliases for multiple .cs imports in the same scope"
                        .into(),
                );
            }
            managed_sources.insert(path.to_path_buf());
            continue;
        }
        let text = fs::read_to_string(path).map_err(|error| {
            format!("CSX1706: cannot read C header {}: {error}", path.display())
        })?;
        available
            .entry(scope)
            .or_default()
            .extend(file_functions(&text));
    }
    let mut native = BTreeSet::new();
    let mut code = String::new();
    let mut managed_calls = Vec::new();
    let mut used_rust_modules = BTreeSet::new();
    let calls: BTreeSet<_> = scoped_entries(program, C_FILE_CALL_PREFIX).collect();
    for (scope, name) in calls {
        let binding = binding_name(scope, name);
        if identifier(name)
            && available
                .get(scope)
                .is_some_and(|symbols| symbols.contains(name))
        {
            native.insert(name);
            code.push_str(&format!("pub fn {binding}(path: &str) -> Result<String, String> {{ round_trip(path, native_{name}) }}\n"));
        } else if let Some(index) = rust_scopes.get(scope).filter(|_| identifier(name)) {
            used_rust_modules.insert(*index);
            code.push_str(&format!(
                "pub fn {binding}(value: &str) -> Result<String, String> {{ let call: fn(&str) -> String = rust_import_{index}::r#{name}; Ok(call(value)) }}\n"
            ));
        } else if let Some(class) = managed_scopes.get(scope).filter(|_| identifier(name)) {
            let index = managed_calls.len();
            managed_calls.push(((*class).to_owned(), name.to_owned()));
            code.push_str(&format!("pub fn {binding}(value: &str) -> Result<String, String> {{ managed_call({index}, value) }}\n"));
        } else {
            let visible_name = if scope.is_empty() {
                name.to_owned()
            } else {
                format!("{scope}.{name}")
            };
            return Err(format!(
                "CSX1706: function {visible_name} has no supported declaration in its imports"
            ));
        }
    }
    for (path, index) in rust_modules {
        if used_rust_modules.contains(&index) {
            code.push_str(&format!(
                "#[path = {:?}] mod rust_import_{index};\n",
                path.to_string_lossy()
            ));
        }
    }
    if !native.is_empty() {
        code.push_str(include_str!("c_file_bridge.rs"));
    }
    if !managed_calls.is_empty() {
        code.push_str(include_str!("managed_runtime.rs"));
    }
    for name in &native {
        code.push_str(&format!(
            r#"
#[link(name = "csx_c_file_native", kind = "static")]
extern "C" {{
    #[link_name = "{name}"]
    fn native_{name}(path: *const c_char, output: *mut u8, capacity: usize) -> c_int;
}}
"#
        ));
    }
    Ok(Bridge {
        code,
        native: !native.is_empty(),
        managed_sources,
        managed_calls,
    })
}

#[cfg(test)]
mod tests {
    use super::file_functions;

    #[test]
    fn preserves_c_names_for_the_supported_signature() {
        let header = r#"
            #include <stddef.h>
            /* int IgnoreComment(const char *p, unsigned char *out, size_t n); */
            int Open_File(const char*p, unsigned char*out, size_t n);
            int read_next(
                const char *path, unsigned char *buffer, size_t max);
            int wrong(const char *path, char *buffer, size_t max);
        "#;
        let names: Vec<_> = file_functions(header).into_iter().collect();
        assert_eq!(names, ["Open_File", "read_next"]);
    }
}
