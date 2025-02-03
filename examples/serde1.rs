use anyhow::{anyhow, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, OsRng};
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt;
use std::str::FromStr;

const KEY: &[u8] = b"01234567890123456789012345678901";

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    #[serde(rename = "privateAge")]
    age: u32,
    date_of_birth: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skills: Vec<String>,
    state: WorkState,
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
    #[serde_as(as = "DisplayFromStr")]
    sensitive: Sensitive,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    url: Vec<http::Uri>,
}

#[derive(Debug)]
struct Sensitive(String);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "details")]
enum WorkState {
    Working(String),
    OnLeave(DateTime<Utc>),
    Terminated,
}

fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;
    Ok(decoded)
}

fn encrypt(data: &[u8]) -> Result<String> {
    let cipher = ChaCha20Poly1305::new(KEY.into());

    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .or(Err(anyhow!("encrypt error")))?;

    // 拼接 nonce 和密文
    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(nonce.as_slice());
    output.extend_from_slice(&ciphertext);
    let encoded = URL_SAFE_NO_PAD.encode(&output);
    Ok(encoded)
}

fn decrypt(encoded: &str) -> Result<Vec<u8>> {
    let decoded = URL_SAFE_NO_PAD.decode(encoded.as_bytes())?;
    let nonce = decoded[..12].into();
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let decrypted = cipher
        .decrypt(nonce, &decoded[12..])
        .or(Err(anyhow!("decrypt error")))?;
    Ok(decrypted)
}

impl fmt::Display for Sensitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let encrypted = encrypt(self.0.as_bytes()).unwrap();
        write!(f, "{}", encrypted)
    }
}

impl FromStr for Sensitive {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let decrypted = decrypt(s)?;
        let decrypted = String::from_utf8(decrypted)?;
        Ok(Self(decrypted))
    }
}

fn main() -> Result<()> {
    let state1 = WorkState::OnLeave(Utc::now());
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        date_of_birth: Utc::now(),
        skills: vec!["rust".to_string(), "python".to_string()],
        state: state1,
        data: vec![1, 2, 3, 4, 5],
        sensitive: Sensitive("secret".to_string()),
        url: vec!["http://example.com".parse()?, "http://example.org".parse()?],
    };

    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    let user1: User = serde_json::from_str(&json)?;
    println!("{:?}", user1);
    println!("{:?}", user1.url[0].host());

    Ok(())
}
