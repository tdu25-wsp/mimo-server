use std::io;

use serde::{Serialize, Deserialize};
use jsonwebtoken::{Encode, Decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use p256::ecdsa::{SigningKey, VerifyingKey};
use chrono::{Utc, DateTime, DateTime};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

//////
//公開鍵認証関係の実装

// キーペアの生成
pub fn gen_keypair() -> Result<(SigningKey, VerifyingKey)> {
    let private = SigningKey::random(&mut rand::OsRng);
    let public = VerifyingKey::from(&private);
    Ok((private, public))
}

// 署名鍵の読み込み
pub fn load_signing_key(path: std::path::Path) -> Result<SigningKey> {
    if !path.exists() {
        return Err("指定されたファイルパスに署名鍵が存在しません".into());
    }
    
    let pem = std::fs::read_to_string(path)?;
    let secret = p256::ecdsa::SecretKey::from_pkcs8_pem(&pem)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData
                ,format!("指定された署名鍵の読み込みに失敗しました．{:?}", e)))?;
    Ok(SigningKey::from(secret))
}

//////
//JWTの実装

// トークン種別
#[derive(Debug, Serialize, Deserialize)]
enum TokenType {
    Refresh,
    Access,
}

// アクセストークンで認可する操作
#[derive(Debug, Serialize, Deserialize)]
enum Role {
    EditMemo,
    ViewMemo,
    SummarizeMemo,
    EditTag,
    EditAccount, //アカウントの削除を除く
    DeleteAccount,
    ResetPassword,
}

// JWTヘッダー
static  JWT_HEADER = Header {
    alg: Algorithm::ES256,
    typ: Some("JWT".to_string())
}

// JWTペイロード(クレーム)
#[derive(Debug, Serialize, Deserialize)]
pub struct JWT_Claim {
    iss: String, // JWT issuer
    aud: String, // JWTを行使する対象(APIサーバのURL)
    sub: String, // User ID
    jti: String, // JWT ID
    nbf: usize, // not before ここで指定した日時以前のリクエストは拒否
    exp: usize, // 有効期限

    typ: TokenType, // トークンの種別
    role: Optional<Role>, // アクセストークンで認可する操作
}

