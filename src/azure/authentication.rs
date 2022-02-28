use std::{env, fs};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use azure_core::HttpClient;
use http::Request;
use jwt::{AlgorithmType, Claims, JoseHeader, PKeyWithDigest, RegisteredClaims, SignWithKey, Token};
use jwt::claims::SecondsSinceEpoch;
use jwt::token::Signed;
use openssl::hash::{DigestBytes, MessageDigest};
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::x509::X509;
use serde::Serialize;
use uuid::Uuid;
use crate::azure::{ClientId, Directory};

const AZ_TOKEN_ENDPOINT_BASE: &str = "https://login.microsoftonline.com";
const AZ_TOKEN_ENDPOINT_TAIL: &str = "oauth2/v2.0/token";
const AZ_CERT_FILE: &str = "azure.cer";
const AZ_CERT_PRIV_FILE: &str = "azure.key";

const JWT_EXP_DURATION: Duration = Duration::from_secs(60);

macro_rules! api {
    ($api:expr) => {
        concat!("https://", $api, ".microsoft.com")
    }
}

macro_rules! scope {
    ($api:expr) => {
        concat!(api!($api), "/.default")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TokenScope {
    Management,
}

impl TokenScope {
    pub fn api(self) -> &'static str {
        match self {
            TokenScope::Management => scope!("management"),
        }
    }
}

pub fn generate_auth_request(az_client: &dyn HttpClient, tenant_id: &Directory, client_id: &ClientId, scope: TokenScope) -> Request<()> {
    let url = token_endpoint(tenant_id);
    let req = Request::builder().method("POST").uri(token_endpoint(tenant_id)).header("Content-Type", "application/x-www-form-urlencoded");

    todo!()
}

struct RequestBody {
    client_id: Option<ClientId>,
}

fn token_endpoint(tenant_id: &Directory) -> String {
    format!("{}/{}/{}", AZ_TOKEN_ENDPOINT_BASE, tenant_id.id(), AZ_TOKEN_ENDPOINT_TAIL)
}

fn jwt_token(cert: &X509, key: &Rsa<Private>, tenant_id: &Directory, client_id: &ClientId) -> Token<Header, Claims, Signed> {
    let fingerprint = cert_fingerprint_sha1(cert);

    let header = Header::with_fingerprint(&fingerprint);
    let claims = jwt_claims(tenant_id, client_id);

    let token = Token::new(header, claims);

    let pkey = PKey::from_rsa(key.clone()).unwrap();
    let pkey_with_digest = PKeyWithDigest {
        key: pkey,
        digest: MessageDigest::sha256(),
    };

    token.sign_with_key(&pkey_with_digest).unwrap()
}

#[derive(Serialize)]
struct Header {
    alg: AlgorithmType,
    typ: Type,
    x5t: String,
}

impl Header {
    fn with_fingerprint(digest: &[u8]) -> Header {
        Header {
            alg: AlgorithmType::Rs256,
            typ: Type::JWT,
            x5t: base64::encode(digest),
        }
    }
}

impl JoseHeader for Header {
    fn algorithm_type(&self) -> AlgorithmType {
        self.alg
    }
}

#[derive(Serialize)]
enum Type {
    JWT,
}

fn jwt_claims(tenant_id: &Directory, client_id: &ClientId) -> Claims {
    let (now, exp) = jwt_now_exp();

    let reg = RegisteredClaims {
        audience: Some(token_endpoint(tenant_id)),
        issuer: Some(client_id.id().to_string()),
        subject: Some(client_id.id().to_string()),
        not_before: Some(now),
        expiration: Some(exp),
        json_web_token_id: Some(jti()),
        issued_at: Some(now),
    };

    Claims::new(reg)
}

fn jwt_now_exp() -> (SecondsSinceEpoch, SecondsSinceEpoch) {
    let now = SystemTime::now();
    let now_anchored = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let exp = now + JWT_EXP_DURATION;
    let exp_anchored = exp.duration_since(SystemTime::UNIX_EPOCH).unwrap();

    (now_anchored.as_secs(), exp_anchored.as_secs())
}

fn jti() -> String {
    Uuid::new_v4().to_string()
}

fn cert_path() -> PathBuf {
    let mut exec_dir = env::current_exe().unwrap();
    exec_dir.set_file_name(AZ_CERT_FILE);
    exec_dir
}

fn load_cert_buf() -> Vec<u8> {
    fs::read(cert_path()).unwrap()
}

fn load_cert() -> X509 {
    X509::from_pem(&load_cert_buf()).unwrap()
}

fn cert_fingerprint_sha1(cert: &X509) -> DigestBytes {
    cert.digest(MessageDigest::sha1()).unwrap()
}

fn priv_key_path() -> PathBuf {
    let mut exec_dir = env::current_exe().unwrap();
    exec_dir.set_file_name(AZ_CERT_PRIV_FILE);
    exec_dir
}

fn load_priv_key() -> Vec<u8> {
    fs::read(priv_key_path()).unwrap()
}
