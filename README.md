# J.A.R.V.I.S. 2.0 (Mark I)

Otonom ve proaktif kişisel yapay zeka asistanı — Rust çekirdeği.

## Mimari

Monorepo workspace yapısı (18 crate):

| Crate | Açıklama |
|-------|----------|
| `jarvis-common` | Paylaşılan tipler, hatalar, event bus, lifecycle trait'leri |
| `jarvis-config` | YAML config yükleyici (deep merge, env var çözümleme) |
| `jarvis-core` | Async runtime, kernel, HTTP/WS sunucu, komut yönlendirici |
| `jarvis-mind` | LLM motoru (Llama.cpp/GGUF), sistem prompt'u |
| `jarvis-memory` | RAG motoru, Google Drive 5TB hafıza, vektör cache |
| `jarvis-voice` | STT (Whisper), TTS (XTTS v2 — Iron Man 2 JARVIS sesi), wake word |
| `jarvis-shield` | Öz savunma antivirüs, YARA, karantina, dosya watchdog |
| `jarvis-cocoon` | Metamorfoz modu, self-update, Mark versiyon yönetimi |
| `jarvis-biometrics` | Yüz doğrulama (Face ID) |
| `jarvis-protocols` | Pose Protokolü (uzak erişim), Misafir Protokolü |
| `jarvis-developer` | Kod analizi, God Mode optimizasyonu |
| `jarvis-media` | Medya ajansı (video, podcast, görsel üretimi) |
| `jarvis-body` | Ekran izleme, sistem araçları |
| `jarvis-daemon` | PID yönetimi, sinyal işleyiciler |
| `jarvis-tui` | Terminal arayüzü (gelecek: ratatui) |
| `jarvis-image` | Görsel işleme |
| `jarvis-utils` | Yardımcı fonksiyonlar (hash, format, zaman) |
| `jarvis-install` | Kurulum ve dizin yapısı oluşturucu |

## Donanım Gereksinimleri

- **CPU:** AMD Ryzen 5 7600X (6C/12T)
- **RAM:** 32GB DDR5
- **GPU:** NVIDIA RTX 4060 (8GB VRAM)

LLM optimizasyonu: `n_gpu_layers=35` (VRAM offload), `n_threads=12`, `n_batch=1024`, `use_mmap=true`

## Kurulum ve Çalıştırma

```bash
# Derle
cargo build --workspace

# Çalıştır
cargo run

# Testler
cargo test --workspace

# Lint
cargo clippy --workspace
```

## Konfigürasyon

Config dosyaları `config/` dizininde:

- `config.yaml` — ana yapılandırma + includes listesi
- `llm.yaml` — LLM/Llama.cpp parametreleri
- `memory.yaml` — RAG ve Google Drive ayarları
- `security.yaml` — antivirüs, WebSocket auth, sandbox
- `voice.yaml` — STT/TTS, wake word, ses profili
- `biometrics.yaml` — yüz doğrulama
- `daemon.yaml` — PID, log dizinleri
- `proactive.yaml` — otonom araştırma

## Endpointler

- `GET http://127.0.0.1:8080/health` — sağlık kontrolü
- `WS ws://127.0.0.1:8080/ws` — WebSocket komut arayüzü

## Koza Modu (Metamorfoz)

Büyük güncellemelerde sistem "koza" moduna girer:
- Gereksiz modüller uyku moduna geçer
- **Shield (antivirüs) KESİNLİKLE KAPANMAZ**
- Güncelleme tamamlanınca Mark versiyonu yükseltilir (Mark I → Mark II)
- Başarısız güncellemelerde otomatik rollback

## Lisans

Proprietary — Tüm hakları saklıdır.
