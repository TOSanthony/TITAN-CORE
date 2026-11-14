fn load_notice(mode: &[u8], out: &mut [u8; 64]) -> usize {
    let mut n = 0;
    for part in [
        b"chmod v0.1" as &[u8],
        b"" as &[u8],
        b" (",
        mode,
        b") running",
    ] {
        out[n..n + part.len()].copy_from_slice(part);
        n += part.len();
    }
    n
}
#[test]fn all_existing_modes_have_the_requested_notice(){for mode in [b"pkg".as_slice(),b"zstd",b"lz4",b"plain"]{let mut out=[0;64];let n=load_notice(mode,&mut out);let expected=format!("chmod v0.1 ({}) running",std::str::from_utf8(mode).unwrap());assert_eq!(&out[..n],expected.as_bytes());assert_eq!(out[n],0);}}
