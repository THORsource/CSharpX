//! Experimental object subset: single inheritance, virtual dispatch, and conservative
//! whole-method ownership classification. No unsafe Rust is emitted.
use crate::language::{lex,Tok};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone,Debug)]struct Method{name:String,virtual_:bool,override_:bool,value:i64}
#[derive(Clone,Debug)]struct Class{name:String,base:Option<String>,methods:Vec<Method>}
#[derive(Clone,Debug)]enum Action{New{ty:String,name:String,concrete:String},Alias{ty:String,name:String,from:String},Print{receiver:String,method:String}}
struct P{tokens:Vec<Tok>,i:usize}
impl P{
 fn peek(&self)->&Tok{&self.tokens[self.i]}
 fn next(&mut self)->Tok{let t=self.peek().clone();if t!=Tok::Eof{self.i+=1}t}
 fn word(&mut self,s:&str)->bool{if *self.peek()==Tok::Ident(s.into()){self.next();true}else{false}}
 fn sym(&mut self,c:char)->bool{if *self.peek()==Tok::Sym(c){self.next();true}else{false}}
 fn need(&mut self,c:char)->Result<(),String>{if self.sym(c){Ok(())}else{Err(format!("CSX2101: expected '{c}', got {:?}",self.peek()))}}
 fn ident(&mut self)->Result<String,String>{match self.next(){Tok::Ident(s)=>Ok(s),t=>Err(format!("CSX2102: expected identifier, got {t:?}"))}}
 fn expect_word(&mut self,s:&str)->Result<(),String>{if self.word(s){Ok(())}else{Err(format!("CSX2103: expected {s}"))}}
 fn modifiers(&mut self)->(bool,bool){let(mut v,mut o)=(false,false);loop{if self.word("public")||self.word("private")||self.word("protected")||self.word("static")||self.word("sealed")||self.word("abstract"){}else if self.word("virtual"){v=true}else if self.word("override"){o=true}else{break}}(v,o)}
 fn class(&mut self)->Result<Class,String>{let name=self.ident()?;let base=if self.sym(':'){Some(self.ident()?)}else{None};self.need('{')?;let mut methods=vec![];while !self.sym('}'){
 let(v,o)=self.modifiers();self.expect_word("int")?;let method=self.ident()?;self.need('(')?;self.need(')')?;self.need('{')?;self.expect_word("return")?;let value=match self.next(){Tok::Num(n)=>n,_=>return Err("CSX2104: method body must return an integer literal in this subset".into())};self.need(';')?;self.need('}')?;methods.push(Method{name:method,virtual_:v,override_:o,value});
 }Ok(Class{name,base,methods})}
 fn main(&mut self)->Result<Vec<Action>,String>{self.need('(')?;self.need(')')?;self.need('{')?;let mut actions=vec![];while !self.sym('}'){
 if self.word("Console"){self.need('.')?;self.expect_word("WriteLine")?;self.need('(')?;let receiver=self.ident()?;self.need('.')?;let method=self.ident()?;self.need('(')?;self.need(')')?;self.need(')')?;self.need(';')?;actions.push(Action::Print{receiver,method});continue;}
 let ty=if self.word("var"){String::new()}else{self.ident()?};let name=self.ident()?;self.need('=')?;if self.word("new"){let concrete=self.ident()?;self.need('(')?;self.need(')')?;self.need(';')?;actions.push(Action::New{ty,name,concrete});}else{let from=self.ident()?;self.need(';')?;actions.push(Action::Alias{ty,name,from});}
 }Ok(actions)}
 fn program(&mut self)->Result<(Vec<Class>,Vec<Action>),String>{let mut classes=vec![];let mut actions=None;while self.peek()!=&Tok::Eof{
 let _=self.modifiers();if self.word("class"){classes.push(self.class()?)}else if self.word("void"){self.expect_word("Main")?;if actions.is_some(){return Err("CSX2105: duplicate Main".into())}actions=Some(self.main()?)}else{return Err(format!("CSX2106: expected class or void Main, got {:?}",self.peek()))}
 }Ok((classes,actions.ok_or("CSX2107: Main not found")?))}
}
fn ancestor(classes:&BTreeMap<String,Class>,child:&str,parent:&str)->bool{let mut cur=Some(child.to_string());let mut visited=BTreeSet::new();while let Some(c)=cur{if !visited.insert(c.clone()){return false}if c==parent{return true}cur=classes.get(&c).and_then(|x|x.base.clone())}false}
fn find_method<'a>(classes:&'a BTreeMap<String,Class>,ty:&str,name:&str)->Option<(&'a str,&'a Method)>{let mut cur=Some(ty.to_string());while let Some(c)=cur{let cls=classes.get(&c)?;if let Some(m)=cls.methods.iter().find(|m|m.name==name){return Some((classes.get_key_value(&c)?.0.as_str(),m))}cur=cls.base.clone()}None}
fn safe_ident(s:&str)->bool{!s.is_empty()&&s.chars().all(|c|c.is_ascii_alphanumeric()||c=='_')&&!s.starts_with(|c:char|c.is_ascii_digit())}
pub fn compile(src:&str)->Result<(String,String),String>{let mut p=P{tokens:lex(src)?,i:0};let (defs,actions)=p.program()?;let mut classes=BTreeMap::new();for c in defs{if !safe_ident(&c.name)||classes.insert(c.name.clone(),c).is_some(){return Err("CSX2201: duplicate/invalid class".into())}}
 for c in classes.values(){if let Some(b)=&c.base{if !classes.contains_key(b){return Err(format!("CSX2202: unknown base class {b}"))}if ancestor(&classes,b,&c.name){return Err("CSX2203: inheritance cycle".into())}}let mut names=BTreeSet::new();for m in &c.methods{if !names.insert(&m.name){return Err("CSX2204: duplicate method".into())}let inherited=c.base.as_ref().and_then(|b|find_method(&classes,b,&m.name));if m.override_&&!inherited.map(|(_,m)|m.virtual_||m.override_).unwrap_or(false){return Err(format!("CSX2205: invalid override {}.{}",c.name,m.name))}if inherited.is_some()&&!m.override_{return Err(format!("CSX2206: method hiding is not implemented: {}.{}",c.name,m.name))}}}
 // The variable environment tracks the declared static type and the originating allocation.
 let mut vars:BTreeMap<String,(String,String)>=BTreeMap::new();let mut allocation_aliases:BTreeMap<String,usize>=BTreeMap::new();for a in &actions{match a{
 Action::New{ty,name,concrete}=>{if vars.contains_key(name){return Err(format!("CSX2301: duplicate variable {name}"))}if !classes.contains_key(concrete){return Err(format!("CSX2302: unknown class {concrete}"))}let static_ty=if ty.is_empty(){concrete}else{ty};if !ancestor(&classes,concrete,static_ty){return Err(format!("CSX2303: cannot convert {concrete} to {static_ty}"))}vars.insert(name.clone(),(static_ty.clone(),name.clone()));allocation_aliases.insert(name.clone(),1);},
 Action::Alias{ty,name,from}=>{if vars.contains_key(name){return Err(format!("CSX2301: duplicate variable {name}"))}let (from_ty,root)=vars.get(from).ok_or_else(||format!("CSX2304: unknown variable {from}"))?.clone();let static_ty=if ty.is_empty(){&from_ty}else{ty};if !ancestor(&classes,&from_ty,static_ty){return Err(format!("CSX2303: cannot convert {from_ty} to {static_ty}"))}*allocation_aliases.get_mut(&root).unwrap()+=1;vars.insert(name.clone(),(static_ty.clone(),root));},
 Action::Print{receiver,method}=>{let (ty,_)=vars.get(receiver).ok_or_else(||format!("CSX2304: unknown variable {receiver}"))?;if find_method(&classes,ty,method).is_none(){return Err(format!("CSX2305: method {ty}.{method} not found"))}}
 }}
 // Object representation is a safe tagged enum; no raw vtables or unsafe casts.
 let mut rust=String::from("#![forbid(unsafe_code)]\nuse std::rc::Rc;\n#[derive(Debug)]\nenum CsxObject {\n");for name in classes.keys(){rust.push_str(&format!("    {name},\n"))}rust.push_str("}\nimpl CsxObject {\n");
 let methods:BTreeSet<String>=classes.values().flat_map(|c|c.methods.iter().map(|m|m.name.clone())).collect();for method in methods{rust.push_str(&format!("  fn csx_{method}(&self) -> i64 {{\n    match self {{\n"));for name in classes.keys(){if let Some((_,m))=find_method(&classes,name,&method){rust.push_str(&format!("      Self::{name} => {},\n",m.value))}else{rust.push_str(&format!("      Self::{name} => panic!(\"invalid dispatch\"),\n"))}}rust.push_str("    }\n  }\n");}rust.push_str("}\nfn main() {\n");
 let mut env:BTreeMap<String,String>=BTreeMap::new();let mut report=String::from("# CSharpX Ownership Analysis (experimental)\n\nThis is a conservative, straight-line alias analysis. No borrowing, cycles, mutation or escape analysis yet.\n\n| Allocation | Strategy | Aliases |\n|---|---|---|\n");for a in &actions{match a{
 Action::New{name,concrete,..}=>{let count=allocation_aliases[name];if count==1{rust.push_str(&format!("  let v_{name} = Box::new(CsxObject::{concrete});\n"));report.push_str(&format!("| {name} | Box (exclusive) | 1 |\n"));}else{rust.push_str(&format!("  let v_{name} = Rc::new(CsxObject::{concrete});\n"));report.push_str(&format!("| {name} | Rc (shared) | {count} |\n"));}env.insert(name.clone(),name.clone());},
 Action::Alias{name,from,..}=>{let root=env.get(from).unwrap().clone();rust.push_str(&format!("  let v_{name} = Rc::clone(&v_{from});\n"));env.insert(name.clone(),root);},
 Action::Print{receiver,method}=>rust.push_str(&format!("  println!(\"{{}}\", v_{receiver}.csx_{method}());\n"))
 }}rust.push_str("}\n");Ok((rust,report))}
#[cfg(test)]mod tests{use super::*;
 const GOOD:&str="class Animal { public virtual int Speak() { return 1; } } class Dog : Animal { public override int Speak() { return 2; } } public static void Main() { Animal a = new Dog(); Animal b = a; Console.WriteLine(b.Speak()); }";
 #[test]fn inheritance_and_shared(){let (r,report)=compile(GOOD).unwrap();assert!(r.contains("Self::Dog => 2"));assert!(r.contains("Rc::clone"));assert!(report.contains("Rc (shared)"));}
 #[test]fn exclusive(){let (r,report)=compile("class A { public int Get() { return 3; } } void Main() { A a = new A(); Console.WriteLine(a.Get()); }").unwrap();assert!(r.contains("Box::new"));assert!(report.contains("Box (exclusive)"));}
 #[test]fn invalid_override(){assert!(compile("class A { public int Get() { return 3; } } class B : A { public override int Get() { return 4; } } void Main() {}").is_err());}
 #[test]fn cycle(){assert!(compile("class A : B {} class B : A {} void Main() {}").is_err());}
}
