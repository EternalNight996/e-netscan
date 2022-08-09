/// base64加密
fn encode(s: &str) -> String {
    use e_utils::base64::encode;
    encode(s)
}
/// base64解密
fn decode(s: &str) -> String {
    use e_utils::base64::decode;
    match decode(s) {
        Ok(r) => match std::str::from_utf8(&r) {
            Ok(r) => r.to_string(),
            Err(_e) => "".to_owned(),
        },
        Err(_e) => "".to_owned(),
    }
}
fn main() {
    let en = encode("hello world~");
    eprintln!("encode -> {}", en);
    let de = decode(&en);
    eprintln!("decode -> {}", de);
}
