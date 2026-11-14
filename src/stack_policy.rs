//! Side-effect-free decisions for the single-entry launcher.

pub fn firmware_label(raw: u32) -> Option<String> {
    let major = (raw >> 24) & 0xff;
    let minor = (raw >> 16) & 0xff;
    if [major >> 4, major & 15, minor >> 4, minor & 15].iter().any(|n| *n > 9) { return None; }
    let major = (major >> 4) * 10 + (major & 15);
    let minor = (minor >> 4) * 10 + (minor & 15);
    if major < 1 || major > 13 || (major == 13 && minor > 60) { return None; }
    Some(format!("{major}.{minor:02}"))
}

pub fn api_port(config: &str) -> Result<u16, String> {
    let mut port = 10101;
    for line in config.lines() {
        let line = line.split(['#', ';']).next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim().trim_matches('"');
        match key.trim() {
            "api_enabled" if value == "0" => return Err("ShadowMount API is disabled in its existing config.ini".into()),
            "api_port" => { port = value.parse::<u16>().map_err(|_| "Invalid ShadowMount API port")?; }
            "api_bind_address" if !["0.0.0.0", "127.0.0.1", ""].contains(&value) => return Err("ShadowMount API must include loopback access for TITAN CORE".into()),
            _ => {}
        }
    }
    if port == 0 || port == 8097 || port == 9021 { return Err("ShadowMount API port conflicts with TITAN CORE or elfldr".into()); }
    Ok(port)
}

#[derive(Debug, PartialEq, Eq)]
pub enum BootDecision { Reuse, Start, ObservePrevious, Inconsistent }
pub fn boot_decision(kstuff: bool, shadow: bool, previous_attempt: bool) -> BootDecision {
    if shadow { return if kstuff { BootDecision::Reuse } else { BootDecision::Inconsistent }; }
    if previous_attempt { BootDecision::ObservePrevious } else { BootDecision::Start }
}

/// Read the same kinfo record name field used by pinned ShadowMountPlus main.c.
/// Malformed/empty snapshots are unknown, never proof that a process is absent.
pub fn dependency_presence(bytes: &[u8]) -> Result<(bool,bool),String> {
    if bytes.is_empty() { return Err("Empty process snapshot".into()); }
    let mut offset = 0; let mut kstuff = false; let mut shadow = false;
    while offset < bytes.len() {
        let rest = &bytes[offset..];
        if rest.len() < 4 { return Err("Truncated process snapshot".into()); }
        let size = i32::from_ne_bytes(rest[..4].try_into().unwrap());
        if size <= 447 || size as usize > rest.len() { return Err("Unknown process record layout".into()); }
        let name = &rest[447..size as usize];
        let end = name.iter().position(|b| *b == 0).ok_or("Unterminated process name")?;
        let name = &name[..end];
        kstuff |= name.starts_with(b"kstuff") || name.starts_with(b"ps5-kstuff");
        shadow |= name.starts_with(b"shadowmountplus");
        offset += size as usize;
    }
    Ok((kstuff,shadow))
}

pub fn attempt_name(boot: Option<(u64,u64)>) -> String {
    match boot.filter(|(seconds,micros)| *seconds > 0 && *micros < 1_000_000) {
        Some((seconds,micros)) => format!("/system_tmp/titan-core-stack-{seconds}-{micros}.attempted"),
        None => "/system_tmp/titan-core-stack.attempted".into(),
    }
}

pub fn native_endpoint(path: &str) -> bool {
    path == "/api/settings" || path == "/api/purge" || path == "/api/rebuild-all" || path.starts_with("/api/rebuild/") || path.starts_with("/api/delete/")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn observes_existing_or_starting_dependency_processes() {
        let mut record=vec![0u8;600];record[..4].copy_from_slice(&600i32.to_ne_bytes());record[447..457].copy_from_slice(b"kstuff.elf");
        assert_eq!(dependency_presence(&record),Ok((true,false)));
        let mut shadow=vec![0u8;600];shadow[..4].copy_from_slice(&600i32.to_ne_bytes());shadow[447..466].copy_from_slice(b"shadowmountplus.elf");
        record.extend(shadow);assert_eq!(dependency_presence(&record),Ok((true,true)));
    }
    #[test] fn invalid_process_data_never_means_absent() {
        assert!(dependency_presence(&[]).is_err());assert!(dependency_presence(&[0;10]).is_err());
        let mut record=vec![0u8;500];record[..4].copy_from_slice(&700i32.to_ne_bytes());assert!(dependency_presence(&record).is_err());
    }
    #[test] fn unrelated_processes_do_not_match() {
        let mut record=vec![0u8;600];record[..4].copy_from_slice(&600i32.to_ne_bytes());record[447..452].copy_from_slice(b"titan");assert_eq!(dependency_presence(&record),Ok((false,false)));
    }
    #[test] fn boot_markers_separate_boots_and_reject_invalid_time() {
        assert_ne!(attempt_name(Some((100,0))),attempt_name(Some((101,0))));
        assert_eq!(attempt_name(Some((100,0))),attempt_name(Some((100,0))));
        assert_eq!(attempt_name(Some((0,0))),attempt_name(None));
        assert_eq!(attempt_name(Some((100,1_000_000))),attempt_name(None));
    }
    #[test] fn legacy_mutations_cannot_reach_native_engine() {
        for path in ["/api/settings", "/api/purge", "/api/rebuild-all", "/api/rebuild/PPSA12345", "/api/delete/PPSA12345"] { assert!(native_endpoint(path)); }
        for path in ["/api/state", "/api/sizes/PPSA12345", "/api/rescan", "/api/quit", "/icon/PPSA12345"] { assert!(!native_endpoint(path)); }
    }
    #[test] fn bcd_firmware_boundaries() {
        assert_eq!(firmware_label(0x13600000).as_deref(), Some("13.60"));
        assert_eq!(firmware_label(0x01000000).as_deref(), Some("1.00"));
        assert_eq!(firmware_label(0x09600000).as_deref(), Some("9.60"));
        for raw in [0, 0x14000000, 0x13610000, 0x0a000000, 0x130f0000] { assert_eq!(firmware_label(raw), None); }
    }
    #[test] fn existing_services_are_never_reloaded() {
        assert_eq!(boot_decision(true, true, true), BootDecision::Reuse);
        assert_eq!(boot_decision(true, true, false), BootDecision::Reuse);
        assert_eq!(boot_decision(false, true, false), BootDecision::Inconsistent);
        assert_eq!(boot_decision(false, false, true), BootDecision::ObservePrevious);
        assert_eq!(boot_decision(true, false, true), BootDecision::ObservePrevious);
        assert_eq!(boot_decision(false, false, false), BootDecision::Start);
    }
    #[test] fn respects_existing_api_configuration() {
        assert_eq!(api_port("# api_enabled=0\n;api_port=0"), Ok(10101));
        assert_eq!(api_port("[config]\napi_port=12000\napi_bind_address=127.0.0.1"), Ok(12000));
        for text in ["api_enabled=0", "api_port=0", "api_port=8097", "api_port=9021", "api_port=65536", "api_bind_address=192.168.0.4"] { assert!(api_port(text).is_err()); }
    }
}
