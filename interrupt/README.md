# このプログラムについて

このプログラムはボタン割り込みによってプログラムの状態を切り替える実験のために作成しました。

# 実行

## 回路

* GPIO0  - タクトスイッチ1
* GPIO1  - タクトスイッチ2
* GPIO10 - タクトスイッチ3
* GPIO8  - タクトスイッチ4

## コマンド

以下のコマンドを実行します。

```bash
CRATE_CC_NO_DEFAULTS=1 cargo espflash --release --monitor /dev/cu.usbmodem1101
```
