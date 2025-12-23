# 概要

Mimo Server API について仕様を説明します。

| ホスト | プロトコル | データ形式 |

|  | https | JSON |



## アクセストークン

本APIは HttpOnly Cookie を使用して認証管理を行います。 ログイン成功時にサーバーから access_token および refresh_token クッキーが発行されます。以降のリクエストでは、ブラウザ（またはクライアント）が自動的にこのクッキーを送信する必要があります。

※手動でリクエストを送る場合は、Cookieヘッダーにトークンを含めてください。
Cookie: access_token=eyJhbGciOiJIUzI1NiIsInR5cCI...; refresh_token=...

## ステータスコード

下記のコードを返却します。

| ステータスコード | 説明 |
| - | - |
| 200 | リクエスト成功 |
| 201 | 登録成功 |
| 204 | リクエストに成功したが返却するbodyが存在しない |
| 400 | 不正なリクエストパラメータを指定している |
| 401 | APIアクセストークンが不正、または権限不正 |
| 404 | 存在しないURLにアクセス |
| 429 | リクエスト制限を超えている |
| 500 | 不明なエラー |
| 502 | APIサービスエラー |


## 利用制限

認証試行（IP単位）: 1時間あたり30回まで

認証試行（ユーザー単位）: 1分あたり5回まで

メール送信: 15分あたり2回まで

制限を超える場合は、429 Too Many Requests が返却されます。

# 認証

## ログイン

```
POST /api/auth/login HTTP/1.1
```

### Request

| パラメータ | 内容 | 必須 | デフォルト値 | 最大値 |
|  email  |  メールアドレス  |  ○  |  -  |  -  |
| password | パスワード | ○ | - | 256 |


```
{
  "email": "user@example.com",
  "password": "password123"
}
```

### Response

認証に成功すると Set-Cookie ヘッダーにてトークンが付与されます。
```
HTTP/1.1 200 OK
Set-Cookie: access_token=...; HttpOnly; Path=/; Max-Age=3600
Set-Cookie: refresh_token=...; HttpOnly; Path=/; Max-Age=604800

{
  "message": "Login successful",
  "user": {
    "user_id": "user_001",
    "email": "user@example.com",
    "display_name": "Mimo User"
  }
}
```


# メモ

## メモ作成

```
POST /api/memos HTTP/1.1
```

### Request

| パラメータ | 内容 | 必須 | デフォルト値 | 最大値 |
| user_id | ユーザID  |  ○  |  -  |  32  |
| content | メモの内容 | ○ | - | 512 |


```
{
  "user_id": "user_001",
  "content": "メモの内容"
}
```

### Response

```
HTTP/1.1 200 OK
{
  "memo_id": "123a4567-b89c-d0e1-f234-5678ghik90jl",
  "content": "メモの内容",
  "user_id": "user_001",
  "auto_tag_id": [
    "tag_id_study"
  ],
  "manual_tag_id": null,
  "created_at": "2025-12-23T10:00:00Z",
  "updated_at": "2025-12-23T10:00:00Z"
}
```

## メモ一覧取得


```
GET /api/memos/list/:user_id HTTP/1.1
```

### Request

| パラメータ | 内容 | 必須 | デフォルト値 | 最大値 |
|  user_id  |  ユーザID  |  ○  |  -  |  32  |


```
{
}
```

### Response

```
HTTP/1.1 200 OK
{
  "memos": [
    {
      "memo_id": "123a4567-b89c-d0e1-f234-5678ghik90jl",
      "content": "メモの内容",
      "user_id": "user_001",
      "created_at": "2025-12-23T10:00:00Z",
      "updated_at": "2025-12-23T10:00:00Z"
    }
  ]
}
```

# タグ

## タグ作成


```
POST /api/tags/:user_id HTTP/1.1
```

### Request

| パラメータ | 内容 | 必須 | デフォルト値 | 最大値 |
|  user_id  |  ユーザID  |  ○  |  -  |  32  |
| name | タグ名 | ○ | - | - |
| color_code | カラーコード | ○ | - | - |


```
{
  "name": "タグ名",
  "color_code": "#000000"
}
```

### Response

```
HTTP/1.1 200 OK
{
  "tag_id": "tag_uuid_0001",
  "user_id": "user_001",
  "name": "タグ名",
  "color_code": "#000000",
  "created_at": "2025-12-23T10:00:00Z",
  "updated_at": "2025-12-23T10:00:00Z"
}
```

# 要約

## AI要約作成


```
POST /api/sum/summarize HTTP/1.1
```

### Request

| パラメータ | 内容 | 必須 | デフォルト値 | 最大値 |
|  memo_ids  |  要約対象のメモID配列  |  ○  |  -  |  -  |

```
{
  "memo_ids": [
    "memo_id_1",
    "memo_id_2",
    "memo_id_3"
  ]
}
```

### Response

```
HTTP/1.1 200 OK
{
  "summary_id": "summary_uuid_0001",
  "user_id": "user_001",
  "content": "要約内容",
  "memo_ids": [
    "memo_id_1",
    "memo_id_2",
    "memo_id_3"
  ],
  "is_auto_generated": true,
  "created_at": "2025-12-23T20:00:00Z",
  "updated_at": "2025-12-23T20:00:00Z"
}
```

