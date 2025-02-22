# wifi

このプロジェクトはM5Stamp C3U MateでWifiに接続する例です。

## 実行

環境変数`WIFI_SSID`と`WIFI_PASS`を設定し、以下のコマンドを実行します。

```bash
CRATE_CC_NO_DEFAULTS=1 cargo espflash --release --monitor /dev/cu.usbmodem1101
```

## 環境構築

このプロジェクトは[テンプレート](https://docs.esp-rs.org/book/writing-your-own-application/generate-project/index.html#esp-idf-template)を修正して作成されました。

事前にツールチェインの設定が必要です。

```bash
rustup override set nightly
```

`rust-src`の設定も必要なようです。

```bash
rustup toolchain install nightly --component rust-src
```

雛形作成時の実行コマンドと指定されたオプションは以下の通りです。

```console
% cargo generate esp-rs/esp-idf-template cargo
⚠️   Favorite `esp-rs/esp-idf-template` not found in config, using it as a git repository: https://github.com/esp-rs/esp-idf-template.git
🤷   Project Name: wifi
🔧   Destination: /Users/taku-y/github/taku-y/learn-m5stamp-c3umate/wifi ...
🔧   project-name: wifi ...
🔧   Generating template ...
✔ 🤷   Which MCU to target? · esp32c3
✔ 🤷   Configure advanced template options? · true
✔ 🤷   ESP-IDF version (master = UNSTABLE) · v5.3
✔ 🤷   Configure project to use Dev Containers (VS Code and GitHub Codespaces)? · false
✔ 🤷   Configure project to support Wokwi simulation with Wokwi VS Code extension? · false
✔ 🤷   Add CI files for GitHub Action? · false
🔧   Moving generated files into: `/Users/taku-y/github/taku-y/learn-m5stamp-c3umate/wifi`...
🔧   Initializing a fresh Git repository
✨   Done! New project created /Users/taku-y/github/taku-y/learn-m5stamp-c3umate/wifi
```

## 参考情報

* [The Rust on ESP Book](https://docs.esp-rs.org/book/introduction.html)
  * [Generating Projects from Templates](https://docs.esp-rs.org/book/writing-your-own-application/generate-project/index.html#generating-projects-from-templates)
    * `esp-idf-template` - `std` template.
* [wifi example](https://github.com/esp-rs/esp-idf-svc/blob/15febb17eebcb0ccff3a2c247bc69cbd86eeede1/examples/wifi.rs)
  * `src/wifi.rs`はほぼこのコードのコピペです。
