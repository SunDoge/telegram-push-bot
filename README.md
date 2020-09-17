# Telegram Push Bot

## Run

```bash
export TELOXIDE_TOKEN="YOUR BOT TOKEN"
# export TELOXIDE_PROXY="http://127.0.0.1:8000"
export TELOXIDE_HOST="http://xxx.com"

cargo run --release
```

## Push

```python
import requests

url = 'http://kr.vultr.betacat.tech:3000/chatid/{chat_id}/sign/{sign}/text'

requests.post(url, json={'text': '阿巴阿巴阿巴'})
```
