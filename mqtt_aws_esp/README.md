# mqtt_aws_esp

このプロジェクトはM5Stamp C3U MateでAWS IoT CoreにMQTTでメッセージを送受信する例です。

## 実行

`src/certificates`に以下のファイルを配置します。

* AWS IoT Coreの認証情報
  * `AmazonRootCA1.pem`
  * `DeviceCertificate.pem`
  * `client.private.key`
* AWS IoT Coreのエンドポイント
  * endpoint.txt（`mqtts://???:8883`のようになります）
* Wifi情報（M5Stampの接続先）
  * `wifi_pass.txt`
  * `wifi_ssid.txt`

以下のコマンドを実行します。

```bash
CRATE_CC_NO_DEFAULTS=1 cargo espflash --release --monitor /dev/cu.usbmodem1101
```

## 環境構築

このプロジェクトは`wifi`と同様にテンプレートを修正して作成されました。
