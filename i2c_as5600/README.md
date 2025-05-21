* `wifi`と同様にテンプレートから作成
* AS5600は次のように接続
    * VCC - M5Stampの5V
    * GND - 同GND
    * SDA - GPIO0
    * SCL - GPIO1
    * 残りのピンはフリー
* 以下のコマンドを実行。
    ```
    CRATE_CC_NO_DEFAULTS=1 cargo espflash --release --monitor /dev/cu.usbmodem1101
    ```
* ICの上でマグネットを回転
