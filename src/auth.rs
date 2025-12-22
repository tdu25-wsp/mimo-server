use crate::error::{AppError, Result};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//////
// ユーザ検証関係の実装
pub fn validate_email_format(email: &str) -> Result<()> {
    let email_regex = regex::Regex::new(r"(?i)^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$")
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    if !email_regex.is_match(email) {
        return Err(AppError::ValidationError(
            "無効なメールアドレス形式です".into(),
        ));
    }

    Ok(())
}

pub fn validate_display_name_format(name: &str) -> Result<()> {
    let length_min = 1;
    let length_max = 50;
    if name.len() < length_min || name.len() > length_max {
        return Err(AppError::ValidationError(format!(
            "表示名は{}文字以上{}文字以下である必要があります",
            length_min, length_max
        )));
    }

    Ok(())
}

pub fn validate_user_id_format(user_id: &str) -> Result<()> {
    let user_id_regex = regex::Regex::new(r"^[a-zA-Z0-9_-]{3,32}$")
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    if !user_id_regex.is_match(user_id) {
        return Err(AppError::ValidationError(
            "ユーザーIDは3〜32文字の英数字、アンダースコア、ハイフンのみ使用できます".into(),
        ));
    }

    Ok(())
}

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
// 鍵関係の実装
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

// トークン種別
#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum TokenType {
    Refresh,
    Access,
    Registration,  // ユーザー登録用の一時トークン
    PasswordReset, // パスワードリセット用の一時トークン
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

    typ: TokenType,          // トークンの種別
    role: Option<Vec<Role>>, // アクセストークンで認可する操作 (Optional -> Optionに修正)
}

// ------------------------------------------------------------------
// JWTの発行関数群
// ------------------------------------------------------------------

/// リフレッシュトークンの発行
/// 引数: &LoginRequest
/// 戻り値: Result<JWT, 任意のError>
pub fn issue_refresh_token(user_id: &str, secret: &str) -> Result<String> {
    let now = Utc::now();
    let expiration = now + Duration::days(7); // 例: 7日間有効

    let claims = JwtClaim {
        jti: Uuid::new_v4().to_string(),
        iss: "mimo-server".to_string(),
        aud: "mimo-client".to_string(),
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        nbf: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
        typ: TokenType::Refresh,
        role: None, // リフレッシュトークンには権限を付与しない
    };

    let header = Header::new(JWT_ALGORITHM);
    let key = create_encoding_key(secret);
    let token =
        encode(&header, &claims, &key).map_err(|e| AppError::EnvironmentError(e.to_string()))?;
    Ok(token)
}

/// アクセストークンの発行
/// 引数: &UserID, 要求する権限, &リフレッシュトークン, &秘密鍵
/// 戻り値: Result<JWT, 任意のError>
pub fn issue_access_token(user_id: &str, roles: Vec<Role>, secret: &str) -> Result<String> {
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
        role: Some(roles), // 要求された権限をセット
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

/// トークンからユーザーIDを抽出（型チェックなし）
pub fn extract_user_id_from_token(token: &str, key: &DecodingKey) -> Result<String> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_audience(&["mimo-client"]);
    let token_data = decode::<JwtClaim>(token, key, &validation)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    Ok(token_data.claims.sub)
}

/// トークンからJTIを抽出
pub fn extract_jti_from_token(token: &str, key: &DecodingKey) -> Result<String> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_audience(&["mimo-client"]);
    let token_data = decode::<JwtClaim>(token, key, &validation)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    Ok(token_data.claims.jti)
}

/// リフレッシュトークンの検証
/// 引数: &UserID, &JWT
/// 戻り値: Result<(), 任意のError>
pub fn validate_refresh_token(user_id: &str, token: &str, key: &DecodingKey) -> Result<()> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_audience(&["mimo-client"]);
    let token_data = decode::<JwtClaim>(token, key, &validation)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let claims = token_data.claims;

    if claims.typ != TokenType::Refresh {
        return Err(AppError::ValidationError(
            "Token type is not Refresh".to_string(),
        ));
    }
    if claims.sub != user_id {
        return Err(AppError::ValidationError(
            "User ID does not match".to_string(),
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
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_audience(&["mimo-client"]);
    let token_data = decode::<JwtClaim>(token, key, &validation)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let claims = token_data.claims;

    if claims.typ != TokenType::Access {
        return Err(AppError::ValidationError(
            "Token type is not Access".to_string(),
        ));
    }
    if claims.sub != user_id {
        return Err(AppError::ValidationError(
            "User ID does not match".to_string(),
        ));
    }

    // 権限のチェック
    match claims.role {
        Some(r) if r.contains(&required_role) => Ok(()),
        _ => Err(AppError::ValidationError(
            "Insufficient permissions".to_string(),
        )),
    }
}

/// 登録用トークンの発行
/// 引数: メールアドレス, 秘密鍵
/// 戻り値: Result<JWT, AppError>
pub fn issue_registration_token(email: &str, secret: &str) -> Result<String> {
    let now = Utc::now();
    let expiration = now + Duration::minutes(15); // 15分間有効

    let claims = JwtClaim {
        jti: Uuid::new_v4().to_string(),
        iss: "mimo-server".to_string(),
        aud: "mimo-client".to_string(),
        sub: email.to_string(), // subにメールアドレスを格納
        iat: now.timestamp() as usize,
        nbf: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
        typ: TokenType::Registration,
        role: None,
    };

    let header = Header::new(JWT_ALGORITHM);
    let key = create_encoding_key(secret);
    let token =
        encode(&header, &claims, &key).map_err(|e| AppError::EnvironmentError(e.to_string()))?;
    Ok(token)
}

/// 登録用トークンの検証
/// 引数: トークン, 期待されるメールアドレス, 秘密鍵
/// 戻り値: Result<String, AppError> (成功時はJTI)
pub fn validate_registration_token(token: &str, email: &str, key: &DecodingKey) -> Result<String> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_audience(&["mimo-client"]);
    let token_data = decode::<JwtClaim>(token, key, &validation)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let claims = token_data.claims;

    if claims.typ != TokenType::Registration {
        return Err(AppError::ValidationError(
            "Token type is not Registration".to_string(),
        ));
    }
    if claims.sub != email {
        return Err(AppError::ValidationError(
            "Email address does not match".to_string(),
        ));
    }

    Ok(claims.jti)
}

/// パスワードリセット用トークンの発行
/// 引数: メールアドレス, 秘密鍵
/// 戻り値: Result<JWT, AppError>
pub fn issue_password_reset_token(email: &str, secret: &str) -> Result<String> {
    let now = Utc::now();
    let expiration = now + Duration::minutes(30); // 30分間有効

    let claims = JwtClaim {
        jti: Uuid::new_v4().to_string(),
        iss: "mimo-server".to_string(),
        aud: "mimo-client".to_string(),
        sub: email.to_string(),
        iat: now.timestamp() as usize,
        nbf: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
        typ: TokenType::PasswordReset,
        role: None,
    };

    let header = Header::new(JWT_ALGORITHM);
    let key = create_encoding_key(secret);
    let token =
        encode(&header, &claims, &key).map_err(|e| AppError::EnvironmentError(e.to_string()))?;
    Ok(token)
}

/// パスワードリセット用トークンの検証
/// 引数: トークン, 期待されるメールアドレス, 秘密鍵
/// 戻り値: Result<String, AppError> (成功時はJTI)
pub fn validate_password_reset_token(
    token: &str,
    email: &str,
    key: &DecodingKey,
) -> Result<String> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_audience(&["mimo-client"]);
    let token_data = decode::<JwtClaim>(token, key, &validation)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let claims = token_data.claims;

    if claims.typ != TokenType::PasswordReset {
        return Err(AppError::ValidationError(
            "Token type is not PasswordReset".to_string(),
        ));
    }
    if claims.sub != email {
        return Err(AppError::ValidationError(
            "Email address does not match".to_string(),
        ));
    }

    Ok(claims.jti)
}

//////
// ヘルパー関数: CookieからJWTを検証してユーザーIDを取得

pub fn authenticate_from_cookie(jar: &CookieJar, decoding_key: &DecodingKey) -> Result<String> {
    let access_token = jar
        .get("access_token")
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    extract_user_id_from_token(access_token.value(), decoding_key)
}
