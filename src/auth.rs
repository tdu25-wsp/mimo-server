use serde::{Serialize, Deserialize};
use jsonwebtoken::{Encode, Decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, DateTime, DateTime};

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
let mut header = Header::new(Algorithm::ES256);

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

