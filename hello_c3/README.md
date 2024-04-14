# hello_c3

20240414 [M5Stamp C3U Mate](https://www.switch-science.com/products/7894)で動作確認。

## 環境構築

* [こちらの記事](https://docs.esp-rs.org/std-training/02_2_software.html)を参考にした。

* ツールチェインのインストール
  ```console
  rustup toolchain install nightly-2023-11-14 --component rust-src
  ```
  ```console
  cargo install cargo-espflash espflash ldproxy
  ```

* （Macの場合）
  ```console
  # インストール後の環境変数の設定は必要なかった
  brew install llvm libuv
  ```

## 実行

```console
# cu.usbmodem1101は適当なデバイス名に変更
CRATE_CC_NO_DEFAULTS=1 cargo espflash --release --monitor /dev/cu.usbmodem1101
```
