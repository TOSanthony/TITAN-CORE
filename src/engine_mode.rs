//! One active mount owner per boot. A requested change never hot-switches engines.
use std::fs::{self,OpenOptions};
use std::io::Write;
const CONFIG:&str="/data/titan-core/engine-mode";
pub fn parse(value:&str)->Result<&str,String>{match value.trim(){"native"=>Ok("native"),"images"=>Ok("images"),_=>Err("Engine mode must be native or images".into())}}
pub fn requested()->Result<String,String>{match fs::read_to_string(CONFIG){Ok(s)=>Ok(parse(&s)?.into()),Err(e)if e.kind()==std::io::ErrorKind::NotFound=>Ok("native".into()),Err(e)=>Err(e.to_string())}}
pub fn active(boot:Option<(u64,u64)>)->Result<String,String>{
    let requested=requested()?;let path=crate::stack_policy::attempt_name(boot).replace(".attempted",".owner");
    pin(std::path::Path::new(&path),&requested)
}
fn pin(path:&std::path::Path,requested:&str)->Result<String,String>{
    match OpenOptions::new().write(true).create_new(true).open(&path){
        Ok(mut file)=>{file.write_all(requested.as_bytes()).map_err(|e|e.to_string())?;Ok(requested.into())},
        Err(e)if e.kind()==std::io::ErrorKind::AlreadyExists=>Ok(parse(&fs::read_to_string(path).map_err(|e|e.to_string())?)?.into()),
        Err(e)=>Err(format!("Boot ownership file unavailable: {e}")),
    }
}
pub fn queue(mode:&str)->Result<(),String>{
    let mode=parse(mode)?;fs::create_dir_all("/data/titan-core").map_err(|e|e.to_string())?;
    let temp=format!("{CONFIG}.{}.tmp",std::process::id());let mut f=OpenOptions::new().write(true).create_new(true).open(&temp).map_err(|e|e.to_string())?;
    f.write_all(mode.as_bytes()).and_then(|_|f.sync_all()).and_then(|_|fs::rename(&temp,CONFIG)).map_err(|e|e.to_string())
}
#[cfg(test)]mod tests{use super::*;
#[test]fn profile_change_cannot_hot_switch_owner(){let path=std::env::temp_dir().join(format!("titan-owner-{}",std::process::id()));assert_eq!(pin(&path,"native"),Ok("native".into()));assert_eq!(pin(&path,"images"),Ok("native".into()));std::fs::remove_file(path).unwrap();}
#[test]fn only_two_explicit_profiles(){assert_eq!(parse("native\n"),Ok("native"));assert_eq!(parse("images"),Ok("images"));assert!(parse("auto").is_err());assert!(parse("../images").is_err());}}
