//! Experimental object subset: single inheritance, virtual dispatch, and conservative
//! whole-method ownership classification. No unsafe Rust is emitted.
#[path = "generics.rs"]
mod generics;

use crate::language::temporal::{append_runtime, TemporalType};
use crate::language::{arrays, strings};
use crate::language::{
    append_c_imports, emit_constant_expression, emit_scalar_statement, emit_write_line, lex,
    read_file_import, scalar_statement_length, CHeaderUsage, Tok, Ty,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ReturnType {
    Int,
    String,
    Bool,
    Temporal(TemporalType),
}
impl ReturnType {
    fn rust_type(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Temporal(TemporalType::DateOnly) => "Result<csx_temporal::Date, String>",
            Self::Temporal(TemporalType::TimeOnly) => "Result<csx_temporal::Time, String>",
            Self::Temporal(TemporalType::DateTime) => "Result<csx_temporal::DateTime, String>",
            Self::Temporal(TemporalType::TimeSpan) => "Result<csx_temporal::Span, String>",
            Self::Int => "i64",
            Self::String => "&'static str",
        }
    }
}
#[derive(Clone, Debug)]
struct Method {
    name: String,
    virtual_: bool,
    override_: bool,
    ty: ReturnType,
    value: Option<String>,
}
#[derive(Clone, Debug)]
struct Class {
    scope: Scope,
    name: String,
    base: Option<String>,
    methods: Vec<Method>,
    abstract_: bool,
}
#[derive(Clone, Debug)]
enum Action {
    Scalar(Vec<Tok>),
    New {
        ty: String,
        name: String,
        concrete: String,
    },
    Alias {
        ty: String,
        name: String,
        from: String,
    },
    Print {
        receiver: String,
        method: String,
        tokens: Vec<Tok>,
    },
}
pub(crate) const FILE_BOUNDARY: Tok = Tok::Sym('\0');

#[derive(Clone, Debug, Default)]
struct Scope {
    namespace: String,
    imports: Vec<String>,
    file_imports: Vec<CHeaderUsage>,
    aliases: BTreeMap<String, String>,
}

impl Scope {
    fn qualify(&self, name: &str) -> String {
        if self.namespace.is_empty() { name.to_owned() } else { format!("{}.{name}", self.namespace) }
    }

    fn resolve(&self, name: &str, types: &BTreeMap<String, String>) -> Result<Option<String>, String> {
        let (head, tail) = name.split_once('.').map_or((name, None), |(head, tail)| (head, Some(tail)));
        if let Some(target) = self.aliases.get(head) {
            let full_name = tail.map_or_else(|| target.clone(), |tail| format!("{target}.{tail}"));
            return types.get(&full_name).cloned().map(Some).ok_or_else(|| {
                format!("CSX2212: alias {head} does not resolve to a class at {full_name}")
            });
        }
        let mut namespace = self.namespace.as_str();
        while !namespace.is_empty() {
            if let Some(found) = types.get(&format!("{namespace}.{name}")) {
                return Ok(Some(found.clone()));
            }
            namespace = namespace.rsplit_once('.').map_or("", |(parent, _)| parent);
        }
        if let Some(found) = types.get(name) {
            return Ok(Some(found.clone()));
        }
        let mut matches = self.imports.iter().filter_map(|prefix| types.get(&format!("{prefix}.{name}")));
        let first = matches.next();
        if matches.any(|other| Some(other) != first) {
            return Err(format!("CSX2210: ambiguous type {name}"));
        }
        Ok(first.cloned())
    }
}

struct P {
    tokens: Vec<Tok>,
    i: usize,
}
impl P {
    fn peek(&self) -> &Tok {
        &self.tokens[self.i]
    }
    fn next(&mut self) -> Tok {
        let t = self.peek().clone();
        if t != Tok::Eof {
            self.i += 1
        }
        t
    }
    fn word(&mut self, s: &str) -> bool {
        if *self.peek() == Tok::Ident(s.into()) {
            self.next();
            true
        } else {
            false
        }
    }
    fn sym(&mut self, c: char) -> bool {
        if *self.peek() == Tok::Sym(c) {
            self.next();
            true
        } else {
            false
        }
    }
    fn need(&mut self, c: char) -> Result<(), String> {
        if self.sym(c) {
            Ok(())
        } else {
            Err(format!("CSX2101: expected '{c}', got {:?}", self.peek()))
        }
    }
    fn ident(&mut self) -> Result<String, String> {
        match self.next() {
            Tok::Ident(s) => Ok(s),
            t => Err(format!("CSX2102: expected identifier, got {t:?}")),
        }
    }
    fn qualified_name(&mut self) -> Result<String, String> {
        let mut name = self.ident()?;
        while self.sym('.') {
            name.push('.');
            name.push_str(&self.ident()?);
        }
        Ok(name)
    }
    fn expect_word(&mut self, s: &str) -> Result<(), String> {
        if self.word(s) {
            Ok(())
        } else {
            Err(format!("CSX2103: expected {s}"))
        }
    }
    fn modifiers(&mut self) -> (bool, bool, bool) {
        let (mut v, mut o, mut a) = (false, false, false);
        loop {
            if self.word("public")
                || self.word("private")
                || self.word("protected")
                || self.word("static")
                || self.word("sealed")
            {
            } else if self.word("abstract") {
                a = true
            } else if self.word("virtual") {
                v = true
            } else if self.word("override") {
                o = true
            } else {
                break;
            }
        }
        (v, o, a)
    }
    fn class(&mut self, abstract_: bool, scope: Scope) -> Result<Class, String> {
        let name = scope.qualify(&self.ident()?);
        let base = if self.sym(':') {
            Some(self.qualified_name()?)
        } else {
            None
        };
        self.need('{')?;
        let mut methods = vec![];
        while !self.sym('}') {
            let (virtual_, override_, abstract_method) = self.modifiers();
            let ty = match self.ident()?.as_str() {
                "int" => ReturnType::Int,
                "string" => ReturnType::String,
                "bool" => ReturnType::Bool,
                name => ReturnType::Temporal(
                    TemporalType::from_name(name)
                        .ok_or_else(|| format!("CSX2104: unsupported method return type {name}"))?,
                ),
            };
            let method = self.ident()?;
            self.need('(')?;
            self.need(')')?;
            let value = if abstract_method {
                if !abstract_ {
                    return Err("CSX2108: abstract method requires an abstract class".into());
                }
                self.need(';')?;
                None
            } else {
                self.need('{')?;
                self.expect_word("return")?;
                let value = if let ReturnType::Temporal(temporal) = ty {
                    let start = self.i;
                    while self.peek() != &Tok::Sym(';') {
                        if self.next() == Tok::Eof {
                            return Err("CSX2109: expected end of return statement".into());
                        }
                    }
                    let code = emit_constant_expression(
                        self.tokens[start..self.i].to_vec(),
                        Ty::Temporal(temporal),
                    )?;
                    format!("Ok({code})")
                } else {
                    match (ty, self.next()) {
                        (ReturnType::Int, Tok::Num(n)) => format!("{n}i64"),
                        (ReturnType::String, Tok::Str(s)) => format!("{s:?}"),
                        (ReturnType::Bool, Tok::Ident(value))
                            if matches!(value.as_str(), "true" | "false") =>
                        {
                            value
                        }
                        _ => {
                            return Err(
                                "CSX2104: method must return a literal of its declared type".into(),
                            )
                        }
                    }
                };
                self.need(';')?;
                self.need('}')?;
                Some(value)
            };
            methods.push(Method {
                name: method,
                virtual_: virtual_ || abstract_method,
                override_,
                ty,
                value,
            });
        }
        Ok(Class {
            scope,
            name,
            base,
            methods,
            abstract_,
        })
    }
    fn main(&mut self, imports: &[CHeaderUsage]) -> Result<Vec<Action>, String> {
        self.need('(')?;
        self.need(')')?;
        self.need('{')?;
        let mut actions = vec![];
        while !self.sym('}') {
            let remaining = &self.tokens[self.i..];
            if matches!(remaining,[Tok::Ident(console),Tok::Sym('.'),Tok::Ident(write),Tok::Sym('('),Tok::Ident(receiver),Tok::Sym('.'),Tok::Ident(_),Tok::Sym('('),Tok::Sym(')'),Tok::Sym(')'),..] if console=="Console" && write=="WriteLine" && receiver!="Enumerable")
            {
                let start = self.i;
                self.i += 4;
                let receiver = self.ident()?;
                self.need('.')?;
                let method = self.ident()?;
                self.need('(')?;
                self.need(')')?;
                self.need(')')?;
                self.need(';')?;
                actions.push(Action::Print {
                    receiver,
                    method,
                    tokens: self.tokens[start..self.i].to_vec(),
                });
            } else if matches!(remaining,[Tok::Ident(word),..] if (matches!(word.as_str(),"int"|"string"|"String"|"System"|"bool"|"Console") || TemporalType::from_name(word).is_some()))
                || matches!(remaining, [Tok::Ident(word), ..] if matches!(word.as_str(), "if" | "while" | "unsafe" | "try" | "throw"))
                || matches!(remaining, [Tok::Ident(_), Tok::Sym('='), ..])
                || matches!(remaining, [Tok::Ident(_), Tok::Sym('['), ..])
                || matches!(remaining, [Tok::Ident(word), Tok::Ident(_), Tok::Sym('='), Tok::Str(_) | Tok::Interpolated(_), ..] if word == "var")
                || matches!(remaining, [Tok::Ident(word), Tok::Ident(_), Tok::Sym('='), Tok::Ident(keyword), rest @ ..] if word == "var" && keyword == "new" && matches!(rest, [Tok::Sym('['), ..] | [Tok::Ident(_), Tok::Sym('['), ..]))
                || matches!(remaining, [Tok::Ident(word), Tok::Ident(_), Tok::Sym('='), Tok::Ident(_), Tok::Sym('.'), ..] if word == "var")
                || matches!(remaining, [Tok::Ident(word), Tok::Ident(_), Tok::Sym('='), Tok::Ident(keyword), Tok::Ident(name), ..] if word == "var" && keyword == "new" && TemporalType::from_name(name).is_some())
            {
                let start = self.i;
                self.i += scalar_statement_length(remaining, imports)?;
                actions.push(Action::Scalar(self.tokens[start..self.i].to_vec()));
            } else {
                let ty = if self.word("var") {
                    String::new()
                } else {
                    self.qualified_name()?
                };
                let name = self.ident()?;
                self.need('=')?;
                if self.word("new") {
                    let concrete = self.qualified_name()?;
                    self.need('(')?;
                    self.need(')')?;
                    self.need(';')?;
                    actions.push(Action::New { ty, name, concrete });
                } else {
                    let from = self.ident()?;
                    self.need(';')?;
                    actions.push(Action::Alias { ty, name, from });
                }
            }
        }
        Ok(actions)
    }
    fn entry_class(&self) -> bool {
        let [Tok::Ident(_), Tok::Sym('{'), members @ ..] = &self.tokens[self.i..] else {
            return false;
        };
        let mut static_method = false;
        let mut members = members;
        while let [Tok::Ident(modifier), rest @ ..] = members {
            match modifier.as_str() {
                "static" => static_method = true,
                "public" | "private" | "protected" => {}
                _ => break,
            }
            members = rest;
        }
        static_method
            && matches!(members, [Tok::Ident(kind), Tok::Ident(name), ..] if kind == "void" && name == "Main")
    }

    fn program(&mut self, startup: Option<&str>) -> Result<(Vec<Class>, Vec<Action>, Scope), String> {
        let mut classes = vec![];
        let mut entries = Vec::new();
        let mut entry_classes = BTreeSet::new();
        let mut scope = Scope::default();
        let mut outer = Vec::new();
        while self.peek() != &Tok::Eof {
            if self.peek() == &FILE_BOUNDARY {
                if !outer.is_empty() { return Err("CSX2209: namespace crosses a file boundary".into()); }
                self.next();
                scope = Scope::default();
                continue;
            }
            if self.sym('}') {
                scope = outer.pop().ok_or("CSX2209: unexpected namespace closing brace")?;
                continue;
            }
            if self.word("namespace") {
                let namespace = scope.qualify(&self.qualified_name()?);
                if self.sym('{') {
                    outer.push(scope.clone());
                } else {
                    if !outer.is_empty() || !scope.namespace.is_empty() {
                        return Err("CSX2209: file-scoped namespace must be at file level".into());
                    }
                    self.need(';')?;
                }
                scope.namespace = namespace;
                continue;
            }
            if self.word("using") {
                if matches!(self.peek(), Tok::Str(_)) {
                    self.i += read_file_import(&self.tokens[self.i..], &mut scope.file_imports)?;
                    continue;
                }
                let name = self.qualified_name()?;
                if self.sym('=') {
                    if !safe_ident(&name) {
                        return Err("CSX2212: expected a single identifier as alias".into());
                    }
                    let target = self.qualified_name()?;
                    if scope.aliases.insert(name.clone(), target).is_some() {
                        return Err(format!("CSX2212: duplicate using alias {name}"));
                    }
                } else {
                    scope.imports.push(name);
                }
                self.need(';')?;
                continue;
            }
            let (_, _, abstract_) = self.modifiers();
            if self.word("class") {
                if !abstract_ && self.entry_class() {
                    let name = scope.qualify(&self.ident()?);
                    if !entry_classes.insert(name.clone()) {
                        return Err(format!("CSX2201: duplicate class {name}"));
                    }
                    self.need('{')?;
                    self.modifiers();
                    self.expect_word("void")?;
                    self.expect_word("Main")?;
                    entries.push((Some(name), self.main(&scope.file_imports)?, scope.clone()));
                    self.need('}')?;
                } else {
                    classes.push(self.class(abstract_, scope.clone())?);
                }
            } else if self.word("void") {
                self.expect_word("Main")?;
                entries.push((None, self.main(&scope.file_imports)?, scope.clone()));
            } else {
                return Err(format!(
                    "CSX2106: expected class or void Main, got {:?}",
                    self.peek()
                ));
            }
        }
        if !outer.is_empty() {
            return Err("CSX2209: unclosed namespace".into());
        }
        if let Some(class) = classes.iter().find(|class| entry_classes.contains(&class.name)) {
            return Err(format!("CSX2201: duplicate class {}", class.name));
        }
        let (actions, action_scope) = if let Some(name) = startup {
            entries.into_iter().find(|(entry, _, _)| entry.as_deref() == Some(name))
                .map(|(_, actions, scope)| (actions, scope))
                .ok_or_else(|| format!("CSX2112: startup object {name} has no supported static Main"))?
        } else {
            match entries.len() {
                0 => return Err("CSX2107: Main not found".into()),
                1 => {
                    let (_, actions, scope) = entries.pop().expect("one entry point");
                    (actions, scope)
                }
                _ => return Err("CSX2105: multiple Main methods; set the StartupObject project property".into()),
            }
        };
        Ok((classes, actions, action_scope))
    }
}
fn ancestor(classes: &BTreeMap<String, Class>, child: &str, parent: &str) -> bool {
    let mut cur = Some(child.to_string());
    let mut visited = BTreeSet::new();
    while let Some(c) = cur {
        if !visited.insert(c.clone()) {
            return false;
        }
        if c == parent {
            return true;
        }
        cur = classes.get(&c).and_then(|x| x.base.clone())
    }
    false
}
fn find_method<'a>(
    classes: &'a BTreeMap<String, Class>,
    ty: &str,
    name: &str,
) -> Option<(&'a str, &'a Method)> {
    let mut cur = Some(ty.to_string());
    while let Some(c) = cur {
        let cls = classes.get(&c)?;
        if let Some(m) = cls.methods.iter().find(|m| m.name == name) {
            return Some((classes.get_key_value(&c)?.0.as_str(), m));
        }
        cur = cls.base.clone()
    }
    None
}
fn getter_type(
    classes: &BTreeMap<String, Class>,
    class: &str,
    dispatch: &BTreeMap<(&str, ReturnType), String>,
    borrowed: Option<&crate::abi::Contract>,
) -> Ty {
    if let Some(contract) = borrowed.filter(|contract| contract.class == class) {
        return Ty::Object(class.to_owned(), contract.fields.iter().enumerate().map(|(index, (name, ty))| {
            let result = if *ty == crate::abi::Scalar::Bool { Ty::Bool } else { Ty::Int };
            (name.to_owned(), (format!("get_{index}"), result))
        }).collect());
    }
    let methods = dispatch.iter().filter_map(|(&(name, ty), lowered)| {
        let (_, method) = find_method(classes, class, name)?;
        if method.ty != ty { return None; }
        let result = match ty {
            ReturnType::Int => Ty::Int,
            ReturnType::String => Ty::Str,
            ReturnType::Bool => Ty::Bool,
            ReturnType::Temporal(temporal) => Ty::Temporal(temporal),
        };
        Some((name.to_owned(), (lowered.clone(), result)))
    }).collect();
    Ty::Object(class.to_owned(), methods)
}

fn safe_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.starts_with(|c: char| c.is_ascii_digit())
}
pub fn compile(src: &str) -> Result<(String, String), String> {
    compile_tokens(lex(src)?, None)
}
pub fn compile_tokens(tokens: Vec<Tok>, startup: Option<&str>) -> Result<(String, String), String> {
    compile_tokens_with_abi(tokens, startup, None)
}

pub(crate) fn abi_class(source: &str) -> Result<(String, BTreeMap<String, crate::abi::Scalar>), String> {
    use crate::abi::Scalar;

    let mut parser = P { tokens: lex(source)?, i: 0 };
    let (_, _, abstract_) = parser.modifiers();
    parser.expect_word("class")?;
    let class = parser.class(abstract_, Scope::default())?;
    if abstract_ || class.base.is_some() || parser.peek() != &Tok::Eof {
        return Err("CSX2501: ABI configuration must be a single concrete class without a base class".into());
    }
    let mut fields = BTreeMap::new();
    for method in class.methods {
        let ty = match method.ty {
            ReturnType::Int => Scalar::Int,
            ReturnType::Bool => Scalar::Bool,
            _ => return Err("CSX2501: configuration getters must return int or bool".into()),
        };
        if method.virtual_ || method.override_ || fields.insert(method.name, ty).is_some() {
            return Err("CSX2501: configuration getters must be distinct and nonvirtual".into());
        }
    }
    Ok((class.name, fields))
}

pub(crate) fn compile_tokens_with_abi(
    tokens: Vec<Tok>,
    startup: Option<&str>,
    abi: Option<&crate::abi::Contract>,
) -> Result<(String, String), String> {
    let namespaced = tokens.iter().any(|token| matches!(token, Tok::Ident(word) if word == "namespace"));
    let (tokens, specializations) = if namespaced {
        (tokens, Vec::new())
    } else {
        generics::specialize(tokens)?
    };
    let mut p = P { tokens, i: 0 };
    let (mut defs, mut actions, action_scope) = p.program(startup)?;
    let mut types = BTreeMap::new();
    let mut used = BTreeSet::new();
    for class in &defs {
        if !class.name.contains('.') { used.insert(class.name.clone()); }
    }
    let mut index = 0;
    for class in &defs {
        let name = &class.name;
        let lowered = if name.contains('.') {
            loop {
                let candidate = format!("CsxNamespace{index}");
                index += 1;
                if used.insert(candidate.clone()) { break candidate; }
            }
        } else {
            name.clone()
        };
        if !name.split('.').all(safe_ident) || types.insert(name.clone(), lowered).is_some() {
            return Err(format!("CSX2201: duplicate or invalid class {name}"));
        }
    }
    let resolve = |scope: &Scope, name: &mut String| -> Result<(), String> {
        if let Some(resolved) = scope.resolve(name, &types)? { *name = resolved; }
        Ok(())
    };
    for action in &mut actions {
        match action {
            Action::New { ty, concrete, .. } => {
                resolve(&action_scope, ty)?;
                resolve(&action_scope, concrete)?;
            }
            Action::Alias { ty, .. } => resolve(&action_scope, ty)?,
            _ => {}
        }
    }
    let mut classes = BTreeMap::new();
    for mut class in defs.drain(..) {
        if let Some(base) = &mut class.base { resolve(&class.scope, base)?; }
        class.name = types[&class.name].clone();
        classes.insert(class.name.clone(), class);
    }
    for c in classes.values() {
        if let Some(b) = &c.base {
            if !classes.contains_key(b) {
                return Err(format!("CSX2202: unknown base class {b}"));
            }
            if ancestor(&classes, b, &c.name) {
                return Err("CSX2203: inheritance cycle".into());
            }
        }
        let mut names = BTreeSet::new();
        for m in &c.methods {
            if !names.insert(&m.name) {
                return Err("CSX2204: duplicate method".into());
            }
            let inherited = c
                .base
                .as_ref()
                .and_then(|b| find_method(&classes, b, &m.name));
            let method_type = m.ty;
            if m.override_
                && !inherited
                    .map(|(_, m)| (m.virtual_ || m.override_) && m.ty == method_type)
                    .unwrap_or(false)
            {
                return Err(format!("CSX2205: invalid override {}.{}", c.name, m.name));
            }
            if inherited.is_some() && !m.override_ {
                return Err(format!(
                    "CSX2206: method hiding is not implemented: {}.{}",
                    c.name, m.name
                ));
            }
        }
    }
    for class in classes.values().filter(|class| !class.abstract_) {
        for base in classes
            .values()
            .filter(|base| ancestor(&classes, &class.name, &base.name))
        {
            for method in base.methods.iter().filter(|method| method.value.is_none()) {
                if find_method(&classes, &class.name, &method.name)
                    .and_then(|(_, m)| m.value.as_ref())
                    .is_none()
                {
                    return Err(format!(
                        "CSX2207: {} must implement {}",
                        class.name, method.name
                    ));
                }
            }
        }
    }
    let borrowed = abi.filter(|contract| contract.borrowed);
    let borrowed_values = if let Some(contract) = borrowed {
        let class = classes.get(&contract.class)
            .ok_or("CSX2513: borrowed DLL configuration class is missing")?;
        if class.abstract_ || class.base.is_some()
            || classes.values().any(|derived| derived.name != class.name && ancestor(&classes, &derived.name, &class.name))
        {
            return Err("CSX2513: borrowed configuration classes cannot use inheritance".into());
        }
        let mut members = BTreeMap::new();
        for method in &class.methods {
            let ty = match method.ty {
                ReturnType::Int => crate::abi::Scalar::Int,
                ReturnType::Bool => crate::abi::Scalar::Bool,
                _ => return Err("CSX2513: borrowed configuration getters must return int or bool".into()),
            };
            let value = method.value.as_ref().filter(|_| !method.virtual_ && !method.override_)
                .ok_or("CSX2513: borrowed configuration getters must be nonvirtual and immutable")?;
            members.insert(method.name.as_str(), (ty, value));
        }
        if members.len() != contract.fields.len() || contract.fields.iter().any(|(name, ty)| {
            members.get(name.as_str()).is_none_or(|(actual, _)| actual != ty)
        }) {
            return Err("CSX2504: configuration class does not match the borrowed DLL contract".into());
        }
        Some(members.values().map(|(ty, value)| {
            if *ty == crate::abi::Scalar::Bool { format!("i64::from({value})") } else { (*value).clone() }
        }).collect::<Vec<_>>().join(", "))
    } else {
        None
    };
    // The variable environment tracks the declared static type and the originating allocation.
    let mut vars: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut allocation_aliases: BTreeMap<String, usize> = BTreeMap::new();
    for a in &actions {
        match a {
            Action::Scalar(_) => {}
            Action::New { ty, name, concrete } => {
                if vars.contains_key(name) {
                    return Err(format!("CSX2301: duplicate variable {name}"));
                }
                if !classes.contains_key(concrete) {
                    return Err(format!("CSX2302: unknown class {concrete}"));
                }
                if classes[concrete].abstract_ {
                    return Err(format!(
                        "CSX2306: cannot instantiate abstract class {concrete}"
                    ));
                }
                let static_ty = if ty.is_empty() { concrete } else { ty };
                if !ancestor(&classes, concrete, static_ty) {
                    return Err(format!("CSX2303: cannot convert {concrete} to {static_ty}"));
                }
                vars.insert(name.clone(), (static_ty.clone(), name.clone()));
                allocation_aliases.insert(name.clone(), 1);
            }
            Action::Alias { ty, name, from } => {
                if vars.contains_key(name) {
                    return Err(format!("CSX2301: duplicate variable {name}"));
                }
                let (from_ty, root) = vars
                    .get(from)
                    .ok_or_else(|| format!("CSX2304: unknown variable {from}"))?
                    .clone();
                let static_ty = if ty.is_empty() { &from_ty } else { ty };
                if !ancestor(&classes, &from_ty, static_ty) {
                    return Err(format!("CSX2303: cannot convert {from_ty} to {static_ty}"));
                }
                *allocation_aliases.get_mut(&root).unwrap() += 1;
                vars.insert(name.clone(), (static_ty.clone(), root));
            }
            Action::Print {
                receiver, method, ..
            } => {
                if let Some((ty, _)) = vars.get(receiver) {
                    if find_method(&classes, ty, method).is_none() {
                        return Err(format!("CSX2305: method {ty}.{method} not found"));
                    }
                }
            }
        }
    }
    // Object representation is a safe tagged enum; no raw vtables or unsafe casts.
    let mut rust = String::from(
        "#![forbid(unsafe_code)]\nuse std::rc::Rc;\n#[derive(Debug)]\nenum CsxObject {\n",
    );
    for name in classes.keys() {
        rust.push_str(&format!("    {name},\n"))
    }
    rust.push_str("}\nimpl CsxObject {\n");
    let mut signatures: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for method in classes.values().flat_map(|class| &class.methods) {
        signatures
            .entry(method.name.as_str())
            .or_default()
            .insert(method.ty);
    }
    let mut dispatch_names = BTreeMap::new();
    for (method, types) in signatures {
        for &ty in &types {
            let lowered_name = if types.len() == 1 {
                format!("csx_{method}")
            } else {
                let tag = match ty {
                    ReturnType::Int => "int",
                    ReturnType::String => "string",
                    ReturnType::Bool => "bool",
                    ReturnType::Temporal(TemporalType::DateOnly) => "date",
                    ReturnType::Temporal(TemporalType::TimeOnly) => "time",
                    ReturnType::Temporal(TemporalType::DateTime) => "datetime",
                    ReturnType::Temporal(TemporalType::TimeSpan) => "span",
                };
                format!("csx__{tag}_{method}")
            };
            dispatch_names.insert((method, ty), lowered_name);
        }
    }
    let mut used_dispatch_names = BTreeSet::new();
    for (&(method, ty), lowered_name) in &dispatch_names {
        if !used_dispatch_names.insert(lowered_name) {
            return Err(format!(
                "CSX2208: conflicting lowered method name for {method}"
            ));
        }
        rust.push_str(&format!(
            "  fn {lowered_name}(&self) -> {} {{\n    match self {{\n",
            ty.rust_type()
        ));
        for name in classes.keys() {
            let value = find_method(&classes, name, method)
                .filter(|(_, definition)| definition.ty == ty)
                .and_then(|(_, definition)| definition.value.as_ref());
            match value {
                Some(value) => rust.push_str(&format!("      Self::{name} => {value},\n")),
                None => rust.push_str(&format!(
                    "      Self::{name} => panic!(\"invalid dispatch\"),\n"
                )),
            }
        }
        rust.push_str("    }\n  }\n");
    }
    rust.push_str("}\npub fn csx_main() -> Result<(), String> {\n");
    let mut scalar_vars = BTreeMap::new();
    if let Some(abi) = abi {
        if vars.contains_key("Dll") {
            return Err("CSX2503: Dll is reserved for the typed DLL reference".into());
        }
        scalar_vars.insert("Dll".to_owned(), Ty::DllApi(abi.clone()));
    }
    let object_names = vars.keys().cloned().collect::<BTreeSet<_>>();
    let mut env: BTreeMap<String, String> = BTreeMap::new();
    let mut report=String::from("# CSharpX Ownership Analysis (experimental)\n\nThis is a conservative, straight-line alias analysis. No borrowing, cycles, mutation or escape analysis yet.\n\n| Allocation | Strategy | Aliases |\n|---|---|---|\n");
    for a in &actions {
        match a {
            Action::Scalar(tokens) => {
                emit_scalar_statement(tokens.clone(), &mut scalar_vars, &object_names, &action_scope.file_imports, &mut rust)?
            }
            Action::New { name, concrete, .. } => {
                let count = allocation_aliases[name];
                let value = if let Some(values) = borrowed_values.as_ref().filter(|_| {
                    borrowed.is_some_and(|contract| contract.class == *concrete)
                }) {
                    format!("::csx_typed_abi::Configuration::new([{values}])")
                } else {
                    format!("CsxObject::{concrete}")
                };
                if count == 1 {
                    rust.push_str(&format!(
                        "  let v_{name} = Box::new({value});\n"
                    ));
                    report.push_str(&format!("| {name} | Box (exclusive) | 1 |\n"));
                } else {
                    rust.push_str(&format!(
                        "  let v_{name} = Rc::new({value});\n"
                    ));
                    report.push_str(&format!("| {name} | Rc (shared) | {count} |\n"));
                }
                env.insert(name.clone(), name.clone());
                scalar_vars.insert(name.clone(), getter_type(&classes, &vars[name].0, &dispatch_names, borrowed));
            }
            Action::Alias { name, from, .. } => {
                let root = env.get(from).unwrap().clone();
                rust.push_str(&format!("  let v_{name} = Rc::clone(&v_{from});\n"));
                env.insert(name.clone(), root);
                scalar_vars.insert(name.clone(), getter_type(&classes, &vars[name].0, &dispatch_names, borrowed));
            }
            Action::Print {
                receiver,
                method,
                tokens,
            } => {
                if env.contains_key(receiver) && !borrowed.is_some_and(|contract| contract.class == vars[receiver].0) {
                    let ty = &vars[receiver].0;
                    let definition = find_method(&classes, ty, method)
                        .ok_or_else(|| format!("CSX2305: method {ty}.{method} not found"))?
                        .1;
                    let lowered_name = &dispatch_names[&(method.as_str(), definition.ty)];
                    let call = format!("v_{receiver}.{lowered_name}()");
                    let value = match definition.ty {
                        ReturnType::Bool => {
                            format!("if {call} {{ \"True\" }} else {{ \"False\" }}")
                        }
                        ReturnType::Temporal(_) => format!("{call}?"),
                        _ => call,
                    };
                    emit_write_line(&mut rust, "    ", &value);
                } else {
                    emit_scalar_statement(
                        tokens.clone(),
                        &mut scalar_vars,
                        &object_names,
                        &action_scope.file_imports,
                        &mut rust,
                    )?;
                }
            }
        }
    }
    rust.push_str("    Ok(())\n}\n#[cfg(not(csharpx_library))]\nfn main(){if let Err(error)=csx_main(){eprintln!(\"{}\",error);std::process::exit(1);}}\n");
    append_c_imports(&mut rust, &action_scope.file_imports);
    append_runtime(&mut rust);
    strings::append_runtime(&mut rust);
    arrays::append_runtime(&mut rust);
    crate::language::append_clock_runtime(&mut rust);
    if !specializations.is_empty() {
        report.push_str("\n## Generic specializations\n\n");
        for specialization in specializations {
            let bindings = specialization
                .bindings
                .iter()
                .map(|(parameter, ty, default)| match default {
                    Some(default) => format!("{parameter} = {ty} (optional; default {default})"),
                    None => format!("{parameter} = {ty}"),
                })
                .collect::<Vec<_>>();
            report.push_str(&format!(
                "- {}<{}>\n",
                specialization.name,
                bindings.join(", ")
            ));
        }
    }
    Ok((rust, report))
}
#[cfg(test)]
mod tests {
    use super::*;
    const GOOD:&str="class Animal { public virtual int Speak() { return 1; } } class Dog : Animal { public override int Speak() { return 2; } } public static void Main() { Animal a = new Dog(); Animal b = a; Console.WriteLine(b.Speak()); }";
    #[test]
    fn inheritance_and_shared() {
        let (r, report) = compile(GOOD).unwrap();
        assert!(r.contains("Self::Dog => 2i64"));
        assert!(r.contains("Rc::clone"));
        assert!(report.contains("Rc (shared)"));
    }
    #[test]
    fn exclusive() {
        let (r,report)=compile("class A { public int Get() { return 3; } } void Main() { A a = new A(); Console.WriteLine(a.Get()); }").unwrap();
        assert!(r.contains("Box::new"));
        assert!(report.contains("Box (exclusive)"));
    }
    #[test]
    fn invalid_override() {
        assert!(compile("class A { public int Get() { return 3; } } class B : A { public override int Get() { return 4; } } void Main() {}").is_err());
    }
    #[test]
    fn cycle() {
        assert!(compile("class A : B {} class B : A {} void Main() {}").is_err());
    }

    #[test]
    fn specializes_generic_base_and_rejects_cross_type_assignment() {
        let src = include_str!("../../../examples/ownership/Program.csx");
        let (rust, report) = compile(src).unwrap();
        assert!(rust.contains("fn csx__int_Value"));
        assert!(rust.contains("fn csx__string_Value"));
        assert!(report.contains("| calibration | Box (exclusive) | 1 |"));
        assert!(report.contains("| acquisition | Rc (shared) | 3 |"));
        let wrong = src.replace("Reading<int> calibration", "Reading<string> calibration");
        assert!(compile(&wrong).unwrap_err().contains("CSX2303"));
    }

    #[test]
    fn reports_all_resolved_arguments_and_default_types() {
        let src = include_str!("../../../examples/ownership/Program.csx");
        let (_, report) = compile(src).unwrap();
        assert!(report.contains("Reading<T = string, TLabel = string (optional; default string)>"));
        assert!(report.contains("Reading<T = int, TLabel = bool (optional; default string)>"));
        assert!(report.contains("Snapshot<TDate = DateOnly, TTime = TimeOnly, TStamp = DateTime, TDuration = TimeSpan (optional; default TimeSpan)>"));
        let missing = src.replace("Reading<int> calibration", "Reading<> calibration");
        assert!(compile(&missing)
            .unwrap_err()
            .contains("missing generic argument T"));
        let extra = src.replace(
            "Reading<int> calibration",
            "Reading<int, string, bool> calibration",
        );
        assert!(compile(&extra)
            .unwrap_err()
            .contains("too many generic arguments"));
        let wrong_order = src.replace(
            "class Reading<T, TLabel = string>",
            "class Reading<T = int, TLabel>",
        );
        assert!(compile(&wrong_order)
            .unwrap_err()
            .contains("required generic parameters must precede optional ones"));
    }
}
