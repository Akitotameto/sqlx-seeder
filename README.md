# sqlx-seeder

## 説明

- Rustの`sqlx`というクレートに`seeder`コマンドが存在していなかったので作成してみました。
- SQLファイルを読み込んでテストデータを作成が可能。

## インストール

- 下記のコマンドでクレートをインストールします。

```
& cargo install sqlx-seeder
```

## コマンド

- 下記のコマンドでseedsディレクトリ内のSQLファイルを読み込んでテストデータを作成することができます。

```
$ sqlx seeder run
```

## 関数

- `cargo.toml`に`sqlx-seeder = "0.1.0"`と追記すると`lib.rs`の関数を使用することができます。
※バージョンが更新されている場合がありますので[こちら](https://crates.io/crates/sqlx-seeder)で確認していただいてから追記をお願いいたします。

### Examples

```toml
## Cargo.toml
[package]
name = "パッケージ名"
version = "0.1.0"
edition = "2018"

[dependencies]
sqlx-seeder = "0.1.0" <= 追記
```

```Rust
// main.rs
use rust_seeder::lib_hello;

fn main() {
    lib_hello();
}
```


