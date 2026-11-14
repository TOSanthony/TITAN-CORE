//! Internal-storage installation of supported external debug PKGs; never deletes the USB source.
use std::{fs,io::{self,Write},path::{Path,PathBuf},sync::{Mutex,atomic::{AtomicBool,Ordering}}};
use crate::installer_io::{transfer,verify};
use serde_json::{json,Value};
use std::os::unix::fs::{OpenOptionsExt,PermissionsExt};
pub const ROOT:&str="/data/homebrew/titan-core/installed";
#[derive(Default)]pub struct Installer{pub job:Mutex<Value>,pending:Mutex<Option<PathBuf>>,cancel:AtomicBool}
impl Installer{
 pub fn status(&self)->Value{self.job.lock().unwrap().clone()}
 pub fn active(&self)->bool{matches!(self.status()["phase"].as_str(),Some("queued"|"copying"|"verifying"))}
 pub fn cancel(&self){self.cancel.store(true,Ordering::SeqCst);}
 pub fn queue(&self,path:PathBuf)->io::Result<()>{let mut job=self.job.lock().unwrap();if matches!(job["phase"].as_str(),Some("queued"|"copying"|"verifying")){return Err(io::Error::other("An installation is already active"));}self.cancel.store(false,Ordering::SeqCst);*self.pending.lock().unwrap()=Some(path.clone());*job=json!({"phase":"queued","path":path,"done":0,"total":0});Ok(())}
 fn progress(&self,phase:&str,done:u64,total:u64){let mut j=self.job.lock().unwrap();j["phase"]=json!(phase);j["done"]=json!(done);j["total"]=json!(total);}
 pub fn run(&self,state:&crate::web::State){let Some(path)=self.pending.lock().unwrap().take()else{return};let result=(||->io::Result<()>{
  let source=external(&path,false)?;let meta=fs::metadata(&source)?;let size=meta.len();let (tid,name,_)=crate::cache::peek_pkg(&source)?;if !tid.starts_with("PPSA")||!crate::scan::valid_tid(&tid){return Err(io::Error::other("Only supported PS5 debug PKGs can be installed"));}
  let free=crate::storage::drives().iter().find(|d|d.mount=="/data").map_or(0,|d|d.free);if free<size.saturating_add(256<<20){return Err(io::Error::other("Not enough internal storage for PKG and working reserve"));}
  if state.games.lock().unwrap().iter().any(|g|g.src.title_id==tid && g.src.drive=="/data"){return Err(io::Error::other("This title already has an internal source; nothing was overwritten"));}
  let root=Path::new(ROOT);fs::create_dir_all(root)?;if root.canonicalize()?!=root{return Err(io::Error::other("Installation root must not contain symlinks"));}
  let dir=root.join(&tid);fs::create_dir(&dir)?;fs::set_permissions(&dir,fs::Permissions::from_mode(0o755))?;
  // Reservation never reuses an existing directory. On error only this transaction's files are removed.
  let result=(||->io::Result<()>{let tmp=dir.join("package.part");let dst=dir.join(format!("{tid}.pkg"));let mut input=fs::File::open(&source)?;let mut out=fs::OpenOptions::new().write(true).create_new(true).mode(0o644).open(&tmp)?;
   self.progress("copying",0,size);transfer(&mut input,&mut out,size,|done|{if self.cancel.load(Ordering::SeqCst)||state.paused(){return Err(io::Error::other("Installation stopped; external source kept"));}self.progress("copying",done,size);Ok(())})?;out.sync_all()?;drop(out);
   self.progress("verifying",0,size);verify(&source,&tmp,size,|done|{if self.cancel.load(Ordering::SeqCst)||state.paused(){return Err(io::Error::other("Installation stopped; external source kept"));}self.progress("verifying",done,size);Ok(())})?;
   if fs::metadata(&source)?.modified()?!=meta.modified()?||fs::metadata(&source)?.len()!=size{return Err(io::Error::other("External source changed during installation"));}
   let (read_tid,_,_)=crate::cache::peek_pkg(&tmp)?;if read_tid!=tid{return Err(io::Error::other("Copied package title mismatch"));}
   let receipt=json!({"owner":"TITAN CORE installer","title_id":tid,"size":size,"source":source});let mut f=fs::OpenOptions::new().write(true).create_new(true).open(dir.join("install.json"))?;f.write_all(receipt.to_string().as_bytes())?;f.sync_all()?;
   fs::rename(&tmp,&dst)?;let mut job=self.job.lock().unwrap();*job=json!({"phase":"complete","title_id":tid,"name":name,"path":dst,"done":size,"total":size});Ok(())})();
  if result.is_err(){let _=fs::remove_file(dir.join("package.part"));let _=fs::remove_file(dir.join("install.json"));let _=fs::remove_dir(&dir);}result
 })();if let Err(e)=result{log!("PKG installation {}: {e}",path.display());let mut j=self.job.lock().unwrap();j["phase"]=json!("error");j["error"]=json!(e.to_string());}state.wake();}
}
pub fn external(path:&Path,directory:bool)->io::Result<PathBuf>{
 if path.components().any(|c|!matches!(c,std::path::Component::RootDir|std::path::Component::Normal(_))){return Err(io::Error::other("Invalid external path"));}
 let canonical=path.canonicalize()?;if canonical!=path{return Err(io::Error::other("External symlinks are not supported"));}
 let allowed=crate::storage::drives().iter().filter(|d|d.kind!="internal").any(|d|canonical.starts_with(&d.mount));if !allowed{return Err(io::Error::other("Select a connected USB or extended drive"));}
 let m=fs::metadata(&canonical)?;if directory&&!m.is_dir()||!directory&&(!m.is_file()||!canonical.extension().is_some_and(|x|x.eq_ignore_ascii_case("pkg"))){return Err(io::Error::other("Select a PKG file or folder"));}Ok(canonical)
}
pub fn browse(path:&str)->io::Result<Value>{let drives=crate::storage::drives();let external_drives=drives.iter().filter(|d|d.kind!="internal").map(|d|json!({"name":d.label,"path":d.mount,"directory":true})).collect::<Vec<_>>();if path.is_empty(){return Ok(json!({"path":"","entries":external_drives}));}let dir=external(Path::new(path),true)?;let mut entries=Vec::new();for entry in fs::read_dir(&dir)?.take(1025){let e=entry?;let m=e.file_type()?;if m.is_symlink(){continue;}let p=e.path();if m.is_dir()||p.extension().is_some_and(|x|x.eq_ignore_ascii_case("pkg")){entries.push(json!({"name":e.file_name().to_string_lossy(),"path":p,"directory":m.is_dir(),"size":if m.is_file(){e.metadata()?.len()}else{0}}));}}entries.sort_by_key(|e|(!e["directory"].as_bool().unwrap_or(false),e["name"].as_str().unwrap_or("").to_lowercase()));Ok(json!({"path":dir,"entries":entries,"limit":1025}))}
pub fn roots()->Vec<String>{let mut roots=Vec::new();let Ok(entries)=fs::read_dir(ROOT)else{return roots};for e in entries.flatten(){let dir=e.path();let tid=e.file_name().to_string_lossy().into_owned();if !crate::scan::valid_tid(&tid)||dir.canonicalize().ok().as_ref()!=Some(&dir){continue;}let Ok(data)=fs::read(dir.join("install.json"))else{continue};let Ok(v)=serde_json::from_slice::<Value>(&data)else{continue};let pkg=dir.join(format!("{tid}.pkg"));if v["owner"]=="TITAN CORE installer"&&v["title_id"]==tid&&fs::symlink_metadata(&pkg).is_ok_and(|m|m.is_file()&&Some(m.len())==v["size"].as_u64()){roots.push(dir.to_string_lossy().into_owned());}}roots.sort();roots}

/// Remove only an empty, owned installation receipt after the user's confirmed source deletion.
pub fn forget_empty(pkg:&Path)->io::Result<()>{let Some(dir)=pkg.parent()else{return Ok(())};if dir.parent()!=Some(Path::new(ROOT))||dir.canonicalize().ok().as_deref()!=Some(dir)||pkg.exists(){return Ok(());}let tid=dir.file_name().and_then(|n|n.to_str()).unwrap_or("");if !crate::scan::valid_tid(tid)||pkg.file_name().and_then(|n|n.to_str())!=Some(&format!("{tid}.pkg")){return Ok(());}let receipt=dir.join("install.json");let v:Value=serde_json::from_slice(&fs::read(&receipt)?).map_err(io::Error::other)?;if v["owner"]!="TITAN CORE installer"||v["title_id"]!=tid{return Ok(());}let entries=fs::read_dir(dir)?.collect::<Result<Vec<_>,_>>()?;if entries.len()==1&&entries[0].path()==receipt{fs::remove_file(receipt)?;fs::remove_dir(dir)?;}Ok(())}
