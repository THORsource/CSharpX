#![forbid(unsafe_code)]
#[path = "temporal.rs"]
pub(crate) mod temporal;
use temporal::TemporalType;
use crate::diagnostics::{Position, SourceChars};
#[path = "arrays.rs"]
pub(crate) mod arrays;
#[path = "interpolation.rs"]
mod interpolation;
#[path = "tuples.rs"]
mod tuples;
#[path = "strings.rs"]
pub(crate) mod strings;
#[derive(Clone, Debug, PartialEq)]
pub enum Tok { Interpolated(Vec<interpolation::LiteralPart>), Ident(String), Num(i64), Str(String), Sym(char), Op(String), Eof }
fn ident_start(c: char)->bool {c.is_ascii_alphabetic()||c=='_'}
fn ident_part(c: char)->bool {ident_start(c)||c.is_ascii_digit()}
pub(crate) fn lex_with_lines(src: &str) -> Result<(Vec<Tok>, Vec<Position>), String> {
    let mut input = SourceChars::new(src);
    let mut tokens = Vec::new();
    let mut positions = Vec::new();
    loop {
        let position = input.position;
        let Some(ch) = input.next() else { break };
        if ch.is_whitespace() || (position == Position::START && ch == '\u{feff}') {
            continue;
        }
        if ch == '/' && input.peek() == Some(&'/') {
            for ch in input.by_ref() {
                if ch == '\n' { break; }
            }
            continue;
        }
        if ch == '/' && input.peek() == Some(&'*') {
            input.next();
            let mut previous = ' ';
            let mut closed = false;
            for ch in input.by_ref() {
                if previous == '*' && ch == '/' {
                    closed = true;
                    break;
                }
                previous = ch;
            }
            if !closed {
                return Err(position.annotate("CSX1001: unterminated comment"));
            }
            continue;
        }
        let token = if ident_start(ch) || ch.is_ascii_digit() {
            let is_ident = ident_start(ch);
            let mut value = ch.to_string();
            while let Some(&next) = input.peek() {
                if if is_ident { !ident_part(next) } else { !next.is_ascii_digit() } {
                    break;
                }
                value.push(next);
                input.next();
            }
            if is_ident {
                Tok::Ident(value)
            } else {
                Tok::Num(value.parse().map_err(|_| position.annotate("CSX1002: integer out of range"))?)
            }
        } else if matches!(ch, '"' | '@' | '$') {
            interpolation::scan(ch, &mut input).map_err(|error| position.annotate(error))?
        } else if "=+-*/%!<>".contains(ch) && input.peek() == Some(&'=') {
            input.next();
            Tok::Op(format!("{ch}="))
        } else if "=+-*/%!<>{}[]();,.:&".contains(ch) {
            Tok::Sym(ch)
        } else {
            return Err(position.annotate(format!("CSX1006: invalid character {ch:?}")));
        };
        positions.push(position);
        tokens.push(token);
    }
    positions.push(input.position);
    tokens.push(Tok::Eof);
    Ok((tokens, positions))
}

#[derive(Clone,Debug)]pub enum Expr{Tuple(Vec<(Option<String>, Expr)>), AddressOf(String, Position), Dereference(Box<Expr>), NativeFileRoundTrip(Option<String>, String, Box<Expr>),Timestamp, ClockFrequency, Interpolated(Vec<interpolation::Part>),Array(Option<arrays::Element>,arrays::Initializer),Index(Box<Expr>,Box<Expr>),Linq(Box<Query>),Temporal(TemporalType,Vec<Expr>),Member(Box<Expr>,String),MemberCall(Box<Expr>,String,Vec<Expr>),StringType,Null,Bool(bool),Num(i64),Str(String),Var(String,Position),Binary(Box<Expr>,char,Box<Expr>)}

#[derive(Clone, Debug)]
pub struct Query {
    start: Expr,
    count: Expr,
    steps: Vec<QueryStep>,
    aggregate: Aggregate,
}

#[derive(Clone, Debug)]
enum QueryStep {
    Where { parameter: String, left: Expr, operator: String, right: Expr },
    Select { parameter: String, expression: Expr },
}

#[derive(Clone, Copy, Debug)]
enum Aggregate { Count, Sum }

enum Stmt{Deconstruct(Vec<String>, Expr), SetTupleField(String, String, Expr), StorePointer(Expr, Expr), Unsafe(Vec<LocatedStmt>), If(Condition, Vec<LocatedStmt>, Vec<LocatedStmt>), While(Condition, Vec<LocatedStmt>), Break, SetIndex(Expr,Expr,Expr),Let(String,Expr,Option<Ty>),Assign(String,Expr),Print(Expr),ReadLine,Throw(Expr),Try{body:Vec<LocatedStmt>,catch_name:Option<String>,catch_body:Option<Vec<LocatedStmt>>,finally_body:Option<Vec<LocatedStmt>>}}
pub fn lex(src: &str) -> Result<Vec<Tok>, String> {
    lex_with_lines(src).map(|(tokens, _)| tokens)
}

struct LocatedStmt {
    position: Position,
    kind: Stmt,
}

struct Condition {
    left: Expr,
    comparison: Option<(String, Expr)>,
}

#[derive(Clone)]
struct CHeaderUsage {
    name: String,
    alias: Option<String>,
}

struct Parser { t: Vec<Tok>, lines: Vec<Position>, i: usize, loop_depth: usize, in_unsafe: bool, c_headers: Vec<CHeaderUsage> }
impl Parser{
 fn new(t: Vec<Tok>, lines: Vec<Position>) -> Self { Self { t, lines, i: 0, loop_depth: 0, in_unsafe: false, c_headers: Vec::new() } }
 fn peek(&self)->&Tok{&self.t[self.i]}
 fn next(&mut self)->Tok{let t=self.peek().clone();if t!=Tok::Eof{self.i+=1;}t}
 fn sym(&mut self,c:char)->bool{if *self.peek()==Tok::Sym(c){self.next();true}else{false}}
 fn expect(&mut self,c:char)->Result<(),String>{if self.sym(c){Ok(())}else{Err(format!("CSX1101: expected '{c}', got {:?}",self.peek()))}}
 fn word(&mut self,s:&str)->bool{if self.peek()==&Tok::Ident(s.into()){self.next();true}else{false}}
 fn name(&mut self)->Result<String,String>{match self.next(){Tok::Ident(s)=>Ok(s),x=>Err(format!("CSX1102: expected identifier, got {x:?}"))}}

 fn primary(&mut self) -> Result<Expr, String> {
     let position = self.lines[self.i];
     let expr = match self.next() {
         Tok::Num(n) => Expr::Num(n),
         Tok::Str(s) => Expr::Str(s),
         Tok::Interpolated(parts) => interpolation::parse(parts, position, &self.c_headers, self.in_unsafe)?,
         Tok::Ident(alias) if self.c_headers.iter().any(|header| header.alias.as_deref() == Some(&alias))
             && self.sym('.') =>
         {
             self.require_unsafe(position)?;
             let name = self.name()?;
             self.expect('(')?;
             let path = self.expr(0)?;
             self.expect(')')?;
             Expr::NativeFileRoundTrip(Some(alias), name, Box::new(path))
         }
         Tok::Ident(s) if s == "CFile" && self.sym('.') => {
             self.require_c_header(position)?;
             self.require_unsafe(position)?;
             let name = self.name()?;
             let name = if name == "RoundTrip" { "csx_c_file_round_trip".into() } else { name };
             self.expect('(')?;
             let path = self.expr(0)?;
             self.expect(')')?;
             Expr::NativeFileRoundTrip(None, name, Box::new(path))
         }
         Tok::Ident(s) if s == "Stopwatch" && self.sym('.') => {
             match self.name()?.as_str() {
                 "GetTimestamp" => {
                     self.expect('(')?;
                     self.expect(')')?;
                     Expr::Timestamp
                 }
                 "Frequency" => Expr::ClockFrequency,
                 _ => return Err("CSX1108: unknown Stopwatch member".into()),
             }
         }
         Tok::Ident(s) if s == "Enumerable" && self.peek() == &Tok::Sym('.') => self.linq_query()?,
         Tok::Ident(s) if s == "new" => self.array_or_temporal()?,
         Tok::Ident(s) if matches!(s.as_str(), "String" | "string") => Expr::StringType,
         Tok::Ident(s) if s == "System" && self.peek() == &Tok::Sym('.') => {
             self.next();
             if !self.word("String") {
                 return Err("CSX1501: only System.String is supported in a qualified expression".into());
             }
             Expr::StringType
         }
         Tok::Ident(s) if s == "null" => Expr::Null,
         Tok::Ident(s) if s == "true" || s == "false" => Expr::Bool(s == "true"),
         Tok::Ident(s) if self.sym('(') => {
             self.require_c_header(position)?;
             self.require_unsafe(position)?;
             let path = self.expr(0)?;
             self.expect(')')?;
             Expr::NativeFileRoundTrip(None, s, Box::new(path))
         }
         Tok::Ident(s) => Expr::Var(s,position),
         Tok::Sym('&') => {
             self.require_pointer_scope(position)?;
             Expr::AddressOf(self.name()?, position)
         }
         Tok::Sym('*') => {
             self.require_pointer_scope(position)?;
             Expr::Dereference(Box::new(self.primary()?))
         }
         Tok::Sym('-') => Expr::Binary(Box::new(Expr::Num(0)), '-', Box::new(self.primary()?)),
         Tok::Sym('(') => tuples::expression(self)?,
         token => return Err(format!("CSX1103: invalid expression {token:?}")),
     };
     self.member_postfix(expr)
 }


    pub(super) fn member_postfix(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            if self.sym('[') {
                let index = self.expr(0)?;
                self.expect(']')?;
                expr = Expr::Index(Box::new(expr), Box::new(index));
                continue;
            }
            if !self.sym('.') {
                break;
            }
            let member = self.name()?;
            if self.sym('(') {
                let mut args = Vec::new();
                if !self.sym(')') {
                    args.push(self.expr(0)?);
                    while self.sym(',') {
                        args.push(self.expr(0)?);
                    }
                    self.expect(')')?;
                }
                expr = Expr::MemberCall(Box::new(expr), member, args);
            } else {
                expr = Expr::Member(Box::new(expr), member);
            }
        }
        Ok(expr)
    }

 fn linq_query(&mut self) -> Result<Expr, String> {
     self.expect('.')?;
     if !self.word("Range") {
         return Err("CSX1301: expected Enumerable.Range".into());
     }
     self.expect('(')?;
     let start = self.expr(0)?;
     self.expect(',')?;
     let count = self.expr(0)?;
     self.expect(')')?;
     let mut steps = Vec::new();
     loop {
         self.expect('.').map_err(|_| "CSX1302: LINQ requires Count() or Sum()".to_string())?;
         let method = self.name()?;
         self.expect('(')?;
         match method.as_str() {
             "Count" | "Sum" => {
                 self.expect(')')?;
                 let aggregate = if method == "Count" { Aggregate::Count } else { Aggregate::Sum };
                 return Ok(Expr::Linq(Box::new(Query { start, count, steps, aggregate })));
             }
             "Where" | "Select" => {
                 let parameter = self.name()?;
                 self.expect('=')?;
                 self.expect('>')?;
                 let expression = self.expr(0)?;
                 if method == "Where" {
                     let operator = match self.next() {
                         Tok::Sym(op @ ('<' | '>')) => op.to_string(),
                         Tok::Op(op) if matches!(op.as_str(), "<=" | ">=" | "==" | "!=") => op,
                         _ => return Err("CSX1303: Where requires an integer comparison".into()),
                     };
                     let right = self.expr(0)?;
                     steps.push(QueryStep::Where { parameter, left: expression, operator, right });
                 } else {
                     steps.push(QueryStep::Select { parameter, expression });
                 }
                 self.expect(')')?;
             }
             _ => return Err(format!("CSX1304: unsupported LINQ operator {method}")),
         }
     }
 }

 fn expr(&mut self,min:u8)->Result<Expr,String>{let mut a=self.primary()?;loop{let (op,p)=match self.peek(){Tok::Sym('+')=>('+',1),Tok::Sym('-')=>('-',1),Tok::Sym('*')=>('*',2),Tok::Sym('/')=>('/',2),Tok::Sym('%')=>('%',2),_=>break};if p<min{break;}self.next();let b=self.expr(p+1)?;a=Expr::Binary(Box::new(a),op,Box::new(b));}Ok(a)}
 fn block(&mut self)->Result<Vec<LocatedStmt>,String>{self.expect('{')?;let mut v=Vec::new();while !self.sym('}'){if self.peek()==&Tok::Eof{return Err("CSX1104: unclosed block".into())}v.push(self.stmt()?);}Ok(v)}
 fn stmt(&mut self) -> Result<LocatedStmt, String> {
     let position = self.lines[self.i];
     Ok(LocatedStmt { position, kind: self.statement_kind().map_err(|error| self.lines[self.i].annotate(error))? })
 }

 fn condition(&mut self) -> Result<Condition, String> {
     self.expect('(')?;
     let left = self.expr(0)?;
     let operator = match self.peek() {
         Tok::Sym(op @ ('<' | '>')) => Some(op.to_string()),
         Tok::Op(op) if matches!(op.as_str(), "<=" | ">=" | "==" | "!=") => Some(op.clone()),
         _ => None,
     };
     let comparison = if let Some(operator) = operator {
         self.next();
         Some((operator, self.expr(0)?))
     } else {
         None
     };
     self.expect(')')?;
     Ok(Condition { left, comparison })
 }

 fn exception_block(&mut self) -> Result<Vec<LocatedStmt>, String> {
     let outer_depth = std::mem::take(&mut self.loop_depth);
     let body = self.block();
     self.loop_depth = outer_depth;
     body
 }

 fn statement_kind(&mut self)->Result<Stmt,String>{
  if self.word("unsafe") {
      let was_unsafe = std::mem::replace(&mut self.in_unsafe, true);
      let body = self.block();
      self.in_unsafe = was_unsafe;
      return Ok(Stmt::Unsafe(body?));
  }
  if self.word("if") {
      let condition = self.condition()?;
      let body = self.block()?;
      let otherwise = if self.word("else") { self.block()? } else { Vec::new() };
      return Ok(Stmt::If(condition, body, otherwise));
  }
  if self.word("while") {
      let condition = self.condition()?;
      self.loop_depth += 1;
      let body = self.block();
      self.loop_depth -= 1;
      return Ok(Stmt::While(condition, body?));
  }
  if self.word("break") {
      if self.loop_depth == 0 {
          return Err("CSX1108: break requires an enclosing loop in the same exception block".into());
      }
      self.expect(';')?;
      return Ok(Stmt::Break);
  }
  if self.word("try") {
      let body = self.exception_block()?;
      let mut catch_name = None;
      let catch_body = if self.word("catch") {
          if self.sym('(') {
              self.name()?;
              if !self.sym(')') {
                  catch_name = Some(self.name()?);
                  self.expect(')')?;
              }
          }
          Some(self.exception_block()?)
      } else {
          None
      };
      let finally_body = if self.word("finally") {
          Some(self.exception_block()?)
      } else {
          None
      };
      if catch_body.is_none() && finally_body.is_none() {
          return Err("CSX1105: try requires catch or finally".into());
      }
      return Ok(Stmt::Try { body, catch_name, catch_body, finally_body });
  }
  if self.word("throw"){let e=self.expr(0)?;self.expect(';')?;return Ok(Stmt::Throw(e));}
  let position = self.lines[self.i];
  if self.sym('*') {
      self.require_pointer_scope(position)?;
      let pointer = self.primary()?;
      self.expect('=')?;
      let value = self.expr(0)?;
      self.expect(';')?;
      return Ok(Stmt::StorePointer(pointer, value));
  }
  if matches!(&self.t[self.i..], [Tok::Ident(word), Tok::Sym('('), ..] if word == "var") {
      self.next();
      self.expect('(')?;
      let mut names = vec![self.name()?];
      self.expect(',')?;
      loop {
          names.push(self.name()?);
          if !self.sym(',') { break; }
      }
      self.expect(')')?;
      self.expect('=')?;
      let expr = self.expr(0)?;
      self.expect(';')?;
      return Ok(Stmt::Deconstruct(names, expr));
  }
  if self.sym('(') {
      let ty = tuples::declared_type(self)?;
      let name = self.name()?;
      self.expect('=')?;
      let expr = self.expr(0)?;
      self.expect(';')?;
      return Ok(Stmt::Let(name, expr, Some(ty)));
  }
  let declaration = match self.peek() {
      Tok::Ident(name) => match name.as_str() {
          "var" => Some(None),
          "int" => Some(Some(Ty::Int)),
          "string" | "String" | "System" => Some(Some(Ty::Str)),
          "bool" => Some(Some(Ty::Bool)),
          name => TemporalType::from_name(name).map(|ty| Some(Ty::Temporal(ty))),
      },
      _ => None,
  };
  if let Some(expected) = declaration {
      if self.word("System") {
          self.expect('.')?;
          if !self.word("String") {
              return Err("CSX1501: only System.String is supported as a qualified type".into());
          }
      } else {
          self.next();
      }
      let expected = if self.sym('*') {
          self.require_pointer_scope(position)?;
          Some(match expected {
              Some(Ty::Int) => Ty::Pointer(Pointee::Int),
              Some(Ty::Bool) => Ty::Pointer(Pointee::Bool),
              _ => return Err("CSX1902: only int* and bool* pointer types are supported".into()),
          })
      } else if self.sym('[') {
          self.expect(']')?;
          let ty = expected.ok_or("CSX1601: var[] is not an array type")?;
          Some(Ty::Array(arrays::Element::from_type(&ty)?))
      } else {
          expected
      };
      let name = self.name()?;
      self.expect('=')?;
      let value = match (&expected, self.peek()) {
          (Some(Ty::Array(element)), Tok::Sym('{')) => self.array_items(Some(*element))?,
          _ => self.expr(0)?,
      };
      self.expect(';')?;
      return Ok(Stmt::Let(name, value, expected));
  }
  let position = self.lines[self.i];
  let first=self.name()?;
  if first == "Console" {
      self.expect('.')?;
      let method = self.name()?;
      self.expect('(')?;
      let statement = match method.as_str() {
          "WriteLine" => Stmt::Print(self.expr(0)?),
          "ReadLine" => Stmt::ReadLine,
          _ => return Err(format!("CSX1106: unsupported Console method {method}")),
      };
      self.expect(')')?;
      self.expect(';')?;
      return Ok(statement);
  }
  if self.sym('.') {
      let field = self.name()?;
      self.expect('=')?;
      let value = self.expr(0)?;
      self.expect(';')?;
      return Ok(Stmt::SetTupleField(first, field, value));
  }
  if self.sym('[') {
      let index = self.expr(0)?;
      self.expect(']')?;
      self.expect('=')?;
      let value = self.expr(0)?;
      self.expect(';')?;
      return Ok(Stmt::SetIndex(Expr::Var(first,position), index, value));
  }
  self.expect('=')?;let e=self.expr(0)?;self.expect(';')?;Ok(Stmt::Assign(first,e))
 }
 fn require_pointer_scope(&self, position: Position) -> Result<(), String> {
     if self.in_unsafe {
         return Ok(());
     }
     Err(position.annotate("CSX1901: pointer operations require an unsafe block"))
 }

 fn require_unsafe(&self, position: Position) -> Result<(), String> {
     if self.in_unsafe {
         return Ok(());
     }
     Err(position.annotate("CSX1709: calls to imported functions must be inside an unsafe block"))
 }

 fn require_c_header(&self, position: Position) -> Result<(), String> {
     if self.c_headers.iter().any(|header| header.alias.is_none()) {
         return Ok(());
     }
     let message = if self.c_headers.is_empty() {
         "CSX1707: external calls require a using directive with a quoted filename"
     } else {
         "CSX1708: qualify this external call with its import alias"
     };
     Err(position.annotate(message))
 }

 fn read_imports(&mut self) -> Result<(), String> {
     while self.word("using") {
         if matches!(self.peek(), Tok::Str(_)) {
             let position = self.lines[self.i];
             let Tok::Str(name) = self.next() else {
                 return Err(position.annotate("CSX1707: expected a quoted .h, .cs or .rs filename"));
             };
             let valid = name.rsplit_once('.').is_some_and(|(stem, ext)| {
                 !stem.trim().is_empty() && ["h", "cs", "rs"].iter().any(|allowed| ext.eq_ignore_ascii_case(allowed))
             }) && !name.chars().any(|ch| ch.is_control() || matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'));
             if !valid {
                 return Err(position.annotate("CSX1707: using expects a .h, .cs or .rs filename from the project"));
             }
             let alias = if self.word("AS") || self.word("as") {
                 let position = self.lines[self.i];
                 let alias = self.name()?;
                 if !crate::c_header::identifier(&alias) {
                     return Err(position.annotate("CSX1708: expected a C header alias identifier"));
                 }
                 if self.c_headers.iter().any(|header| header.alias.as_deref() == Some(&alias)) {
                     return Err(position.annotate(format!("CSX1708: duplicate C header alias {alias}")));
                 }
                 Some(alias)
             } else {
                 None
             };
             self.expect(';')?;
             if alias.is_some() || !self.c_headers.iter().any(|header| {
                 header.alias.is_none() && header.name.eq_ignore_ascii_case(&name)
             }) {
                 self.c_headers.push(CHeaderUsage { name, alias });
             }
         } else {
             while !self.sym(';') {
                 if self.next() == Tok::Eof {
                     return Err("CSX1107: invalid using".into());
                 }
             }
         }
     }
     if matches!(self.peek(), Tok::Ident(word) if word == "usage") {
         return Err(self.lines[self.i].annotate("CSX1707: use using with a quoted filename for file imports"));
     }
     Ok(())
 }

 fn program(&mut self)->Result<Vec<LocatedStmt>,String>{
  // First supported top-level shape: optional class wrapper around public static void Main().
  self.read_imports()?;
  if self.word("public"){}if self.word("static"){}if self.word("class"){self.name()?;self.expect('{')?;}
  if self.word("public"){}if self.word("static"){}if !self.word("void"){return Err("CSX1108: expected void Main".into())}if !self.word("Main"){return Err("CSX1109: expected Main".into())}self.expect('(')?;self.expect(')')?;let v=self.block()?;while self.sym('}'){}if self.peek()!=&Tok::Eof{return Err("CSX1110: trailing tokens".into())}Ok(v)
 }
}
#[derive(Clone,PartialEq)]pub(crate) enum Ty{Tuple(Vec<tuples::Field>),Int,Str,Bool,Pointer(Pointee),Temporal(TemporalType),Array(arrays::Element)}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Pointee {
    Int,
    Bool,
}

impl Pointee {
    fn ty(self) -> Ty {
        match self {
            Self::Int => Ty::Int,
            Self::Bool => Ty::Bool,
        }
    }
}

// The unescaped quotes distinguish a generated marker from user string literals.
const POINTER_ACCESS_MARKER: &str = r#"/*"CSharpX pointer access"*/"#;
pub(crate) const C_HEADER_USAGE_PREFIX: &str = r#"/*"CSharpX header usage" "#;
pub(crate) const C_FILE_CALL_PREFIX: &str = r#"/*"CSharpX C file bridge" "#;

fn emit_expr(e:&Expr,vars:&std::collections::BTreeMap<String,Ty>)->Result<(String,Ty),String>{match e{
 Expr::Tuple(elements) => tuples::emit(elements, vars),
 Expr::AddressOf(name, position) => {
     let target = match vars.get(name) {
         Some(Ty::Int) => Pointee::Int,
         Some(Ty::Bool) => Pointee::Bool,
         Some(_) => return Err("CSX1902: can only take the address of an int or bool local".into()),
         None => return Err(position.annotate(format!("CSX1201: unknown variable {name}"))),
     };
     Ok((format!("std::ptr::addr_of_mut!(v_{name})"), Ty::Pointer(target)))
 }
 Expr::Dereference(pointer) => {
     let (pointer, target) = emit_pointer(pointer, vars)?;
     let code = format!("{{ {POINTER_ACCESS_MARKER} #[allow(unsafe_code)] let csx_value = unsafe {{ std::ptr::read({pointer}) }}; csx_value }}");
     Ok((code, target.ty()))
 }
 Expr::NativeFileRoundTrip(alias, name, path) => {
     let (path, kind) = emit_expr(path, vars)?;
     if kind != Ty::Str {
         return Err("CSX1701: imported function expects one string argument".into());
     }
     let scope = alias.as_deref().unwrap_or("");
     let binding = crate::c_header::binding_name(scope, name);
     Ok((format!("{C_FILE_CALL_PREFIX}{scope}|{name}*/::csx_c_file::{binding}(&({path}))?"), Ty::Str))
 }
 Expr::Timestamp => Ok(("csx_timestamp()".into(), Ty::Int)),
 Expr::ClockFrequency => Ok(("1_000_000_000i64".into(), Ty::Int)),
 Expr::Interpolated(parts)=>interpolation::emit(parts,vars),
 Expr::Array(ty,initializer)=>arrays::emit_array(*ty,initializer,vars),
 Expr::Index(array,index)=>arrays::get(array,index,vars),
 Expr::Temporal(ty,args)=>temporal::constructor(*ty,args,vars),
 Expr::Member(receiver,member)=>tuples::member(receiver,member,vars),
 Expr::MemberCall(receiver,member,args)=>strings::call(receiver,member,args,vars),
 Expr::StringType | Expr::Null=>Err("CSX1502: String type and null require a supported String member call".into()),
 Expr::Bool(value)=>Ok((value.to_string(),Ty::Bool)),
 Expr::Linq(query)=>Ok((emit_query(query, vars)?,Ty::Int)), Expr::Num(n)=>Ok((format!("{n}i64"),Ty::Int)),Expr::Str(s)=>Ok((format!("{:?}.to_string()",s),Ty::Str)),Expr::Var(n,position)=>{let t=vars.get(n).ok_or_else(||position.annotate(format!("CSX1201: unknown variable {n}")))?;Ok((format!("v_{n}.clone()"),t.clone()))},Expr::Binary(a,op,b)=>{let (aa,at)=emit_expr(a,vars)?;let (bb,bt)=emit_expr(b,vars)?;if *op == '+' && (at == Ty::Str || bt == Ty::Str) && !matches!((&at, &bt), (Ty::Array(_), _) | (_, Ty::Array(_))) {
    let a = display_expression(aa, at);
    let b = display_expression(bb, bt);
    return Ok((format!("format!(\"{{}}{{}}\", {a}, {b})"), Ty::Str));
}
if at!=Ty::Int||bt!=Ty::Int{return Err("CSX1202: arithmetic requires int".into())}let code=if *op=='/'{format!("({aa}).checked_div({bb}).ok_or_else(||\"DivideByZeroException or overflow\".to_string())?")}else if *op=='%'{format!("({aa}).checked_rem({bb}).ok_or_else(||\"DivideByZeroException or overflow\".to_string())?")}else{format!("({aa} {op} {bb})")};Ok((code,Ty::Int))}
}}

fn emit_pointer(
    pointer: &Expr,
    vars: &std::collections::BTreeMap<String, Ty>,
) -> Result<(String, Pointee), String> {
    let (code, ty) = emit_expr(pointer, vars)?;
    match ty {
        Ty::Pointer(target) => Ok((code, target)),
        _ => Err("CSX1903: dereference requires a pointer".into()),
    }
}

fn emit_query(query: &Query, vars: &std::collections::BTreeMap<String, Ty>) -> Result<String, String> {
    use std::fmt::Write;

    fn integer(expr: &Expr, vars: &std::collections::BTreeMap<String, Ty>) -> Result<String, String> {
        let (code, ty) = emit_expr(expr, vars)?;
        if ty != Ty::Int {
            return Err("CSX1305: LINQ requires integer expressions".into());
        }
        Ok(code)
    }

    let start = integer(&query.start, vars)?;
    let count = integer(&query.count, vars)?;
    let mut code = format!(
        "{{ let csx_start = i64::from(i32::try_from({start}).map_err(|_| \"ArgumentOutOfRangeException\".to_string())?); \
         let csx_count = i64::from(i32::try_from({count}).map_err(|_| \"ArgumentOutOfRangeException\".to_string())?); \
         if csx_count < 0 || csx_start + csx_count - 1 > i64::from(i32::MAX) {{ \
             return Err(\"ArgumentOutOfRangeException\".to_string()); \
         }} let mut csx_total = 0i64;"
    );
    let mut local = vars.clone();
    if query.steps.iter().any(|step| matches!(step, QueryStep::Select { .. })) {
        code.push_str(" for mut csx_item in csx_start..csx_start+csx_count {");
    } else {
        code.push_str(" for csx_item in csx_start..csx_start+csx_count {");
    }
    for step in &query.steps {
        let parameter = match step {
            QueryStep::Where { parameter, .. } | QueryStep::Select { parameter, .. } => parameter,
        };
        let previous = local.insert(parameter.clone(), Ty::Int);
        write!(code, " {{ let v_{parameter} = csx_item;").expect("write to string");
        match step {
            QueryStep::Where { left, operator, right, .. } => {
                let left = integer(left, &local)?;
                let right = integer(right, &local)?;
                write!(code, " if !(({left}) {operator} ({right})) {{ continue; }}").expect("write to string");
            }
            QueryStep::Select { expression, .. } => {
                let value = integer(expression, &local)?;
                write!(code, " csx_item = i64::from(({value}) as i32);").expect("write to string");
            }
        }
        code.push_str(" }");
        if let Some(ty) = previous {
            local.insert(parameter.clone(), ty);
        } else {
            local.remove(parameter);
        }
    }
    match query.aggregate {
        Aggregate::Count => code.push_str(" csx_total += 1;"),
        Aggregate::Sum => code.push_str(
            " csx_total = i64::from(i32::try_from(csx_total + csx_item).map_err(|_| \"OverflowException\".to_string())?);"
        ),
    }
    code.push_str(" } csx_total }");
    Ok(code)
}

fn display_expression(code: String, ty: Ty) -> String {
    match ty {
        Ty::Bool => format!("if {code} {{ \"True\" }} else {{ \"False\" }}"),
        Ty::Pointer(_) => format!("format!(\"{{:p}}\", {code})"),
        Ty::Tuple(fields) => tuples::display(code, fields),
        _ => code,
    }
}

pub(crate) fn emit_write_line(out: &mut String, pad: &str, expression: &str) {
    out.push_str(&format!(
        "{pad}print!(\"{{}}{{}}\", {expression}, if cfg!(windows) {{ \"\\r\\n\" }} else {{ \"\\n\" }});\n"
    ));
}

impl Condition {
    fn emit(&self, vars: &std::collections::BTreeMap<String, Ty>) -> Result<String, String> {
        let (left, left_type) = emit_expr(&self.left, vars)?;
        if let Some((operator, right)) = &self.comparison {
            let (right, right_type) = emit_expr(right, vars)?;
            let equality = matches!(operator.as_str(), "==" | "!=");
            if left_type == right_type && (left_type == Ty::Int || (equality && matches!(left_type, Ty::Bool | Ty::Pointer(_)))) {
                return Ok(format!("{left} {operator} {right}"));
            }
            return Err("CSX1204: incompatible comparison operands".into());
        }
        if left_type != Ty::Bool {
            return Err("CSX1204: expected a Boolean condition".into());
        }
        Ok(left)
    }
}

fn emit_stmts(stmts:&[LocatedStmt],vars:&mut std::collections::BTreeMap<String,Ty>,out:&mut String, mappings: &mut Vec<(usize, u32)>, depth:usize)->Result<(),String>{
 let pad="    ".repeat(depth);
 for statement in stmts {
    mappings.push((out.len(), statement.position.line));
    (|| -> Result<(), String> {
    match &statement.kind {
 Stmt::Deconstruct(names, expr) => tuples::deconstruct(names, expr, vars, out, &pad)?,
 Stmt::SetTupleField(name, field, expr) => tuples::assign_field(name, field, expr, vars, out, &pad)?,
 Stmt::StorePointer(pointer, value) => {
     let (pointer, target) = emit_pointer(pointer, vars)?;
     let (value, ty) = emit_expr(value, vars)?;
     if target.ty() != ty {
         return Err("CSX1903: pointer assignment requires a value of the pointed-to type".into());
     }
     out.push_str(&format!("{pad}{POINTER_ACCESS_MARKER} #[allow(unsafe_code)] let () = unsafe {{ std::ptr::write({pointer}, {value}) }};\n"));
 }
 Stmt::Unsafe(body) => {
     out.push_str(&format!("{pad}{{\n"));
     emit_stmts(body, &mut vars.clone(), out, mappings, depth + 1)?;
     out.push_str(&format!("{pad}}}\n"));
 }
 Stmt::While(condition, body) => {
     let condition = condition.emit(vars)?;
     out.push_str(&format!("{pad}while {condition} {{\n"));
     emit_stmts(body, &mut vars.clone(), out, mappings, depth + 1)?;
     out.push_str(&format!("{pad}}}\n"));
 }
 Stmt::If(condition, body, otherwise) => {
     let condition = condition.emit(vars)?;
     out.push_str(&format!("{pad}if {condition} {{\n"));
     emit_stmts(body, &mut vars.clone(), out, mappings, depth + 1)?;
     if !otherwise.is_empty() {
         out.push_str(&format!("{pad}}} else {{\n"));
         emit_stmts(otherwise, &mut vars.clone(), out, mappings, depth + 1)?;
     }
     out.push_str(&format!("{pad}}}\n"));
 }
 Stmt::Break => out.push_str(&format!("{pad}break;\n")),
 Stmt::Let(n,e,expected)=>{if vars.contains_key(n){return Err(format!("CSX1203: duplicate variable {n}"))}let (x,t)=emit_expr(e,vars)?;if let Some(expected) = expected { if !tuples::compatible(expected, &t) { let message = if matches!(expected, Ty::Temporal(_)) {"CSX1407: incompatible date/time initializer"} else {"CSX1504: incompatible initializer"}; return Err(message.into()); } }vars.insert(n.clone(),expected.clone().unwrap_or(t));out.push_str(&format!("{pad}let mut v_{n} = {x};\n"));},
 Stmt::SetIndex(array,index,value)=>{
     let assignment = arrays::set(array,index,value,vars)?;
     out.push_str(&format!("{pad}{assignment}\n"));
 },
 Stmt::Assign(n,e)=>{let expected=vars.get(n).ok_or_else(||format!("CSX1201: unknown variable {n}"))?;let (x,t)=emit_expr(e,vars)?;if !tuples::compatible(expected, &t){return Err("CSX1204: incompatible assignment".into())}out.push_str(&format!("{pad}v_{n} = {x};\n"));},
 Stmt::Print(e)=>{let (x,t)=emit_expr(e,vars)?;if matches!(t,Ty::Array(_)){return Err("CSX1606: print an array element or Length".into())}emit_write_line(out, &pad, &display_expression(x,t));},
 Stmt::ReadLine => out.push_str(&format!(
     "{pad}std::io::Write::flush(&mut std::io::stdout()).map_err(|error| error.to_string())?;\n\
      {pad}std::io::stdin().read_line(&mut String::new()).map_err(|error| error.to_string())?;\n"
 )),
 Stmt::Throw(e)=>{let (x,_)=emit_expr(e,vars)?;out.push_str(&format!("{pad}return Err(format!(\"{{}}\", {x}));\n"));},
 Stmt::Try{body,catch_name,catch_body,finally_body}=>{let mut local=vars.clone();out.push_str(&format!("{pad}let csx_result = (|| -> Result<(), String> {{\n"));emit_stmts(body,&mut local,out,mappings,depth+1)?;out.push_str(&format!("{pad}    Ok(())\n{pad}}})();\n"));if let Some(catch_body) = catch_body {out.push_str(&format!("{pad}let csx_result = match csx_result {{ Ok(()) => Ok(()), Err(csx_error) => {{\n"));let mut local=vars.clone();if let Some(n)=catch_name{local.insert(n.clone(),Ty::Str);out.push_str(&format!("{pad}    let mut v_{n} = csx_error;\n"));}else{out.push_str(&format!("{pad}    let _ = csx_error;\n"));}out.push_str(&format!("{pad}    (|| -> Result<(), String> {{\n"));emit_stmts(catch_body,&mut local,out,mappings,depth+2)?;out.push_str(&format!("{pad}        Ok(())\n{pad}    }})()\n{pad}}} }};\n"));}if let Some(finally_body) = finally_body {out.push_str(&format!("{pad}let csx_finally = (|| -> Result<(), String> {{\n"));let mut local=vars.clone();emit_stmts(finally_body,&mut local,out,mappings,depth+1)?;out.push_str(&format!("{pad}    Ok(())\n{pad}}})();\n{pad}csx_finally?;\n"));}out.push_str(&format!("{pad}csx_result?;\n"));}
 } Ok(()) })().map_err(|error| statement.position.annotate(error))?; } Ok(())}
pub fn compile(src: &str) -> Result<String, String> {
    compile_with_lines(src).map(|(rust, _)| rust)
}

pub fn compile_with_lines(src: &str) -> Result<(String, Vec<u32>), String> {
    let (tokens, lines) = lex_with_lines(src)?;
    let mut parser = Parser::new(tokens, lines);
    let stmts = parser.program().map_err(|error| parser.lines[parser.i].annotate(error))?;
    let mut mappings = Vec::new();
    let mut out=String::from("#![forbid(unsafe_code)]\npub fn csx_main()->Result<(),String>{\n");emit_stmts(&stmts,&mut Default::default(),&mut out,&mut mappings,1)?;out.push_str("    Ok(())\n}\n#[cfg(not(csharpx_library))]\nfn main(){if let Err(e)=csx_main(){eprintln!(\"Unhandled CSharpX exception: {}\",e);std::process::exit(1);}}\n");
    if out.contains(POINTER_ACCESS_MARKER) {
        // Both directives occupy one line; the source mapping stays unchanged.
        out.replace_range(.."#![forbid(unsafe_code)]".len(), "#![deny(unsafe_code)]");
        let removed = "forbid".len() - "deny".len();
        for (start, _) in &mut mappings {
            *start -= removed;
        }
    }
    let mut offset = 0;
    let mut next = mappings.into_iter().peekable();
    let mut source_line = 1;
    let mut line_map = Vec::new();
    for rust_line in out.split_inclusive('\n') {
        while let Some(&(start, line)) = next.peek() {
            if start > offset {
                break;
            }
            source_line = line;
            next.next();
        }
        line_map.push(source_line);
        offset += rust_line.len();
    }
    temporal::append_runtime(&mut out);
    strings::append_runtime(&mut out);
    arrays::append_runtime(&mut out);
    append_clock_runtime(&mut out);
    for header in parser.c_headers {
        let scope = header.alias.as_deref().unwrap_or("");
        out.push_str(&format!("{C_HEADER_USAGE_PREFIX}{scope}|{}*/\n", header.name));
    }
    Ok((out, line_map))
}

pub(crate) fn append_clock_runtime(out: &mut String) {
    if out.contains("csx_timestamp()") {
        out.push_str(
            "fn csx_timestamp() -> i64 {\n\
             static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();\n\
             ORIGIN.get_or_init(std::time::Instant::now).elapsed().as_nanos().min(i64::MAX as u128) as i64\n\
             }\n",
        );
    }
}

// Scalar statements in an object-path Main use the same parser and emitter.
pub(crate) fn emit_constant_expression(mut tokens: Vec<Tok>, required: Ty) -> Result<String, String> {
    tokens.push(Tok::Eof);
    let lines = vec![Position::START; tokens.len()];
    let mut parser = Parser::new(tokens, lines);
    let expression = parser.expr(0)?;
    if parser.peek() != &Tok::Eof {
        return Err("CSX1110: trailing tokens in return expression".into());
    }
    let (code, kind) = emit_expr(&expression, &Default::default())?;
    if kind != required {
        return Err("CSX2104: return expression has the wrong type".into());
    }
    Ok(code)
}

pub(crate) fn emit_scalar_statement(
    mut tokens: Vec<Tok>,
    vars: &mut std::collections::BTreeMap<String, Ty>,
    object_names: &std::collections::BTreeSet<String>,
    out: &mut String,
) -> Result<(), String> {
    tokens.push(Tok::Eof);
    let lines = vec![Position::START; tokens.len()];
    let mut parser = Parser::new(tokens, lines);
    let statement = parser.stmt()?;
    if let Stmt::Let(name, _, _) = &statement.kind {
        if object_names.contains(name) {
            return Err(format!("CSX2301: duplicate variable {name}"));
        }
    }
    if parser.peek() != &Tok::Eof {
        return Err("CSX1110: trailing tokens in statement".into());
    }
    emit_stmts(&[statement], vars, out, &mut Vec::new(), 1)
}

#[cfg(test)]mod tests{use super::*;#[test]fn basic(){assert!(compile("public static void Main(){int x=2+3*4;Console.WriteLine(x);}").unwrap().contains("print!"));}#[test]fn type_error(){assert!(compile("public static void Main(){int x=2;x=\"bad\";}").is_err());}#[test]fn exception(){assert!(compile("public static void Main(){try{throw \"oops\";}catch(Exception ex){Console.WriteLine(ex);}finally{Console.WriteLine(1);}}").is_ok());}}

#[cfg(test)]
mod linq_tests {
    use super::*;

    #[test]
    fn emits_integer_filter_projection_and_sum() {
        let rust = compile(
            "using System.Linq; void Main() { int n = Enumerable.Range(1, 5)
                .Where(x => x % 2 == 0).Select(x => x * 3).Sum(); Console.WriteLine(n); }",
        ).unwrap();
        assert!(rust.contains("checked_rem"));
        assert!(rust.contains("continue;"));
        assert!(rust.contains("OverflowException"));
    }

    #[test]
    fn reports_invalid_operator_and_operand() {
        let invalid = compile("void Main() { int n = Enumerable.Range(1, 5).OrderBy(x => x).Count(); }");
        assert!(invalid.unwrap_err().contains("CSX1304"));
        let invalid = compile("void Main() { int n = Enumerable.Range(1, 5).Select(x => \"text\").Sum(); }");
        assert!(invalid.unwrap_err().contains("CSX1305"));
    }
}
#[cfg(test)]
mod pointer_tests {
    use super::compile;

    #[test]
    fn rejects_pointer_operations_in_safe_code() {
        for body in [
            "int x = 1; int* p = &x;",
            "int x = 1; var p = &x;",
            "int x = 1; Console.WriteLine(*x);",
            "int x = 1; *x = 2;",
            "unsafe { int x = 1; } int y = 2; var p = &y;",
        ] {
            let error = compile(&format!("void Main() {{ {body} }}")).unwrap_err();
            assert!(error.contains("CSX1901"), "{error}");
        }
    }

    #[test]
    fn emits_typed_reads_and_writes_in_unsafe_blocks() {
        let output = compile(
            "void Main() { int value = 10; unsafe { int* p = &value; *p = *p + 32; } }",
        )
        .unwrap();
        assert!(output.contains("std::ptr::addr_of_mut!(v_value)"));
        assert!(output.contains("std::ptr::read"));
        assert!(output.contains("std::ptr::write"));
        let error = compile(
            "void Main() { int value = 10; unsafe { int* p = &value; *p = true; } }",
        )
        .unwrap_err();
        assert!(error.contains("CSX1903"), "{error}");
    }
}
