use std::io;
use std::path::Path;

use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::pkcs8::{EncodePrivateKey, DecodePublicKey}; // 追加: 鍵変換用
use chrono::{Utc, Duration}; // Durationを追加

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
// 注: jsonwebtokenで利用するため、EncodingKeyに変換して返します
pub fn load_signing_key(path: &Path) -> Result<EncodingKey> {
    if !path.exists() {
        return Err("指定されたファイルパスに署名鍵が存在しません".into());
    }
    
    let pem = std::fs::read_to_string(path)?;
    // p256で一度パースして検証しつつ、jsonwebtoken用のKeyを作成します
    let secret = p256::ecdsa::SecretKey::from_pkcs8_pem(&pem)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData
                ,format!("指定された署名鍵の読み込みに失敗しました．{:?}", e)))?;
    
    let key_bytes = secret.to_pkcs8_pem(Default::default())?;
    Ok(EncodingKey::from_ec_pem(key_bytes.as_bytes())?)
}

// 検証鍵（公開鍵）の読み込み（検証用に必要となるため追加）
pub fn load_verifying_key(path: &Path) -> Result<DecodingKey> {
    if !path.exists() {
        return Err("指定されたファイルパスに公開鍵が存在しません".into());
    }
    let pem = std::fs::read_to_string(path)?;
    Ok(DecodingKey::from_ec_pem(pem.as_bytes())?)
}

//////
//JWTの実装

// ログインリクエスト構造体
#[derive(Debug,  Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// トークン種別
#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum TokenType {
    Refresh,
    Access,
}

// アクセストークンで認可する操作
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Role {
    EditMemo,
    ViewMemo,
    SummarizeMemo,
    EditTag,
    EditAccount, //アカウントの削除を除く
    DeleteAccount,
    ResetPassword,
}

// JWTヘッダー
// 注: jsonwebtokenのencode関数ではHeader構造体を都度生成するのが一般的ですが、
// 設定を共有するために残す場合は以下のように参照用として使えます
static JWT_ALGORITHM: Algorithm = Algorithm::ES256;

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
    role: Option<Role>, // アクセストークンで認可する操作 (Optional -> Optionに修正)
}

// ------------------------------------------------------------------
// JWTの発行関数群
// ------------------------------------------------------------------

/// リフレッシュトークンの発行
/// 引数: &LoginRequest
/// 戻り値: Result<JWT, 任意のError>
pub fn issue_refresh_token(req: &LoginRequest, key: &EncodingKey) -> Result<String> {
    let now = Utc::now();
    let expiration = now + Duration::days(7); // 例: 7日間有効

    let claims = JWT_Claim {
        iss: "mimo-server".to_string(),
        aud: "mimo-client".to_string(),
        sub: req.username.clone(),
        iat: now.timestamp() as usize,
        nbf: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
        typ: TokenType::Refresh,
        role: None, // リフレッシュトークンには権限を付与しない
    };

    let header = Header::new(JWT_ALGORITHM);
    let token = encode(&header, &claims, key)?;
    Ok(token)
}

/// アクセストークンの発行
/// 引数: &UserID, 要求する権限, &リフレッシュトークン
/// 戻り値: Result<JWT, 任意のError>
pub fn issue_access_token(
    user_id: &str, 
    role: Role, 
    refresh_token: &str, 
    enc_key: &EncodingKey,
    dec_key: &DecodingKey // リフレッシュトークン検証用
) -> Result<String> {
    // リフレッシュトークンが有効か確認
    validate_refresh_token(user_id, refresh_token, dec_key)?;

    let now = Utc::now();
    let expiration = now + Duration::hours(1); // 例: 1時間有効

    let claims = JWT_Claim {
        iss: "mimo-server".to_string(),
        aud: "mimo-client".to_string(),
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
        typ: TokenType::Access,
        role: Some(role),
    };

    let header = Header::new(JWT_ALGORITHM);
    let token = encode(&header, &claims, enc_key)?;
    Ok(token)
}

// ------------------------------------------------------------------
// JWTの検証関数群
// ------------------------------------------------------------------

/// リフレッシュトークンの検証
/// 引数: &UserID, &JWT
/// 戻り値: Result<(), 任意のError>
pub fn validate_refresh_token(user_id: &str, token: &str, key: &DecodingKey) -> Result<()> {
    let validation = Validation::new(JWT_ALGORITHM);
    let token_data = decode::<JWT_Claim>(token, key, &validation)?;
    let claims = token_data.claims;

    if claims.typ != TokenType::Refresh {
        return Err("トークン種別がRefreshではありません".into());
    }
    if claims.sub != user_id {
        return Err("ユーザーIDが一致しません".into());
    }

    Ok(())
}

/// アクセストークンの検証
/// 引数: &UserID, 要求する権限, &JWT
/// 戻り値: Result<(), 任意のError>
pub fn validate_access_token(
    user_id: &str, 
    required_role: Role, 
    token: &str, 
    key: &DecodingKey
) -> Result<()> {
    let validation = Validation::new(JWT_ALGORITHM);
    let token_data = decode::<JWT_Claim>(token, key, &validation)?;
    let claims = token_data.claims;

    if claims.typ != TokenType::Access {
        return Err("トークン種別がAccessではありません".into());
    }
    if claims.sub != user_id {
        return Err("ユーザーIDが一致しません".into());
    }

    // 権限のチェック
    match claims.role {
        Some(r) if r == required_role => Ok(()),
        _ => Err("必要な権限を持っていません".into()),
    }
}