use std::path::Path;

use crate::error::{AppError, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//////
// パスワード関係の実装
pub fn validate_password_format(password: &str) -> Result<()> {
    let length_min = 8;
    let length_max = 256;
    if password.len() < length_min || password.len() > length_max {
        return Err(AppError::ValidationError(format!(
            "パスワードは{}文字以上{}文字以下である必要があります",
            length_min, length_max
        )));
    }

    Ok(())
}

//////
// 共通鍵（HMAC）認証関係の実装

/// ランダムな共通鍵を生成（32バイト = 256ビット）
/// Base64エンコードされた文字列を返す
pub fn generate_secret_key() -> String {
    use base64::{Engine as _, engine::general_purpose};
    use rand::RngCore;

    let mut rng = rand::rng();
    let mut key = vec![0u8; 32]; // 256ビット
    rng.fill_bytes(&mut key);
    general_purpose::STANDARD.encode(&key)
}

/// JWT秘密鍵の読み込み（または自動生成）
/// 優先順位: 1. 環境変数 JWT_SECRET, 2. ファイル, 3. 自動生成（開発用）
pub fn load_or_generate_secret_key(path: Option<&Path>) -> Result<String> {
    // 環境変数から読み込みを試みる
    if let Ok(secret) = std::env::var("JWT_SECRET") {
        if secret.is_empty() {
            return Err(AppError::EnvironmentError(
                "JWT_SECRET環境変数が空です".to_string(),
            ));
        }
        println!("✓ JWT secret loaded from environment variable");
        return Ok(secret);
    }

    // ファイルから読み込み
    if let Some(p) = path {
        if p.exists() {
            let secret = std::fs::read_to_string(p)
                .map_err(|e| AppError::EnvironmentError(e.to_string()))?;
            if !secret.trim().is_empty() {
                println!("✓ JWT secret loaded from file: {}", p.display());
                return Ok(secret.trim().to_string());
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        // 自動生成（開発・テスト用）
        println!("⚠️  JWT secret not found. Generating a new one for this session...");
        println!("⚠️  WARNING: This is for development/testing only!");
        println!("⚠️  For production, set JWT_SECRET environment variable.");
        let secret = generate_secret_key();
        println!("✓ Generated JWT secret: {}", &secret[..16]); // 最初の16文字のみ表示
        return Ok(secret);
    }
}

/// 後方互換性のため（環境変数・ファイルのみ、自動生成なし）
pub fn load_secret_key(path: Option<&Path>) -> Result<String> {
    // 環境変数から読み込みを試みる
    if let Ok(secret) = std::env::var("JWT_SECRET") {
        if secret.is_empty() {
            return Err(AppError::EnvironmentError(
                "JWT_SECRET環境変数が空です".to_string(),
            ));
        }
        return Ok(secret);
    }

    // ファイルから読み込み
    if let Some(p) = path {
        if !p.exists() {
            return Err(AppError::EnvironmentError(
                "指定されたファイルパスに署名鍵が存在しません".to_string(),
            ));
        }
        let secret =
            std::fs::read_to_string(p).map_err(|e| AppError::EnvironmentError(e.to_string()))?;
        if secret.trim().is_empty() {
            return Err(AppError::EnvironmentError(
                "鍵ファイルの内容が空です".to_string(),
            ));
        }
        return Ok(secret.trim().to_string());
    }

    Err(AppError::EnvironmentError(
        "JWT_SECRET環境変数またはファイルパスを指定してください".to_string(),
    ))
}

/// EncodingKey を作成（署名用）
pub fn create_encoding_key(secret: &str) -> EncodingKey {
    EncodingKey::from_secret(secret.as_bytes())
}

/// DecodingKey を作成（検証用）
pub fn create_decoding_key(secret: &str) -> DecodingKey {
    DecodingKey::from_secret(secret.as_bytes())
}

//////
//JWTの実装

// ログインリクエスト構造体
#[derive(Debug, Deserialize)]
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
static JWT_ALGORITHM: Algorithm = Algorithm::HS256;

// JWTペイロード(クレーム)
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaim {
    iss: String, // JWT issuer
    aud: String, // JWTを行使する対象(APIサーバのURL)
    sub: String, // User ID
    iat: usize,  // issued at 発行日時
    jti: String, // JWT ID
    nbf: usize,  // not before ここで指定した日時以前のリクエストは拒否
    exp: usize,  // 有効期限

    typ: TokenType,     // トークンの種別
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

    let claims = JwtClaim {
        jti: Uuid::new_v4().to_string(),
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
    let token =
        encode(&header, &claims, key).map_err(|e| AppError::EnvironmentError(e.to_string()))?;
    Ok(token)
}

/// アクセストークンの発行
/// 引数: &UserID, 要求する権限, &リフレッシュトークン, &秘密鍵
/// 戻り値: Result<JWT, 任意のError>
pub fn issue_access_token(
    user_id: &str,
    role: Role,
    refresh_token: &str,
    secret: &str,
) -> Result<String> {
    // リフレッシュトークンが有効か確認
    let dec_key = create_decoding_key(secret);
    validate_refresh_token(user_id, refresh_token, &dec_key)?;

    let now = Utc::now();
    let expiration = now + Duration::hours(1); // 例: 1時間有効

    let claims = JwtClaim {
        jti: Uuid::new_v4().to_string(),
        iss: "mimo-server".to_string(),
        aud: "mimo-client".to_string(),
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
        nbf: now.timestamp() as usize,
        typ: TokenType::Access,
        role: Some(role),
    };

    let header = Header::new(JWT_ALGORITHM);
    let enc_key = create_encoding_key(secret);
    let token = encode(&header, &claims, &enc_key)
        .map_err(|e| AppError::EnvironmentError(e.to_string()))?;
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
    let token_data = decode::<JwtClaim>(token, key, &validation)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let claims = token_data.claims;

    if claims.typ != TokenType::Refresh {
        return Err(AppError::ValidationError(
            "トークン種別がRefreshではありません".to_string(),
        ));
    }
    if claims.sub != user_id {
        return Err(AppError::ValidationError(
            "ユーザーIDが一致しません".to_string(),
        ));
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
    key: &DecodingKey,
) -> Result<()> {
    let validation = Validation::new(JWT_ALGORITHM);
    let token_data = decode::<JwtClaim>(token, key, &validation)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let claims = token_data.claims;

    if claims.typ != TokenType::Access {
        return Err(AppError::ValidationError(
            "トークン種別がAccessではありません".to_string(),
        ));
    }
    if claims.sub != user_id {
        return Err(AppError::ValidationError(
            "ユーザーIDが一致しません".to_string(),
        ));
    }

    // 権限のチェック
    match claims.role {
        Some(r) if r == required_role => Ok(()),
        _ => Err(AppError::ValidationError(
            "必要な権限を持っていません".to_string(),
        )),
    }
}
