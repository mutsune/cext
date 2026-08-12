# chrome-ext-manager (`cext`)

Chrome の「独自拡張機能」を git リポジトリとして一括管理する CLI ツールです。
各拡張機能を git remote URL 単位で clone・保存し、一覧の書き出しや読み込み、削除ができます。

- バイナリ名: `cext`
- 実装言語: Rust
- CLI フレームワーク: [clap](https://docs.rs/clap/) (derive API)
- git 操作: システムの `git` コマンドをサブプロセスとして呼び出します（`git` が PATH 上に必要です）

## 保存場所

デフォルトの保存先はこちらです。

```
$HOME/Library/Application Support/Google/private extensions/
```

保存された各拡張機能は、このディレクトリ配下に **1つの git リポジトリ（フォルダ）** として置かれます。
一覧表示の際は、各フォルダの `git remote get-url origin` を読み取って URL を復元するので、
別途メタデータファイルを持つ必要がありません。

`--dir <DIR>` オプションで保存先を上書きできます（テストや別ディレクトリ運用に便利です）。

## ビルド

```bash
cd chrome-ext-manager
cargo build --release
# 生成物: target/release/cext
```

PATH の通った場所に置く場合:

```bash
cp target/release/cext /usr/local/bin/cext
# もしくは
cargo install --path .
```

## 使い方

### 1. 拡張機能を保存する（git clone して配置）

```bash
cext add https://github.com/someone/my-extension.git
```

- `--name` を指定しない場合は、保存先ディレクトリの中で普通に `git clone <url>` を実行するのと同じです。
  フォルダ名は git 自身が決めます（自前で URL をパースしたりはしません）。
- `--name` で保存フォルダ名を明示することもできます。

```bash
cext add git@github.com:someone/my-extension.git --name my-ext
```

- `--name` 指定時は、既に同名フォルダが存在すれば clone をスキップします。
  `--name` 省略時は通常の `git clone` と同様、同名ディレクトリが既にあると git 自体がエラーで止まります。

### 2. 保存済み拡張機能を URL テキストリストとして出力する

```bash
cext list
```

標準出力に、保存されているすべての拡張機能の remote URL が 1 行 1 件で出力されます。

```
https://github.com/someone/my-extension.git
https://github.com/other/another-extension.git
```

ファイルに書き出す場合:

```bash
cext list --output extensions.txt
```

このファイルはそのまま `import` コマンドの入力として使えます。

### 3. リストファイルを読み込んで一括保存する

```bash
cext import extensions.txt
```

- ファイルは 1 行 1 URL の形式（`list --output` の出力と同じ形式）。
- 空行、および `#` から始まる行はコメントとして無視されます。
- 既に保存済みの拡張機能はスキップされ、未保存のものだけ clone されます。
- 複数マシン間で保存内容を同期する用途を想定しています（`list --output` → 別マシンで `import`）。

`extensions.txt` の例:

```
# 業務用拡張機能
https://github.com/someone/my-extension.git
https://github.com/other/another-extension.git
```

### 4. 拡張機能を削除する

```bash
cext remove my-extension
```

- `name` には保存フォルダ名（`add` 時の名前、または `list` で確認できる URL から推測できる名前）を指定します。
- 確認プロンプトが表示されます。スキップするには `-y` / `--yes` を付けます。

```bash
cext remove my-extension --yes
```

## コマンド一覧

```
cext <COMMAND>

Commands:
  add     Clone a Chrome extension from a git remote URL and save it
  list    List saved extensions as a plain URL text list
  import  Import a URL list file and save every extension listed in it
  remove  Remove a saved extension by name

Options:
      --dir <DIR>  Override the extensions storage directory
  -h, --help       Print help
  -V, --version    Print version
```

各サブコマンドの詳細は `cext <command> --help` でも確認できます。

## Chrome への読み込み方（参考）

`cext` は拡張機能ファイルを clone・保存するところまでを担当します。実際に Chrome に読み込ませるには:

1. `chrome://extensions` を開く
2. 「デベロッパー モード」を ON にする
3. 「パッケージ化されていない拡張機能を読み込む」から、`$HOME/Library/Application Support/Google/private extensions/<拡張機能名>` を選択する

## 動作要件

- Rust（`cargo build` でビルドする場合）
- `git` コマンドが PATH 上にあること（`add` / `import` / `list` で使用します）
- macOS を想定したデフォルトパスですが、`--dir` を指定すれば他 OS でも動作します

## プロジェクト構成

```
chrome-ext-manager/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs   # エントリポイント、コマンドのディスパッチ
    ├── cli.rs    # clap によるサブコマンド定義
    └── ops.rs    # add / list / import / remove の実装
```
