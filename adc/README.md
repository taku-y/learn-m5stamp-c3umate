## プログラムの実行

デバイス名（cu.usbmodem101）は適当に変更します。

```bash
CRATE_CC_NO_DEFAULTS=1 cargo espflash --release --monitor /dev/cu.usbmodem101
```
