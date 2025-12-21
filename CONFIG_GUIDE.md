# 設定ガイド

## 概要

Mimo Serverは以下の2つの方法で設定できます：

1. **Config.toml** - ローカル開発環境向け
2. **環境変数** - 本番環境向け（環境変数が優先されます）

## Config.toml（開発環境）

開発環境では`Config.toml`ファイルで設定を管理できます：

```toml
[database.postgres]
connection_url = "postgresql://mimo:mimo_password@localhost:5432"
db_name = "mimo_db"

[database.mongodb]
connection_uri = "mongodb://mimo:mimo_password@localhost:27017"
db_name = "mimo_db"

[server]
host = "127.0.0.1"
port = 5050

[logging]
level = "info"

[jwt]
secret = "your-development-secret-key-change-in-production"

[email]
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "your-email@gmail.com"
smtp_password = "your-app-password"
from_email = "noreply@example.com"
from_name = "Mimo Server"
```

## 環境変数（本番環境）

本番環境では環境変数を使用します。環境変数が設定されている場合、Config.tomlの値より優先されます。

### データベース設定

```bash
# PostgreSQL
export POSTGRES_CONNECTION_URL="postgresql://user:password@host:5432"
export POSTGRES_DB_NAME="mimo_db"

# MongoDB
export MONGODB_CONNECTION_URI="mongodb://user:password@host:27017"
export MONGODB_DB_NAME="mimo_db"
```

### サーバー設定

```bash
export SERVER_HOST="0.0.0.0"
export SERVER_PORT="5050"
export LOG_LEVEL="info"
```

### JWT設定

```bash
# 本番環境では必ず強力な秘密鍵を設定してください
export JWT_SECRET="your-very-strong-secret-key-here"
```

### メール設定（SMTP）

```bash
# SMTP設定
export SMTP_HOST="smtp.gmail.com"
export SMTP_PORT="587"
export SMTP_USERNAME="your-email@gmail.com"
export SMTP_PASSWORD="your-app-password"
export SMTP_FROM_EMAIL="noreply@example.com"
export SMTP_FROM_NAME="Mimo Server"
```

#### Gmail使用時の注意

Gmailを使用する場合は、アプリパスワードを生成する必要があります：

1. Googleアカウントの「セキュリティ」設定にアクセス
2. 「2段階認証プロセス」を有効化
3. 「アプリパスワード」を生成
4. 生成されたパスワードを`SMTP_PASSWORD`に設定

## 優先順位

設定の読み込み優先順位：

1. **環境変数** - 最優先（本番環境）
2. **Config.toml** - 開発環境のデフォルト

例：Config.tomlで`jwt.secret`を設定していても、環境変数`JWT_SECRET`が設定されている場合は環境変数が使用されます。

## セキュリティのベストプラクティス

### 開発環境

- Config.tomlは`.gitignore`に追加して、実際の認証情報をコミットしない
- サンプル設定は`Config.toml.example`として別途管理

### 本番環境

- 必ず環境変数を使用する
- JWT_SECRETは32文字以上のランダムな文字列を使用
- SMTP_PASSWORDは直接パスワードではなく、アプリパスワードを使用
- 環境変数は暗号化されたシークレット管理ツール（AWS Secrets Manager、Kubernetes Secretsなど）で管理

## トラブルシューティング

### Config.tomlが見つからないエラー

```
Failed to read Config.toml. Use environment variables or provide Config.toml
```

解決方法：
- プロジェクトルートに`Config.toml`を作成
- または、必要な環境変数をすべて設定

### SMTP接続エラー

```
SMTP接続エラー: ...
```

解決方法：
- `smtp_host`と`smtp_port`が正しいことを確認
- ファイアウォールでポート587（または465）が開いていることを確認
- Gmailの場合は2段階認証とアプリパスワードを使用

### JWT秘密鍵エラー

```
JWT_SECRET must be set when using env vars
```

解決方法：
- 環境変数のみを使用している場合は`JWT_SECRET`を必ず設定
- または`Config.toml`で`[jwt]`セクションを定義
