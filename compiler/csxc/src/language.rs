#![forbid(unsafe_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum Tok { Ident(String), Num(i64), Str(String), Sym(char), Op(String), Eof }
fn ident_start(c: char)->bool {c.is_ascii_alphabetic()||c=='_'}
fn ident_part(c: char)->bool {ident_start(c)||c.is_ascii_digit()}
pub fn lex(src:&str)->Result<Vec<Tok>,String>{
 let mut it=src.chars().peekable();let mut out=Vec::new();
 while let Some(c)=it.next(){
  if c.is_whitespace(){continue;}
  if c=='/' && it.peek()==Some(&'/'){for x in it.by_ref(){if x=='\n'{break}}continue;}
  if c=='/' && it.peek()==Some(&'*'){it.next();let mut prev=' ';let mut closed=false;for x in it.by_ref(){if prev=='*'&&x=='/'{closed=true;break;}prev=x;}if !closed{return Err("CSX1001: unterminated comment".into());}continue;}
  if ident_start(c){let mut s=c.to_string();while let Some(&x)=it.peek(){if ident_part(x){s.push(x);it.next();}else{break;}}out.push(Tok::Ident(s));continue;}
  if c.is_ascii_digit(){let mut s=c.to_string();while let Some(&x)=it.peek(){if x.is_ascii_digit(){s.push(x);it.next();}else{break;}}out.push(Tok::Num(s.parse().map_err(|_|"CSX1002: integer out of range")?));continue;}
  if c=='"'{let mut s=String::new();let mut closed=false;while let Some(x)=it.next(){if x=='"'{closed=true;break;}if x=='\\'{let y=it.next().ok_or("CSX1003: unfinished escape")?;s.push(match y{'n'=>'\n','r'=>'\r','t'=>'\t','"'=>'"','\\'=>'\\',_=>return Err("CSX1004: invalid escape".into())});}else{s.push(x)}}if !closed{return Err("CSX1005: unterminated string".into())}out.push(Tok::Str(s));continue;}
  if "=+-*/!<>".contains(c){if it.peek()==Some(&'='){it.next();out.push(Tok::Op(format!("{c}=")));}else{out.push(Tok::Sym(c));}continue;}
  if "{}();,.:".contains(c){out.push(Tok::Sym(c));continue;}
  return Err(format!("CSX1006: invalid character {c:?}"));
 }
 out.push(Tok::Eof);Ok(out)
}
#[derive(Clone,Debug)]pub enum Expr{Num(i64),Str(String),Var(String),Binary(Box<Expr>,char,Box<Expr>)}
#[derive(Clone,Debug)]pub enum Stmt{Let(String,Expr),Assign(String,Expr),Print(Expr),Throw(Expr),Try{body:Vec<Stmt>,catch_name:Option<String>,catch_body:Vec<Stmt>,finally_body:Vec<Stmt>}}
pub struct Parser{t:Vec<Tok>,i:usize}
impl Parser{
 pub fn new(t:Vec<Tok>)->Self{Self{t,i:0}}
 fn peek(&self)->&Tok{&self.t[self.i]}
 fn next(&mut self)->Tok{let t=self.peek().clone();if t!=Tok::Eof{self.i+=1;}t}
 fn sym(&mut self,c:char)->bool{if *self.peek()==Tok::Sym(c){self.next();true}else{false}}
 fn expect(&mut self,c:char)->Result<(),String>{if self.sym(c){Ok(())}else{Err(format!("CSX1101: expected '{c}', got {:?}",self.peek()))}}
 fn word(&mut self,s:&str)->bool{if self.peek()==&Tok::Ident(s.into()){self.next();true}else{false}}
 fn name(&mut self)->Result<String,String>{match self.next(){Tok::Ident(s)=>Ok(s),x=>Err(format!("CSX1102: expected identifier, got {x:?}"))}}
 fn primary(&mut self)->Result<Expr,String>{match self.next(){Tok::Num(n)=>Ok(Expr::Num(n)),Tok::Str(s)=>Ok(Expr::Str(s)),Tok::Ident(s)=>Ok(Expr::Var(s)),Tok::Sym('(')=>{let x=self.expr(0)?;self.expect(')')?;Ok(x)},x=>Err(format!("CSX1103: invalid expression {x:?}"))}}
 fn expr(&mut self,min:u8)->Result<Expr,String>{let mut a=self.primary()?;loop{let (op,p)=match self.peek(){Tok::Sym('+')=>('+',1),Tok::Sym('-')=>('-',1),Tok::Sym('*')=>('*',2),Tok::Sym('/')=>('/',2),_=>break};if p<min{break;}self.next();let b=self.expr(p+1)?;a=Expr::Binary(Box::new(a),op,Box::new(b));}Ok(a)}
 fn block(&mut self)->Result<Vec<Stmt>,String>{self.expect('{')?;let mut v=Vec::new();while !self.sym('}'){if self.peek()==&Tok::Eof{return Err("CSX1104: unclosed block".into())}v.push(self.stmt()?);}Ok(v)}
 fn stmt(&mut self)->Result<Stmt,String>{
  if self.word("try"){let body=self.block()?;let mut name=None;let mut cb=vec![];let mut fb=vec![];if self.word("catch"){if self.sym('('){let _ty=self.name()?;if !self.sym(')'){name=Some(self.name()?);self.expect(')')?;}}cb=self.block()?;}if self.word("finally"){fb=self.block()?;}if cb.is_empty()&&fb.is_empty(){return Err("CSX1105: try requires catch or finally".into());}return Ok(Stmt::Try{body,catch_name:name,catch_body:cb,finally_body:fb});}
  if self.word("throw"){let e=self.expr(0)?;self.expect(';')?;return Ok(Stmt::Throw(e));}
  if self.word("var")||self.word("int")||self.word("string"){let n=self.name()?;self.expect('=')?;let e=self.expr(0)?;self.expect(';')?;return Ok(Stmt::Let(n,e));}
  let first=self.name()?;
  if first=="Console"{self.expect('.')?;if !self.word("WriteLine"){return Err("CSX1106: only Console.WriteLine supported".into())}self.expect('(')?;let e=self.expr(0)?;self.expect(')')?;self.expect(';')?;return Ok(Stmt::Print(e));}
  self.expect('=')?;let e=self.expr(0)?;self.expect(';')?;Ok(Stmt::Assign(first,e))
 }
 pub fn program(&mut self)->Result<Vec<Stmt>,String>{
  // First supported top-level shape: optional class wrapper around public static void Main().
  while self.word("using"){while !self.sym(';'){if self.next()==Tok::Eof{return Err("CSX1107: invalid using".into())}}}
  if self.word("public"){}if self.word("static"){}if self.word("class"){self.name()?;self.expect('{')?;}
  if self.word("public"){}if self.word("static"){}if !self.word("void"){return Err("CSX1108: expected void Main".into())}if !self.word("Main"){return Err("CSX1109: expected Main".into())}self.expect('(')?;self.expect(')')?;let v=self.block()?;while self.sym('}'){}if self.peek()!=&Tok::Eof{return Err("CSX1110: trailing tokens".into())}Ok(v)
 }
}
#[derive(Clone,Copy,PartialEq)]enum Ty{Int,Str}
fn emit_expr(e:&Expr,vars:&std::collections::BTreeMap<String,Ty>)->Result<(String,Ty),String>{match e{
 Expr::Num(n)=>Ok((format!("{n}i64"),Ty::Int)),Expr::Str(s)=>Ok((format!("{:?}.to_string()",s),Ty::Str)),Expr::Var(n)=>{let t=vars.get(n).ok_or_else(||format!("CSX1201: unknown variable {n}"))?;Ok((format!("v_{n}.clone()"),*t))},Expr::Binary(a,op,b)=>{let (aa,at)=emit_expr(a,vars)?;let (bb,bt)=emit_expr(b,vars)?;if at!=Ty::Int||bt!=Ty::Int{return Err("CSX1202: arithmetic requires int".into())}let code=if *op=='/'{format!("csx_div({aa}, {bb})?")}else{format!("({aa} {op} {bb})")};Ok((code,Ty::Int))}
}}
fn emit_stmts(stmts:&[Stmt],vars:&mut std::collections::BTreeMap<String,Ty>,out:&mut String,depth:usize)->Result<(),String>{let pad="    ".repeat(depth);for s in stmts{match s{
 Stmt::Let(n,e)=>{if vars.contains_key(n){return Err(format!("CSX1203: duplicate variable {n}"))}let (x,t)=emit_expr(e,vars)?;vars.insert(n.clone(),t);out.push_str(&format!("{pad}let mut v_{n} = {x};\n"));},
 Stmt::Assign(n,e)=>{let expected=*vars.get(n).ok_or_else(||format!("CSX1201: unknown variable {n}"))?;let (x,t)=emit_expr(e,vars)?;if t!=expected{return Err("CSX1204: incompatible assignment".into())}out.push_str(&format!("{pad}v_{n} = {x};\n"));},
 Stmt::Print(e)=>{let (x,_)=emit_expr(e,vars)?;out.push_str(&format!("{pad}println!(\"{{}}\", {x});\n"));},
 Stmt::Throw(e)=>{let (x,_)=emit_expr(e,vars)?;out.push_str(&format!("{pad}return Err(format!(\"{{}}\", {x}));\n"));},
 Stmt::Try{body,catch_name,catch_body,finally_body}=>{let mut local=vars.clone();out.push_str(&format!("{pad}let csx_result = (|| -> Result<(), String> {{\n"));emit_stmts(body,&mut local,out,depth+1)?;out.push_str(&format!("{pad}    Ok(())\n{pad}}})();\n"));if !catch_body.is_empty(){out.push_str(&format!("{pad}let csx_result = match csx_result {{ Ok(()) => Ok(()), Err(csx_error) => {{\n"));let mut local=vars.clone();if let Some(n)=catch_name{local.insert(n.clone(),Ty::Str);out.push_str(&format!("{pad}    let mut v_{n} = csx_error;\n"));}else{out.push_str(&format!("{pad}    let _ = csx_error;\n"));}out.push_str(&format!("{pad}    (|| -> Result<(), String> {{\n"));emit_stmts(catch_body,&mut local,out,depth+2)?;out.push_str(&format!("{pad}        Ok(())\n{pad}    }})()\n{pad}}} }};\n"));}if !finally_body.is_empty(){out.push_str(&format!("{pad}let csx_finally = (|| -> Result<(), String> {{\n"));let mut local=vars.clone();emit_stmts(finally_body,&mut local,out,depth+1)?;out.push_str(&format!("{pad}    Ok(())\n{pad}}})();\n{pad}csx_finally?;\n"));}out.push_str(&format!("{pad}csx_result?;\n"));}
 }}Ok(())}
pub fn compile(src:&str)->Result<String,String>{let mut parser=Parser::new(lex(src)?);let stmts=parser.program()?;let mut out=String::from("#![forbid(unsafe_code)]\nfn csx_div(a:i64,b:i64)->Result<i64,String>{a.checked_div(b).ok_or_else(||\"DivideByZeroException or overflow\".to_string())}\nfn csx_main()->Result<(),String>{\n");emit_stmts(&stmts,&mut Default::default(),&mut out,1)?;out.push_str("    Ok(())\n}\nfn main(){if let Err(e)=csx_main(){eprintln!(\"Unhandled CSharpX exception: {}\",e);std::process::exit(1);}}\n");Ok(out)}
#[cfg(test)]mod tests{use super::*;#[test]fn basic(){assert!(compile("public static void Main(){int x=2+3*4;Console.WriteLine(x);}").unwrap().contains("println!"));}#[test]fn type_error(){assert!(compile("public static void Main(){int x=2;x=\"bad\";}").is_err());}#[test]fn exception(){assert!(compile("public static void Main(){try{throw \"oops\";}catch(Exception ex){Console.WriteLine(ex);}finally{Console.WriteLine(1);}}").is_ok());}}
