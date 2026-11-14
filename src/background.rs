//! Hide only ShadowMount's known web launcher, keeping the resident service alive.
use serde_json::Value;
pub fn owned_launcher(v:&Value)->bool{v["titleId"]=="FAKE10101"&&v["deeplinkUri"]=="http://127.0.0.1:10101/"&&v["localizedParameters"]["en-US"]["titleName"]=="ShadowMount+"}
#[cfg(feature="unified-stack")]
pub fn hide_shadow_launcher(){use std::{fs,path::Path,sync::atomic::{AtomicBool,Ordering}};static ATTEMPTED:AtomicBool=AtomicBool::new(false);if ATTEMPTED.swap(true,Ordering::SeqCst){return;}
 let root=Path::new("/user/app/FAKE10101");let meta=root.join("sce_sys/param.json");let Ok(bytes)=fs::read(&meta)else{return};let Ok(value)=serde_json::from_slice::<Value>(&bytes)else{log!("ShadowMount launcher: invalid metadata; left unchanged");return};
 if !owned_launcher(&value)||root.join("app.pkg").exists(){log!("ShadowMount launcher: ownership not verified; left unchanged");return;}
 match crate::register::uninstall("FAKE10101"){Ok(())=>log!("ShadowMount web launcher removed; background service remains running"),Err(e)=>log!("ShadowMount launcher could not be hidden: {e}")}
}
#[cfg(test)]mod tests{use super::*;use serde_json::json;
#[test]fn only_the_exact_shadow_web_launcher_is_owned(){let mut v=json!({"titleId":"FAKE10101","deeplinkUri":"http://127.0.0.1:10101/","localizedParameters":{"en-US":{"titleName":"ShadowMount+"}}});assert!(owned_launcher(&v));v["titleId"]=json!("PPSA10101");assert!(!owned_launcher(&v));v["titleId"]=json!("FAKE10101");v["deeplinkUri"]=json!("http://other/");assert!(!owned_launcher(&v));assert!(!owned_launcher(&json!({})));}}
