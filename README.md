# sqlx-seeder

## Description

- The `seeder` command didn't exist in Rust's` sqlx` crate, so I created it.
- It is possible to read SQL files and create test data.

## Install

- Install the crate with the following command.

```
& cargo install sqlx-seeder
```

## Command

- You can create test data by reading the SQL file in the seeds directory with the following command.

```
$ sqlx seeder run
```

## Method

- You can use the function of `lib.rs` by adding` sqlx-seeder = "0.1.0" `to` cargo.toml`.
* The version may have been updated, so please check [here](https://crates.io/crates/sqlx-seeder) before adding.

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


