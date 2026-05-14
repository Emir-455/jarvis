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

## Hızlı Başlangıç (Local Kurulum)

### 1. Gerekli Araçlar

```bash
# Rust (eğer kurulu değilse)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# WebSocket test aracı (opsiyonel, test için)
cargo install websocat
```

### 2. Repo'yu Klonla ve Derle

```bash
git clone https://github.com/Emir-455/jarvis.git
cd jarvis

# Derle (ilk seferde ~2-3 dakika)
cargo build --workspace --release

# Kontrol
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

### 3. Temel Çalıştırma (Stub Modda)

LLM, ses ve Drive yapılandırmadan önce sistem stub modda çalışır — doğrudan intent yanıtları (saat, tarih, selamlaşma) ve kategori bazlı stub yanıtlar verir:

```bash
# Çalıştır
RUST_LOG=info cargo run

# Başka bir terminal:
# Sağlık kontrolü
curl http://127.0.0.1:8080/health

# WebSocket komutları
echo 'merhaba' | websocat ws://127.0.0.1:8080/ws
echo 'jarvis' | websocat ws://127.0.0.1:8080/ws
echo 'saat kaç' | websocat ws://127.0.0.1:8080/ws
echo 'sistem durumu' | websocat ws://127.0.0.1:8080/ws
```

### 4. LLM Yapılandırma (Tam Çıkarım)

Bir GGUF model dosyası indirip `config/llm.yaml` içinde ayarlayın:

```bash
# Önerilen model (RTX 4060 8GB VRAM için):
# Mistral 7B Q4_K_M veya Llama 2 7B Q4_K_M
# HuggingFace'den indirin: https://huggingface.co/TheBloke

# config/llm.yaml düzenle:
model_path: "/path/to/model.gguf"   # GGUF dosya yolu
```

**Not:** Tam LLM çıkarımı için `llama-cpp-rs` crate'ini CUDA destekli derlemeniz gerekiyor:
```bash
# CUDA Toolkit kurulumu (RTX 4060 için gerekli)
# https://developer.nvidia.com/cuda-downloads

# Cargo.toml'a ekleyin:
# llama_cpp_rs = { version = "0.3", features = ["cuda"] }
```

LLM parametreleri donanımınıza göre optimize edilmiş durumda:
- `n_gpu_layers: 35` — VRAM'e sığacak kadar katman offload
- `n_threads: 12` — Ryzen 5 7600X'in tüm thread'leri
- `n_batch: 1024` — Hızlı prefill
- `n_ctx: 4096` — Bağlam penceresi
- `use_mmap: true` — Verimli RAM kullanımı

### 5. Google Drive Hafıza (5TB RAG)

```bash
# 1. Google Cloud Console'dan OAuth2 credentials oluşturun:
#    https://console.cloud.google.com/apis/credentials
#    - "OAuth 2.0 Client ID" → Desktop app
#    - credentials.json dosyasını indirin

# 2. config/memory.yaml düzenle:
drive:
  credentials_path: "/path/to/credentials.json"
  capacity_tb: 5
```

### 6. Ses Entegrasyonu (Iron Man 2 JARVIS)

```bash
# config/voice.yaml düzenle:
tts:
  voice_profile: "/path/to/jarvis_ironman2.wav"   # JARVIS ses referansı

stt:
  engine: whisper
  model: small      # veya medium (daha doğru, daha yavaş)
  language: tr
```

**Ses servisleri için Python bağımlılıkları:**
```bash
pip install TTS whisper torch torchaudio
```

## Konfigürasyon Dosyaları

Config dosyaları `config/` dizininde:

| Dosya | Açıklama |
|-------|----------|
| `config.yaml` | Ana yapılandırma + includes listesi |
| `llm.yaml` | LLM/Llama.cpp parametreleri |
| `memory.yaml` | RAG ve Google Drive ayarları |
| `security.yaml` | Antivirüs, WebSocket auth, sandbox |
| `voice.yaml` | STT/TTS, wake word, ses profili |
| `biometrics.yaml` | Yüz doğrulama |
| `daemon.yaml` | PID, log dizinleri |
| `proactive.yaml` | Proaktif izleme aralıkları |
| `screen.yaml` | Ekran izleme |

## Endpointler

| Endpoint | Metod | Açıklama |
|----------|-------|----------|
| `http://127.0.0.1:8080/health` | GET | Sağlık kontrolü (JSON: booted, mark, status) |
| `ws://127.0.0.1:8080/ws` | WebSocket | Komut arayüzü (metin gönder → JSON yanıt) |

### WebSocket Yanıt Formatı

```json
{"type": "response", "text": "Merhaba, Efendim. Size nasıl yardımcı olabilirim?"}
```

## Doğrudan Intent Komutları (LLM Gerektirmez)

| Komut | Yanıt |
|-------|-------|
| `merhaba` / `selam` | "Merhaba, Efendim. Size nasıl yardımcı olabilirim?" |
| `jarvis` | "For you, Efendim, always." |
| `saat kaç` | "Efendim, saat HH:MM." |
| `günaydın` | "Günaydın, Efendim. Sistemler aktif, güvenlik kalkanı çalışıyor." |
| `nasılsın` | "Tüm sistemler nominal seviyede çalışıyor, Efendim." |
| `iyi geceler` | "İyi geceler, Efendim. Güvenlik kalkanı aktif kalacak." |

## Proaktif İzleme

Sistem arka planda disk/RAM/CPU izler ve eşikler aşıldığında Türkçe uyarı verir:
- Disk %85+ → "Efendim, disk kullanımı kritik seviyeye ulaştı"
- RAM %80+ → "Efendim, bellek kullanımı yüksek"
- CPU yüksek yük → performans uyarısı

## Koza Modu (Metamorfoz)

Büyük güncellemelerde sistem "koza" moduna girer:
- Gereksiz modüller uyku moduna geçer
- **Shield (antivirüs) KESİNLİKLE KAPANMAZ**
- Güncelleme tamamlanınca Mark versiyonu yükseltilir (Mark I → Mark II)
- Başarısız güncellemelerde otomatik rollback

## Lisans

Proprietary — Tüm hakları saklıdır.
