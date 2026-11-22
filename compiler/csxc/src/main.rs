#![forbid(unsafe_code)]
mod language;
mod objects;
use std::{env,fs,path::PathBuf,process::{Command,ExitCode}};
fn run()->Result<(),String>{
 let args:Vec<String>=env::args().collect();if args.len()!=3||!["check","emit-rust","build","ownership-report"].contains(&args[1].as_str()){return Err("Usage: csxc <check|emit-rust|build|ownership-report> <file.csx>".into())}
 let p=PathBuf::from(&args[2]);if p.extension().and_then(|s|s.to_str())!=Some("csx"){return Err("CSX9004: expected .csx file (project compilation not yet implemented)".into())}
 let src=fs::read_to_string(&p).map_err(|e|format!("CSX9001: {e}"))?;let object_mode = src.contains("class ") && (src.contains(" : ") || src.contains("virtual ") || src.contains("override ") || src.contains("new "));
 let (rust, ownership) = if object_mode { let (r,report)=objects::compile(&src)?; (r,Some(report)) } else { (language::compile(&src)?,None) };
 match args[1].as_str(){"ownership-report"=>{if let Some(report)=ownership{print!("{report}")}else{println!("No object allocations in the supported analysis subset.")}},"check"=>println!("CSharpX syntax and subset type check passed"),"emit-rust"=>print!("{rust}"),"build"=>{let out=env::current_dir().map_err(|e|e.to_string())?.join("csx-out");fs::create_dir_all(&out).map_err(|e|e.to_string())?;let file=out.join("main.rs");fs::write(&file,rust).map_err(|e|e.to_string())?;let bin=out.join(if cfg!(windows){"csx-app.exe"}else{"csx-app"});let status=Command::new("rustc").arg("--edition=2021").arg(&file).arg("-o").arg(&bin).status().map_err(|e|format!("CSX9002: {e}"))?;if !status.success(){return Err("CSX9003: rustc failed".into())}println!("Built: {}",bin.display());},_=>unreachable!()}Ok(())
}
fn main()->ExitCode{match run(){Ok(())=>ExitCode::SUCCESS,Err(e)=>{eprintln!("{e}");ExitCode::FAILURE}}}
