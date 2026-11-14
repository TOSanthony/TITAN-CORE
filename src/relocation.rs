//! Recoverable, non-overwriting move into the proven homebrew launch workflow.
use std::{fs,io::{self,Write},path::{Path,PathBuf}};
use serde_json::{json,Value};
use std::os::unix::fs::PermissionsExt;
pub const ROOT:&str="/data/homebrew/titan-core";
fn regular(path:&Path)->bool{fs::symlink_metadata(path).is_ok_and(|m|m.is_file()&&!m.file_type().is_symlink())}
fn save(dir:&Path,v:&Value)->io::Result<()>{let tmp=dir.join("receipt.next");let mut f=fs::File::create(&tmp)?;f.write_all(v.to_string().as_bytes())?;f.sync_all()?;fs::rename(tmp,dir.join("receipt.json"))}
fn valid_receipt(v:&Value,tid:&str)->bool{v["owner"]=="TITAN CORE"&&v["title_id"]==tid&&v["size"].as_u64().is_some()&&v["original"].as_str().is_some_and(|p|crate::library::installed_package_path(Path::new(p))&&Path::new(p).parent().and_then(Path::file_name).is_some_and(|n|n==tid))}
pub fn move_package(src:&Path,tid:&str,size:u64,root:&Path)->io::Result<PathBuf>{
    if !crate::scan::valid_tid(tid)||!regular(src)||fs::metadata(src)?.len()!=size{return Err(io::Error::other("Package source is unavailable, changed, or not a regular file"));}
    if fs::symlink_metadata(root).is_ok_and(|m|m.file_type().is_symlink()){return Err(io::Error::other("Import root is a symlink"));}fs::create_dir_all(root)?;
    let dir=root.join(tid);let dst=dir.join(format!("{tid}.pkg"));
    match fs::create_dir(&dir){Ok(())=>{},Err(e)if e.kind()==io::ErrorKind::AlreadyExists=>{
        if fs::symlink_metadata(&dir)?.file_type().is_symlink(){return Err(io::Error::other("Import directory is a symlink"));}
        let v:Value=serde_json::from_slice(&fs::read(dir.join("receipt.json"))?).map_err(io::Error::other)?;
        if !valid_receipt(&v,tid)||v["original"]!=src.to_string_lossy().as_ref(){return Err(io::Error::other("Import directory belongs to another source"));}
    },Err(e)=>return Err(e)}
    if fs::symlink_metadata(&dst).is_ok(){return Err(io::Error::other("Destination already exists; no package is overwritten"));}
    fs::set_permissions(&dir,fs::Permissions::from_mode(0o755))?;
    let mut receipt=json!({"owner":"TITAN CORE","title_id":tid,"original":src.to_string_lossy(),"size":size,"original_mode":fs::metadata(src)?.permissions().mode() & 0o7777,"phase":"pending"});save(&dir,&receipt)?;
    match fs::rename(src,&dst){Ok(())=>{receipt["method"]=json!("atomic-rename");},Err(e)if e.raw_os_error()==Some(libc_exdev())=>{
        // /data is normally a nullfs alias of /user/data. Prove the parent identity before
        // trying the same rename through that alias; never assume it resolves elsewhere.
        let alias=dst.to_str().and_then(|p|p.strip_prefix("/data/")).map(|p|PathBuf::from(format!("/user/data/{p}")));
        let renamed=alias.filter(|a|same_directory(a.parent().unwrap(),dst.parent().unwrap())).is_some_and(|a|fs::rename(src,a).is_ok());
        if renamed{receipt["method"]=json!("atomic-user-data-alias");}else{
            return Err(io::Error::new(io::ErrorKind::CrossesDevices,"Single-copy import: direct rename is unavailable; original package remains in place. No full package copy was created."));
        }
    },Err(e)=>return Err(e)}
    // Imported bytes must be readable by the game process, including when the installed
    // source originally carried owner-only permissions. The source mode is in the receipt.
    fs::set_permissions(&dst,fs::Permissions::from_mode(0o644))?;
    receipt["phase"]=json!("moved");save(&dir,&receipt)?;Ok(dst)
}
fn libc_exdev()->i32{18}
fn same_directory(a:&Path,b:&Path)->bool{use std::os::unix::fs::MetadataExt;match(fs::metadata(a),fs::metadata(b)){(Ok(a),Ok(b))=>a.is_dir()&&b.is_dir()&&a.ino()==b.ino(),_=>false}}
/// A receipt may remain pending if power failed just after rename. Validate the exact final
/// path, size and absent old source, then let the normal PKG parser validate its title.
pub fn scan_roots(root:&Path)->Vec<String>{let mut roots=Vec::new();let Ok(entries)=fs::read_dir(root)else{return roots};for e in entries.flatten(){let dir=e.path();let tid=e.file_name().to_string_lossy().into_owned();if !crate::scan::valid_tid(&tid)||fs::symlink_metadata(&dir).is_ok_and(|m|m.file_type().is_symlink()){continue}let Ok(bytes)=fs::read(dir.join("receipt.json"))else{continue};let Ok(v)=serde_json::from_slice::<Value>(&bytes)else{continue};let pkg=dir.join(format!("{tid}.pkg"));if valid_receipt(&v,&tid)&&regular(&pkg)&&fs::metadata(&pkg).is_ok_and(|m|Some(m.len())==v["size"].as_u64())&&v["original"].as_str().is_some_and(|p|!Path::new(p).exists())&&matches!(v["phase"].as_str(),Some("pending"|"moved")){if fs::metadata(&dir).is_ok_and(|m|m.permissions().mode()&0o777!=0o755)&&fs::set_permissions(&dir,fs::Permissions::from_mode(0o755)).is_err(){continue}
if fs::metadata(&pkg).is_ok_and(|m|m.permissions().mode()&0o777!=0o644)&&fs::set_permissions(&pkg,fs::Permissions::from_mode(0o644)).is_err(){continue}
roots.push(dir.to_string_lossy().into_owned());}}roots.sort();roots}
#[cfg(test)]mod tests{use super::*;
fn root(s:&str)->PathBuf{std::env::temp_dir().join(format!("titan-import-{}-{s}",std::process::id()))}
#[test]fn rename_preserves_bytes_and_refuses_overwrite(){let r=root("rename");fs::create_dir_all(&r).unwrap();let src=r.join("original.pkg");fs::write(&src,b"owner bytes").unwrap();let dst=move_package(&src,"PPSA12345",11,&r.join("dest")).unwrap();assert!(!src.exists());assert_eq!(fs::read(&dst).unwrap(),b"owner bytes");assert_eq!(fs::metadata(&dst).unwrap().permissions().mode()&0o777,0o644);fs::write(&src,b"new content").unwrap();assert!(move_package(&src,"PPSA12345",11,&r.join("dest")).is_err());assert_eq!(fs::read(dst).unwrap(),b"owner bytes");assert_eq!(fs::read(src).unwrap(),b"new content");fs::remove_dir_all(r).unwrap();}
#[test]fn pending_rename_receipt_recovers_only_exact_final_size(){let r=root("recover");let dir=r.join("PPSA12345");fs::create_dir_all(&dir).unwrap();fs::write(dir.join("PPSA12345.pkg"),b"bytes").unwrap();fs::set_permissions(dir.join("PPSA12345.pkg"),fs::Permissions::from_mode(0o600)).unwrap();save(&dir,&json!({"owner":"TITAN CORE","title_id":"PPSA12345","original":"/user/app/PPSA12345/app.pkg","size":5,"phase":"pending"})).unwrap();assert_eq!(scan_roots(&r),vec![dir.to_string_lossy().into_owned()]);assert_eq!(fs::metadata(dir.join("PPSA12345.pkg")).unwrap().permissions().mode()&0o777,0o644);fs::write(dir.join("PPSA12345.pkg"),b"changed").unwrap();assert!(scan_roots(&r).is_empty());fs::remove_dir_all(r).unwrap();}
#[cfg(target_os="linux")]
#[test]fn cross_filesystem_refusal_preserves_original_and_creates_no_package_copy(){use std::os::unix::fs::MetadataExt;let r=root("cross-device");fs::create_dir_all(&r).unwrap();let src=PathBuf::from(format!("/dev/shm/titan-one-copy-{}.pkg",std::process::id()));assert_ne!(fs::metadata("/dev/shm").unwrap().dev(),fs::metadata(&r).unwrap().dev());fs::write(&src,b"single original").unwrap();let e=move_package(&src,"PPSA12345",15,&r.join("dest")).unwrap_err();assert_eq!(e.kind(),io::ErrorKind::CrossesDevices);assert_eq!(fs::read(&src).unwrap(),b"single original");assert!(!r.join("dest/PPSA12345/PPSA12345.pkg").exists());let names=fs::read_dir(r.join("dest/PPSA12345")).unwrap().map(|e|e.unwrap().file_name().to_string_lossy().into_owned()).collect::<Vec<_>>();assert_eq!(names,vec!["receipt.json"]);fs::remove_file(src).unwrap();fs::remove_dir_all(r).unwrap();}
#[test]fn incomplete_or_foreign_receipts_are_not_scanned(){let r=root("invalid");fs::create_dir_all(r.join("PPSA12345")).unwrap();fs::write(r.join("PPSA12345/PPSA12345.pkg"),b"owner").unwrap();assert!(scan_roots(&r).is_empty());fs::write(r.join("PPSA12345/receipt.json"),b"{}").unwrap();assert!(scan_roots(&r).is_empty());fs::remove_dir_all(r).unwrap();}
}
