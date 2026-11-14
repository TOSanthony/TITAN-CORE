//! Small metadata snapshot before re-registering an installed package through native AMPR.
use std::{fs,io,path::Path};
const LIMIT:u64=64<<20;
fn copy(src:&Path,dst:&Path,bytes:&mut u64)->io::Result<()>{
    let m=match fs::symlink_metadata(src){Ok(m)=>m,Err(e)if e.kind()==io::ErrorKind::NotFound=>return Ok(()),Err(e)=>return Err(e)};
    if m.file_type().is_symlink(){return Err(io::Error::other("Metadata snapshot refuses symlinks"));}
    if m.is_dir(){fs::create_dir_all(dst)?;for e in fs::read_dir(src)?{let e=e?;copy(&e.path(),&dst.join(e.file_name()),bytes)?;}}
    else if m.is_file(){*bytes=bytes.checked_add(m.len()).filter(|b|*b<=LIMIT).ok_or_else(||io::Error::other("Metadata snapshot exceeds 64 MiB"))?;if dst.exists(){return Err(io::Error::other("Metadata snapshot destination exists"));}if let Some(p)=dst.parent(){fs::create_dir_all(p)?;}fs::copy(src,dst)?;fs::File::options().write(true).open(dst)?.sync_all()?;}
    Ok(())
}
pub fn snapshot(app:&Path,meta:&Path,dest:&Path)->io::Result<()>{
    if dest.join("complete").is_file(){return Ok(())}
    if dest.exists(){return Err(io::Error::other("Incomplete metadata backup; preserved for inspection"));}
    let parent=dest.parent().ok_or_else(||io::Error::other("Invalid backup directory"))?;fs::create_dir_all(parent)?;
    let mut chosen=None;for i in 0..1000{let temp=parent.join(format!("pending-{}-{}-{i}",dest.file_name().unwrap().to_string_lossy(),std::process::id()));match fs::create_dir(&temp){Ok(())=>{chosen=Some(temp);break},Err(e)if e.kind()==io::ErrorKind::AlreadyExists=>{},Err(e)=>return Err(e)}}
    let temp=chosen.ok_or_else(||io::Error::other("No metadata snapshot directory available"))?;let mut bytes=0;
    copy(&app.join("sce_sys"),&temp.join("app/sce_sys"),&mut bytes)?;copy(&app.join("icon0.png"),&temp.join("app/icon0.png"),&mut bytes)?;copy(meta,&temp.join("appmeta"),&mut bytes)?;
    fs::write(temp.join("complete"),b"Metadata only. Original app.pkg is not moved or copied.\n")?;fs::File::options().write(true).open(temp.join("complete"))?.sync_all()?;fs::rename(temp,dest)
}
pub fn installed(tid:&str)->io::Result<()>{snapshot(Path::new("/user/app").join(tid).as_path(),Path::new("/user/appmeta").join(tid).as_path(),Path::new("/data/titan-core/installed-backups").join(tid).as_path())}
#[cfg(test)]mod tests{use super::*;
#[test]fn snapshot_excludes_package_and_preserves_original_metadata(){let r=std::env::temp_dir().join(format!("titan-backup-{}",std::process::id()));let app=r.join("app");let meta=r.join("meta");let dest=r.join("backup");fs::create_dir_all(app.join("sce_sys")).unwrap();fs::create_dir_all(&meta).unwrap();fs::write(app.join("app.pkg"),b"owner-package").unwrap();fs::write(app.join("sce_sys/param.json"),b"original").unwrap();fs::write(meta.join("icon0.png"),b"icon").unwrap();snapshot(&app,&meta,&dest).unwrap();assert!(!dest.join("app/app.pkg").exists());assert_eq!(fs::read(app.join("app.pkg")).unwrap(),b"owner-package");fs::write(app.join("sce_sys/param.json"),b"new").unwrap();snapshot(&app,&meta,&dest).unwrap();assert_eq!(fs::read(dest.join("app/sce_sys/param.json")).unwrap(),b"original");fs::remove_dir_all(r).unwrap();}
}
