pub mod token_store;

use crate::azure::{ClientId, Directory};
use azure_core::auth::TokenResponse;
use azure_core::HttpClient;
use bytes::Bytes;
use chrono::Utc;
use http::Request;
use jwt::claims::SecondsSinceEpoch;
use jwt::token::Signed;
use jwt::{
    AlgorithmType, Claims, JoseHeader, PKeyWithDigest, RegisteredClaims, SignWithKey, Token,
};
use oauth2::AccessToken;
use openssl::hash::{DigestBytes, MessageDigest};
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::{env, fs};
use uuid::Uuid;

const AZ_TOKEN_ENDPOINT_BASE: &str = "https://login.microsoftonline.com";
const AZ_TOKEN_ENDPOINT_TAIL: &str = "oauth2/v2.0/token";
const AZ_CERT_FILE: &str = "azure.crt";
const AZ_CERT_PRIV_FILE: &str = "azure.key";

const JWT_EXP_DURATION: Duration = Duration::from_secs(60);

macro_rules! token_endpoint {
    ($tenant_id:expr) => {
        format!(
            "{}/{}/{}",
            AZ_TOKEN_ENDPOINT_BASE,
            $tenant_id.id(),
            AZ_TOKEN_ENDPOINT_TAIL
        )
    };
}

pub struct TokenManager {
    directory: Directory,
    client: ClientId,
    key: PKeyWithDigest<Private>,
    fingerprint: DigestBytes,
}

impl TokenManager {
    pub fn new(
        directory: Directory,
        client: ClientId,
        cert: X509,
        key: PKey<Private>,
    ) -> TokenManager {
        TokenManager {
            fingerprint: cert_fingerprint_sha1(&cert),
            key: PKeyWithDigest {
                key,
                digest: MessageDigest::sha256(),
            },
            directory,
            client,
        }
    }

    pub async fn request_new(&self, client: &dyn HttpClient, scope: TokenScope) -> TokenResponse {
        let req = self.generate_auth_request(scope).into();
        let resp = client.execute_request2(&req).await.unwrap();
        let body = resp.into_body_string().await;

        serde_json::from_str::<ResponseBody>(&body).unwrap().into()
    }

    fn generate_auth_request(&self, scope: TokenScope) -> Request<Bytes> {
        const CONTENT_TYPE: &'static str = "application/x-www-form-urlencoded";

        let url = token_endpoint!(self.directory);

        Request::builder()
            .method("POST")
            .uri(url)
            .header("Content-Type", CONTENT_TYPE)
            .body(self.body(scope))
            .unwrap()
    }

    fn body(&self, scope: TokenScope) -> Bytes {
        const CLIENT_ASSERTION_TYPE: &'static str =
            "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";
        const GRANT_TYPE: &'static str = "client_credentials";

        macro_rules! body_key_value {
            ($key:expr, $value:expr) => {
                format!("{}={}", $key, $value)
            };
        }

        let token = body_key_value!("client_assertion", self.jwt_token().as_str());
        let scope = body_key_value!("scope", urlencoding::encode(scope.scope()));
        let client = body_key_value!("client_id", self.client.id());
        let assertion_type = body_key_value!(
            "client_assertion_type",
            urlencoding::encode(CLIENT_ASSERTION_TYPE)
        );
        let grant_type = body_key_value!("grant_type", GRANT_TYPE);

        let raw = [scope, client, assertion_type, token, grant_type].join("&");
        urlencoding::encode(&raw).into_owned().into_bytes().into()
    }

    fn jwt_token(&self) -> Token<Header, Claims, Signed> {
        let header = Header::with_fingerprint(&self.fingerprint);
        let claims = self.jwt_claims();

        let token = Token::new(header, claims);

        token.sign_with_key(&self.key).unwrap()
    }

    fn jwt_claims(&self) -> Claims {
        let (now, exp) = jwt_now_exp();

        let reg = RegisteredClaims {
            audience: Some(token_endpoint!(self.directory)),
            issuer: Some(self.client.id().to_string()),
            subject: Some(self.client.id().to_string()),
            not_before: Some(now),
            expiration: Some(exp),
            json_web_token_id: Some(jti()),
            issued_at: Some(now),
        };

        Claims::new(reg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenScope {
    Management,
}

impl TokenScope {
    pub fn scope(self) -> &'static str {
        match self {
            TokenScope::Management => "https://management.azure.com/.default",
        }
    }
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

#[derive(Deserialize)]
struct ResponseBody {
    expires_in: i64,
    access_token: String,
}

impl From<ResponseBody> for TokenResponse {
    fn from(resp: ResponseBody) -> TokenResponse {
        let now = Utc::now();
        let expires_on = now + chrono::Duration::seconds(resp.expires_in);

        TokenResponse {
            token: AccessToken::new(resp.access_token),
            expires_on,
        }
    }
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

pub fn load_cert() -> X509 {
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

fn load_priv_key_bytes() -> Vec<u8> {
    fs::read(priv_key_path()).unwrap()
}

pub fn load_priv_key() -> PKey<Private> {
    PKey::private_key_from_pem(&load_priv_key_bytes()).unwrap()
}
