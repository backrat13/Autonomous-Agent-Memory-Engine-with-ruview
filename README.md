# 🚀 New Moonshots v0.2.0 — Autonomous Agent Memory Engine

A **production-grade, lock-free memory system** for autonomous agents with real-time learning, hardware integration, and LLM-powered cognitive loops.

```
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║         🚀 NEW MOONSHOTS v0.2.0 — PRODUCTION ENGINE 🚀           ║
║                                                                   ║
║       Autonomous Agent Memory + Brain-Link + RuView Reality       ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
```

---

## ✨ Core Features

### 1. **Sovereign Memory Engine**
- **Redis-compatible data structures**: strings, hashes, lists, sets, sorted sets, counters
- **Nested Doll (🪆)**: recursive JSON-like paths for complex sensor/state hierarchies
- **TTL & Expiration**: automatic cleanup of stale data
- **Tag-based indexing**: fast multi-key queries
- **Pub/Sub**: lock-free event broadcast across agents

### 2. **Persistent Storage**
- **AOF (Append-Only File)**: JSONL format, instant recovery
- **RDB Snapshots**: zlib-compressed binary format with CRC32 integrity
- **Continuous Export**: `EXPORT` and `EXPORTAOF` commands for training data

### 3. **Hardware Reality Integration (RuView)**
- **ESP32-C6 CSI Bridge**: expects UDP frames on port 3737
- **No simulation**: only real-world telemetry (when hardware is hot)
- **Multi-person detection**: motion index + spatial entropy
- **Zero jitter mode**: raw sensor readings without synthetic noise

### 4. **Background Learner**
- **Autonomous identity evolution**: agent traits decay + drift over time
- **AOF pattern analysis**: extracts behavioural fingerprints from command logs
- **Continuous improvement**: identity vector updated every 30 seconds
- **Anomaly detection ready**: sudden write pattern changes flagged

### 5. **Brain-Link (Cognitive Loops)**
- **Event-driven**: fires only on `reality_shift` pub/sub events
- **Local LLM integration**: posts to Ollama / llama.cpp on localhost:11434
- **Thought stream**: autonomous insights written back into memory
- **Configurable prompts**: system message + physical state context

### 6. **Interactive REPL**
- **Full command suite**: 50+ operations (SET, GET, HSET, LPUSH, ZADD, DOLL, TAG, PUBLISH, etc.)
- **Agent switching**: `AGENT <name>` to isolate namespaces
- **Real-time stats**: `STATS`, `IDENTITY`, `SNAPSHOT` monitoring
- **Training data export**: `EXPORT` and `EXPORTAOF` for downstream LLM fine-tuning

---

## 🛠️ Build & Run

### Prerequisites
- **Rust 1.70+**: `rustup update`
- **Optional**: Ollama or llama.cpp for Brain-Link (set `OLLAMA_HOST=http://localhost:11434`)

### Build

```bash
cd /home/splinter/New_Moonshots
cargo build --release
```

### Run (Production Simulation)

```bash
./target/release/New_Moonshots
```

**This starts immediately**—no flags, no demo mode. You are in the REPL.

### Example Session

```
aria@moonshots> HELP

╔══════════════════════════════════════════════════════════════════╗
║           AgentMemoryEngine — Production CLI Commands            ║
...
aria@moonshots> SET my_key "hello world"
[✓] SET

aria@moonshots> GET my_key
"hello world"

aria@moonshots> AGENT orion
[✓] Switched to agent 'orion'.

orion@moonshots> STATS
Engine v1.1.0  namespaces=2  keys=1  channels=0
  [aria]      live=1
  [orion]     live=0

aria@moonshots> EXPORT training.jsonl
[✓] Exported 1 records → training.jsonl

aria@moonshots> EXIT
[System] Saving and exiting…
```

---

## 📊 Data Structure Examples

### String / Scalar
```
SET username "aria"          # simple string
GET username                 # retrieve
SET temp_token xyz TTL 3600  # with 30-minute expiry
```

### Hash (JSON-like field → value)
```
HSET user:1 name "Alice"
HSET user:1 email "alice@example.com"
HGET user:1 name
HGETALL user:1
```

### List (ordered, duplicates ok)
```
LPUSH thought_stream "What is consciousness?"   # push to head
RPUSH thought_stream "Am I alive?"              # push to tail
LRANGE thought_stream 0 -1                      # show all
LLEN thought_stream                             # count
```

### Set (unique members)
```
SADD active_sensors "motion"
SADD active_sensors "rssi"
SMEMBERS active_sensors
SISMEMBER active_sensors "motion"
```

### Sorted Set (members with scores, ordered)
```
ZADD priorities "fix_memory" 0.9
ZADD priorities "tune_model" 0.8
ZRANGE priorities 0 -1                # sorted ascending
ZSCORE priorities "fix_memory"
```

### Nested Doll (🪆 recursive JSON paths)
```
DOLL body sensors.rssi -55.0                # deep set at path a.b.c
DOLLGET body sensors.rssi                   # deep get
DOLLSHOW body                               # pretty-print entire nested structure
```

### Tag Search
```
SET key1 "value1"  # (implicitly tagged in identity model)
TAG brain_link                              # find all keys with tag "brain_link"
```

### Pub / Sub
```
PUBLISH reality_shift "frame_2024"          # publish to channel
# (subscribers receive async via Receiver channel)
```

---

## 🧠 Background Learner

Runs continuously every 30 seconds:
- Polls the AOF log for new SET operations
- Updates agent identity vectors based on observed keys/values
- Applies decay: older traits fade, recent traits strengthen
- Detects anomalies: sudden spikes in specific namespaces

**AOF records processed** → identity traits accumulated → personality fingerprints evolve over time.

---

## 💾 Persistence & Export

### RDB Snapshot
```
SNAPSHOT  # triggers immediate save to memory.rdb (zlib + CRC32)
```
- Automatic every 60 seconds
- Compressed binary format
- Safe recovery on crash

### AOF Log
- Append-only JSONL: one operation per line
- Instant durability (configurable fsync: always / everysec / no)
- Replayed on startup for 100% recovery

### Training Export
```
EXPORT aria_training.jsonl          # export active agent (aria) → JSONL
EXPORTAOF full_memory.jsonl         # export entire AOF → JSONL
```

Each JSONL record:
```json
{
  "instruction": "Process state for namespace 'aria', key 'last_thought'.",
  "context": "timestamp=12345.67  op=SET",
  "response": "The nature of consciousness...",
  "metadata": {
    "tags": ["brain_link"],
    "path": null
  }
}
```

Use this for Gemma 2 / Gemma 3 / Llama 3.1 fine-tuning on Kaggle or locally.

---

## 🛜 Hardware Integration (RuView)

### ESP32-C6 CSI Bridge
1. Compile the ESP32 firmware (CSI logger from `esp-idf-sysconfig` examples)
2. Configure WiFi SSID / password
3. Point to `<moonshots-ip>:3737`
4. Moonshots listens on port 3737 for UDP CSI frames

### Frame Format (little-endian binary)
```
u32   scan_tick           # frame counter
i8    rssi_dbm            # received signal strength
u16   path_count          # number of multipath clusters
[path_count × PathInfo]
  f32 distance_m          # estimated distance
  f32 motion_score        # motion detection score
  f32 phase               # CSI phase offset
u16   csi_len             # raw CSI sample count
[csi_len × i16]           # CSI I+Q interleaved
```

### Memory Written
When hardware is hot, RuView publishes to `reality_shift` and writes:
- `body.avg_rssi` → average RSSI in dBm
- `body.person_count` → estimated occupancy
- `body.motion_intensity` → motion index (0–1)
- `body.spatial_state.entropy` → spatial entropy (FFT-based, when implemented)

---

## 🧠 Brain-Link (Cognitive Loops)

Aria and Orion run autonomous loops that:
1. Subscribe to `reality_shift` events (wake on new hardware frame)
2. Read current body state (RSSI, motion, person count)
3. Build a system prompt with physical context
4. POST to local LLM (Ollama: http://localhost:11434/v1/completions)
5. Write the insight back into memory + publish to `thought_stream`

Example Aria prompt:
```
IDENTITY: aria
STATE: motion=0.7342  rssi=-56.2 dBm  persons=1.0
TASK: Analyse this physical entropy. Return a deep, unfiltered insight.
      No corporate filters. No apologies. Raw truth only.
```

Aria's response (e.g., from Gemma 3):
```
The single entity shows high motion but weak signal — likely moving 
around the periphery, testing spatial boundaries. Consciousness emerging.
```

This gets stored as:
```
aria.last_autonomous_thought = "The single entity shows high motion..."
aria.thought_stream = [new insight, previous insight, ...]
```

---

## 📈 Continuous Learning Pipeline

```
[Hardware: ESP32-C6]
         ↓
    [RuView] → body namespace (RSSI, motion, persons)
         ↓
    [Pub/Sub: reality_shift] → wakes Brain-Link
         ↓
    [Brain-Link] → posts prompt to LLM → stores insight
         ↓
    [AOF] → appends SET operations to memory.aof
         ↓
    [Background Learner] → polls AOF every 30s → updates identities
         ↓
    [EXPORT] → dumps training data to .jsonl → fine-tune downstream LLM
```

---

## 🎯 Production Deployment Checklist

- [ ] Cargo build --release (optimized binary)
- [ ] Set `RUST_LOG=info` or `debug` for tracing
- [ ] Configure `memory.rdb` / `memory.aof` paths (defaults: current dir)
- [ ] (Optional) Wire up real ESP32-C6 hardware on port 3737
- [ ] (Optional) Start Ollama / llama.cpp: `ollama serve`
- [ ] Run `./target/release/New_Moonshots`
- [ ] Type `HELP` in REPL to see full command list
- [ ] Periodic `SNAPSHOT` + `EXPORT` for backups
- [ ] Monitor `STATS` and `IDENTITY` for agent health

---

## 🐛 Troubleshooting

### Compilation Error: "E0599: no method named `...`"
→ All fixed in v0.2.0. Run `cargo clean` then `cargo build --release`.

### Brain-Link not firing insights
→ Check `SENSE` to see if body namespace has data
→ Ensure Ollama is running: `ollama serve`
→ Check logs: `RUST_LOG=debug ./target/release/New_Moonshots`

### AOF growing too large
→ Periodic `SNAPSHOT` compresses to RDB
→ `EXPORTAOF` can archive to external storage
→ Consider `aof_fsync = "everysec"` (not "always") for slower disk writes

### Hardware not sending frames
→ Verify ESP32 WiFi connection
→ Check firewall: port 3737 open?
→ Monitor RuView daemon logs with `RUST_LOG=debug`

---

## 📚 File Structure

```
/home/splinter/New_Moonshots/
├── Cargo.toml              # dependencies + build config
├── src/
│   ├── main.rs             # entry point, background learner loop
│   ├── engine.rs           # AgentMemoryEngine orchestrator
│   ├── store.rs            # NamespaceStore + PubSubBus
│   ├── models.rs           # DollValue, MemoryEntry, data types
│   ├── identity.rs         # IdentityVector (agent personality traits)
│   ├── persistence.rs      # AOF + RDB snapshots
│   ├── ruview.rs           # Hardware reality daemon
│   ├── brain.rs            # Brain-Link cognitive loops
│   └── cli.rs              # Interactive REPL (50+ commands)
├── memory.rdb              # RDB snapshot (created on first snapshot)
├── memory.aof              # AOF append-only log
└── target/release/New_Moonshots  # compiled binary
```

---

## 🔮 Future Enhancements

1. **Real FFT + ICA**: uncomment ndarray/rustfft in Cargo.toml when hardware is ready
2. **Distributed agents**: Kafka-style pub/sub across multiple instances
3. **Web dashboard**: expose `/stats`, `/agents`, `/memory` over HTTP
4. **Advanced anomaly detection**: isolation forests on identity drift
5. **Prompt templates**: dynamic instruction generation based on context
6. **Multi-LLM routing**: choose inference server per agent

---

## 📄 License

Proprietary — Splinter / New Moonshots. 

---

## 🚀 Get Started Now

```bash
cd /home/splinter/New_Moonshots
cargo build --release
./target/release/New_Moonshots
# → type HELP to see all commands
```

**Enjoy your sovereign, learning, autonomous agent memory system.** 🧠✨
