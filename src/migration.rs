//! Rename legacy app data without copying packages or overwriting owner files.
use std::{fs,io,path::Path};
fn merge(src:&Path,dst:&Path)->io::Result<()>{
    if fs::symlink_metadata(dst).is_err(){return fs::rename(src,dst)}
    if src.is_dir() && dst.is_dir() && !fs::symlink_metadata(src)?.file_type().is_symlink() && !fs::symlink_metadata(dst)?.file_type().is_symlink(){
        for e in fs::read_dir(src)?{let e=e?;merge(&e.path(),&dst.join(e.file_name()))?;}
        if fs::read_dir(src)?.next().is_none(){fs::remove_dir(src)?;}
    }
    Ok(()) // A collision stays in the legacy directory for preservation below.
}
pub fn migrate(old:&Path,new:&Path)->io::Result<Option<String>>{
    if fs::symlink_metadata(new).is_ok_and(|m|m.file_type().is_symlink()){return Err(io::Error::other("TITAN data directory is a symlink"));}
    fs::create_dir_all(new)?;
    let meta=match fs::symlink_metadata(old){Ok(m)=>m,Err(e)if e.kind()==io::ErrorKind::NotFound=>return Ok(None),Err(e)=>return Err(e)};
    if !meta.is_dir() || meta.file_type().is_symlink(){return Err(io::Error::other("Legacy data root is not a normal directory"));}
    for e in fs::read_dir(old)?{
        let e=e?;let name=e.file_name();let name=match name.to_str(){Some("kraken-mount.elf")=>"titan-core.elf".into(),Some("kraken.log")=>"titan-core.log".into(),_=>name};
        merge(&e.path(),&new.join(name))?;
    }
    if fs::read_dir(old)?.next().is_none(){fs::remove_dir(old)?;return Ok(Some("App data migrated to /data/titan-core".into()));}
    for i in 0..1000{
        let legacy=new.join(format!("legacy-import-{i}"));
        if fs::symlink_metadata(&legacy).is_err(){fs::rename(old,&legacy)?;return Ok(Some(format!("App data migrated; preserved conflicting files in {}",legacy.display())));}
    }
    Err(io::Error::other("No available legacy import directory"))
}
#[cfg(test)]mod tests{use super::*;
fn root(label:&str)->std::path::PathBuf{std::env::temp_dir().join(format!("titan-migrate-{}-{label}",std::process::id()))}
#[test]fn merges_into_existing_payload_directory_and_preserves_cache_inode(){use std::os::unix::fs::MetadataExt;let r=root("normal");let old=r.join("old");let new=r.join("new");fs::create_dir_all(old.join("caches/TID")).unwrap();fs::create_dir_all(new.join("payloads")).unwrap();fs::write(old.join("config.toml"),b"owner-settings").unwrap();fs::write(old.join("kraken-mount.elf"),b"old-elf").unwrap();let inode=fs::metadata(old.join("caches/TID")).unwrap().ino();migrate(&old,&new).unwrap();assert!(!old.exists());assert_eq!(fs::read(new.join("config.toml")).unwrap(),b"owner-settings");assert_eq!(fs::metadata(new.join("caches/TID")).unwrap().ino(),inode);assert_eq!(fs::read(new.join("titan-core.elf")).unwrap(),b"old-elf");assert!(migrate(&old,&new).unwrap().is_none());fs::remove_dir_all(r).unwrap();}
#[test]fn conflicting_owner_files_survive_without_overwrite(){let r=root("conflict");let old=r.join("old");let new=r.join("new");fs::create_dir_all(&old).unwrap();fs::create_dir_all(&new).unwrap();fs::write(old.join("config.toml"),b"old").unwrap();fs::write(new.join("config.toml"),b"new").unwrap();migrate(&old,&new).unwrap();assert_eq!(fs::read(new.join("config.toml")).unwrap(),b"new");assert_eq!(fs::read(new.join("legacy-import-0/config.toml")).unwrap(),b"old");assert!(!old.exists());fs::remove_dir_all(r).unwrap();}
}
