use crate::error::{AppError, Result};
use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use std::env;

pub struct EmailService {
    smtp_transport: SmtpTransport,
    from_email: String,
    from_name: String,
}

impl EmailService {
    /// Configから初期化
    pub fn from_config(
        smtp_host: &str,
        smtp_port: u16,
        smtp_username: &str,
        smtp_password: &str,
        from_email: &str,
        from_name: &str,
    ) -> Result<Self> {
        let credentials = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
        
        let smtp_transport = SmtpTransport::starttls_relay(smtp_host)
            .map_err(|e| AppError::EnvironmentError(format!("SMTP接続エラー: {}", e)))?
            .port(smtp_port)
            .credentials(credentials)
            .build();
        
        Ok(Self {
            smtp_transport,
            from_email: from_email.to_string(),
            from_name: from_name.to_string(),
        })
    }

    /// 環境変数からSMTP設定を読み込んで初期化（後方互換性のため残す）
    pub fn from_env() -> Result<Self> {
        let smtp_host = env::var("SMTP_HOST")
            .map_err(|_| AppError::EnvironmentError("SMTP_HOST環境変数が設定されていません".to_string()))?;
        
        let smtp_port = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()
            .map_err(|_| AppError::EnvironmentError("SMTP_PORTが無効です".to_string()))?;
        
        let smtp_username = env::var("SMTP_USERNAME")
            .map_err(|_| AppError::EnvironmentError("SMTP_USERNAME環境変数が設定されていません".to_string()))?;
        
        let smtp_password = env::var("SMTP_PASSWORD")
            .map_err(|_| AppError::EnvironmentError("SMTP_PASSWORD環境変数が設定されていません".to_string()))?;
        
        let from_email = env::var("SMTP_FROM_EMAIL")
            .unwrap_or_else(|_| smtp_username.clone());
        
        let from_name = env::var("SMTP_FROM_NAME")
            .unwrap_or_else(|_| "Mimo".to_string());
        
        let credentials = Credentials::new(smtp_username, smtp_password);
        
        let smtp_transport = SmtpTransport::starttls_relay(&smtp_host)
            .map_err(|e| AppError::EnvironmentError(format!("SMTP接続エラー: {}", e)))?
            .port(smtp_port)
            .credentials(credentials)
            .build();
        
        Ok(Self {
            smtp_transport,
            from_email,
            from_name,
        })
    }

    /// 認証コードメールを送信
    pub async fn send_verification_code(&self, to_email: &str, code: &str) -> Result<()> {
        let subject = "【Mimo】メールアドレス認証コード";
        let body = format!(
            r#"
Mimoにご登録いただきありがとうございます。

以下の認証コードを入力して、メールアドレスの確認を完了してください。

認証コード: {}

このコードは15分間有効です。
もしこのメールに心当たりがない場合は、無視してください。

---
Mimo Server
"#,
            code
        );

        self.send_email(to_email, subject, &body).await
    }

    /// パスワードリセット確認コードメールを送信
    pub async fn send_password_reset_code(&self, to_email: &str, code: &str) -> Result<()> {
        let subject = "【Mimo】パスワードリセット確認コード";
        let body = format!(
            r#"
パスワードリセットのリクエストを受け付けました。

以下の確認コードを入力して、パスワードリセットを進めてください。

確認コード: {}

このコードは15分間有効です。
もしこのメールに心当たりがない場合は、無視してください。

---
Mimo Server
"#,
            code
        );

        self.send_email(to_email, subject, &body).await
    }

    /// メール送信（内部メソッド）
    async fn send_email(&self, to_email: &str, subject: &str, body: &str) -> Result<()> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| AppError::ValidationError(format!("送信元アドレスが無効: {}", e)))?)
            .to(to_email.parse()
                .map_err(|e| AppError::ValidationError(format!("送信先アドレスが無効: {}", e)))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| AppError::ValidationError(format!("メール作成エラー: {}", e)))?;

        // 非同期でメール送信（別スレッドで実行）
        let transport = self.smtp_transport.clone();
        tokio::task::spawn_blocking(move || {
            transport.send(&email)
        })
        .await
        .map_err(|e| AppError::ValidationError(format!("メール送信タスクエラー: {}", e)))?
        .map_err(|e| AppError::ValidationError(format!("メール送信エラー: {}", e)))?;

        Ok(())
    }
}
