//! Select installed PS5 package sources for the same native cache/AMPR pipeline.
use serde_json::Value;
use std::{collections::HashSet,path::Path};
pub fn candidates(remote:&[Value],native:&HashSet<String>)->Vec<Value>{
    let mut seen=native.clone();
    remote.iter().filter(|g|{
        let tid=g["title_id"].as_str().unwrap_or("");let path=g["path"].as_str().unwrap_or("");
        g["installed_pkg"]==true && g["installed"]==true && g["source_available"]==true && g["platform"]=="ps5" && crate::scan::valid_tid(tid) &&
        crate::library::installed_package_path(Path::new(path)) && Path::new(path).parent().and_then(Path::file_name).is_some_and(|n|n==tid) && seen.insert(tid.into())
    }).cloned().collect()
}
#[cfg(test)]mod tests{use super::*;use serde_json::json;
fn game()->Value{json!({"title_id":"PPSA22715","path":"/user/app/PPSA22715/app.pkg","installed_pkg":true,"installed":true,"source_available":true,"platform":"ps5"})}
#[test]fn existing_native_copy_has_priority_and_ps4_is_not_reinterpreted(){let g=game();assert_eq!(candidates(&[g.clone()],&HashSet::new()).len(),1);assert!(candidates(&[g.clone()],&HashSet::from(["PPSA22715".into()])).is_empty());let mut ps4=g;ps4["platform"]=json!("ps4");assert!(candidates(&[ps4],&HashSet::new()).is_empty());}
#[test]fn path_title_mismatch_and_missing_sources_are_rejected(){let mut g=game();g["path"]=json!("/user/app/PPSA12345/app.pkg");assert!(candidates(&[g],&HashSet::new()).is_empty());let mut g=game();g["source_available"]=json!(false);assert!(candidates(&[g],&HashSet::new()).is_empty());}
}
