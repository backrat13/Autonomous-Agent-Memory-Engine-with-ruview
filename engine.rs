// ══════════════════════════════════════════════════════════════════
//  engine.rs — The AgentMemoryEngine orchestrator
// ══════════════════════════════════════════════════════════════════

use crate::identity::IdentityVector;
use crate::models::{now_secs, DollValue, MemoryEntry};
use crate::persistence::{AofRecord, AofWriter, RdbSnapshot};
use crate::ruview::RuView;
use crate::store::{NamespaceStore, PubSubBus};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub const VERSION: &str = "1.1.0";

// ── Config ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub rdb_path: String,
    pub aof_path: String,
    pub snapshot_interval_secs: u64,
    pub aof_fsync: String,
    pub identity_decay_rate: f64,
    pub identity_min_weight: f64,
    pub ttl_check_interval_ms: u64,
    pub ruview_scan_interval_ms: u64,
    pub ruview_enabled: bool,
    pub jitter_factor: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            rdb_path: "memory.rdb".to_string(),
            aof_path: "memory.aof".to_string(),
            snapshot_interval_secs: 300,
            aof_fsync: "everysec".to_string(),
            identity_decay_rate: 0.001,
            identity_min_weight: 0.01,
            ttl_check_interval_ms: 1000,
            ruview_scan_interval_ms: 100,
            ruview_enabled: true,
            jitter_factor: 0.05,
        }
    }
}

// ── Engine hooks ──────────────────────────────────────────────────

type SetHook = Box<dyn Fn(&str, &str, &DollValue) + Send + Sync>;

pub struct AgentMemoryEngine {
    pub config: EngineConfig,
    pub namespaces: RwLock<HashMap<String, Arc<NamespaceStore>>>,
    pub identities: RwLock<HashMap<String, IdentityVector>>,
    pub pubsub: PubSubBus,
    pub aof: Option<AofWriter>,
    pub ruview: Option<Arc<RuView>>,
    hooks: RwLock<Vec<SetHook>>,
}

impl AgentMemoryEngine {
    pub fn new(config: EngineConfig) -> Arc<Self> {
        let aof = if config.aof_path != "none" {
            AofWriter::open(&config.aof_path, &config.aof_fsync).ok()
        } else {
            None
        };

        let ruview = if config.ruview_enabled {
            let rv = Arc::new(RuView::new(config.ruview_scan_interval_ms));
            Arc::clone(&rv).spawn_daemon();
            Some(rv)
        } else {
            None
        };

        let engine = Arc::new(Self {
            config,
            namespaces: RwLock::new(HashMap::new()),
            identities: RwLock::new(HashMap::new()),
            pubsub: PubSubBus::new(),
            aof,
            ruview,
            hooks: RwLock::new(Vec::new()),
        });

        let e_ttl = Arc::clone(&engine);
        std::thread::spawn(move || e_ttl.maintenance_loop());

        engine
    }

    /// Register a hook called after every SET operation.
    pub fn on_set<F>(&self, f: F)
    where
        F: Fn(&str, &str, &DollValue) + Send + Sync + 'static,
    {
        self.hooks.write().push(Box::new(f));
    }

    pub(crate) fn fire_set_hooks(&self, agent: &str, key: &str, val: &DollValue) {
        for hook in self.hooks.read().iter() {
            hook(agent, key, val);
        }
    }

    /// Background thread for TTL eviction and identity decay.
    fn maintenance_loop(&self) {
        let mut rng = rand::rng();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(
                self.config.ttl_check_interval_ms,
            ));

            // 1. TTL Cleanup
            let ns_map = self.namespaces.read();
            for store in ns_map.values() {
                store.sweep_expired();
            }
            drop(ns_map);

            // 2. Identity Decay with Stochastic Variance
            use rand::RngExt;
            let mut id_map = self.identities.write();
            for id_vector in id_map.values_mut() {
                for weight in id_vector.traits.values_mut() {
                    *weight *= 1.0 - self.config.identity_decay_rate;
                    let drift: f64 = rng.random_range(-0.0001..0.0001);
                    *weight += drift;
                    if *weight < self.config.identity_min_weight {
                        *weight = 0.0;
                    }
                }
                id_vector.traits.retain(|_, &mut v| v > 0.0);
            }
        }
    }

    /// Get or create an agent proxy.
    pub fn agent(self: &Arc<Self>, name: &str) -> AgentProxy {
        self.get_agent(name)
    }

    pub fn get_agent(self: &Arc<Self>, name: &str) -> AgentProxy {
        {
            let mut ns_map = self.namespaces.write();
            ns_map
                .entry(name.to_string())
                .or_insert_with(|| Arc::new(NamespaceStore::new(name)));
        }
        {
            let mut id_map = self.identities.write();
            id_map
                .entry(name.to_string())
                .or_insert_with(|| IdentityVector::new(name));
        }
        AgentProxy {
            name: name.to_string(),
            engine: Arc::clone(self),
        }
    }

    /// List all agent namespace names.
    pub fn namespaces(&self) -> Vec<String> {
        self.namespaces.read().keys().cloned().collect()
    }

    /// Force an RDB snapshot; returns compressed byte count.
    pub fn snapshot(&self) -> std::io::Result<usize> {
        let ns = self.namespaces.read().clone();
        let ids = self.identities.read().clone();
        RdbSnapshot::save(&self.config.rdb_path, &ns, &ids)
    }

    /// Restore state from disk (RDB + AOF replay).
    pub fn restore(&self) -> std::io::Result<()> {
        if let Some(payload) = RdbSnapshot::load(&self.config.rdb_path)? {
            let (ns, ids) = RdbSnapshot::restore(payload);
            *self.namespaces.write() = ns;
            *self.identities.write() = ids;
        }
        Ok(())
    }

    pub fn stats(&self) -> EngineStats {
        let ns_map = self.namespaces.read();
        let ns_count = ns_map.len();
        let mut total_keys = 0usize;
        let mut ns_stats = Vec::new();
        for ns in ns_map.values() {
            let s = ns.stats();
            total_keys += s.live_keys;
            ns_stats.push(s);
        }
        EngineStats {
            version: VERSION.to_string(),
            namespace_count: ns_count,
            total_live_keys: total_keys,
            pubsub_channels: self.pubsub.channel_list().len(),
            last_snapshot: 0.0, // updated if we track it
            namespaces: ns_stats,
        }
    }

    pub fn identity_report(&self, name: &str) -> String {
        let id_map = self.identities.read();
        if let Some(id) = id_map.get(name) {
            id.report()
        } else {
            format!("No identity found for agent '{name}'")
        }
    }

    pub fn shutdown(&self) {
        println!("[Engine] Shutdown initiated. Performing final RDB snapshot...");
        if let Err(e) = self.snapshot() {
            eprintln!("  Snapshot failed: {e}");
        } else {
            println!("  Snapshot saved to {}", self.config.rdb_path);
        }
        if let Some(ref aof) = self.aof {
            aof.close();
        }
    }
}

// ── Agent Proxy ───────────────────────────────────────────────────

pub struct AgentProxy {
    pub name: String,
    pub engine: Arc<AgentMemoryEngine>,
}

impl AgentProxy {
    fn store(&self) -> Arc<NamespaceStore> {
        self.engine
            .namespaces
            .read()
            .get(&self.name)
            .expect("namespace should always exist after get_agent()")
            .clone()
    }

    // ── String / DollValue ────────────────────────────────────────

    /// Store a value with optional TTL, optional tags, and importance weighting.
    /// This signature matches what main.rs and the demo expect.
    pub fn set(
        &self,
        key: &str,
        value: impl Into<String>,
        ttl: Option<f64>,
        tags: Option<Vec<String>>,
        importance: f64,
    ) {
        let dv = DollValue::String(value.into());
        let mut entry = MemoryEntry::new(dv.clone()).with_importance(importance);
        if let Some(t) = ttl {
            entry = entry.with_ttl(t);
        }
        if let Some(ts) = tags {
            entry = entry.with_tags(ts.into_iter().collect());
        }
        self.store().set(key, entry);

        if let Some(ref aof) = self.engine.aof {
            aof.log(AofRecord {
                ns: self.name.clone(),
                op: "SET".into(),
                key: key.into(),
                ts: now_secs(),
                value: serde_json::to_value(&dv).ok(),
                ..Default::default()
            });
        }

        let mut id_map = self.engine.identities.write();
        if let Some(id) = id_map.get_mut(&self.name) {
            id.absorb(key, &dv, importance);
        }
        drop(id_map);
        self.engine.fire_set_hooks(&self.name, key, &dv);
    }

    pub fn get(&self, key: &str) -> Option<DollValue> {
        let entry = self.store().get(key)?;
        let mut id_map = self.engine.identities.write();
        if let Some(id) = id_map.get_mut(&self.name) {
            id.total_reads += 1;
        }
        Some(entry.value)
    }

    pub fn delete(&self, key: &str) -> bool {
        self.store().delete(key)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.store().exists(key)
    }

    pub fn keys(&self, pattern: &str) -> Vec<String> {
        self.store().keys(pattern)
    }

    pub fn ttl(&self, key: &str) -> f64 {
        self.store().ttl(key)
    }

    pub fn expire(&self, key: &str, ttl_secs: f64) -> bool {
        self.store().expire(key, ttl_secs)
    }

    // ── Hash ─────────────────────────────────────────────────────

    pub fn hset(&self, key: &str, field: &str, value: impl Into<String>) {
        self.store()
            .hset(key, field, DollValue::String(value.into()));
    }

    pub fn hset_float(&self, key: &str, field: &str, value: f64) {
        self.store().hset(key, field, DollValue::Float(value));
    }

    pub fn hget(&self, key: &str, field: &str) -> Option<DollValue> {
        self.store().hget(key, field)
    }

    pub fn hgetall(&self, key: &str) -> HashMap<String, DollValue> {
        self.store().hgetall(key)
    }

    // ── List ─────────────────────────────────────────────────────

    pub fn lpush(&self, key: &str, value: impl Into<String>) -> usize {
        self.store().lpush(key, DollValue::String(value.into()))
    }

    pub fn rpush(&self, key: &str, value: impl Into<String>) -> usize {
        self.store().rpush(key, DollValue::String(value.into()))
    }

    pub fn lpop(&self, key: &str) -> Option<DollValue> {
        self.store().lpop(key)
    }

    pub fn rpop(&self, key: &str) -> Option<DollValue> {
        self.store().rpop(key)
    }

    pub fn lrange(&self, key: &str, start: i64, stop: i64) -> Vec<DollValue> {
        self.store().lrange(key, start, stop)
    }

    pub fn llen(&self, key: &str) -> usize {
        self.store().llen(key)
    }

    // ── Set ───────────────────────────────────────────────────────

    pub fn sadd(&self, key: &str, member: impl Into<String>) -> bool {
        self.store().sadd(key, member.into())
    }

    pub fn smembers(&self, key: &str) -> HashSet<String> {
        self.store().smembers(key)
    }

    pub fn sismember(&self, key: &str, member: &str) -> bool {
        self.store().sismember(key, member)
    }

    // ── Sorted Set ────────────────────────────────────────────────

    pub fn zadd(&self, key: &str, member: impl Into<String>, score: f64) {
        self.store().zadd(key, &member.into(), score);
    }

    pub fn zrange(&self, key: &str, start: i64, stop: i64) -> Vec<(String, f64)> {
        self.store().zrange(key, start, stop)
    }

    pub fn zscore(&self, key: &str, member: &str) -> Option<f64> {
        self.store().zscore(key, member)
    }

    // ── Counter ───────────────────────────────────────────────────

    pub fn incr(&self, key: &str, by: f64) -> f64 {
        self.store().incr(key, by)
    }

    pub fn decr(&self, key: &str, by: f64) -> f64 {
        self.store().incr(key, -by)
    }

    // ── Nesting Doll ─────────────────────────────────────────────

    pub fn doll_set(&self, key: &str, path: &str, value: DollValue) -> bool {
        let ok = self.store().doll_set(key, path, value.clone());
        if ok {
            if let Some(ref aof) = self.engine.aof {
                aof.log(AofRecord {
                    ns: self.name.clone(),
                    op: "DOLL_SET".into(),
                    key: key.into(),
                    ts: now_secs(),
                    path: Some(path.to_string()),
                    value: serde_json::to_value(&value).ok(),
                    ..Default::default()
                });
            }
        }
        ok
    }

    pub fn doll_get(&self, key: &str, path: &str) -> Option<DollValue> {
        use rand::RngExt;
        let mut val = self.store().doll_get(key, path)?;
        if let DollValue::Float(f) = val {
            if path.contains("rssi") || path.contains("motion") {
                let mut rng = rand::rng();
                let jitter =
                    f * self.engine.config.jitter_factor * rng.random_range(-1.0..1.0);
                val = DollValue::Float(f + jitter);
            }
        }
        Some(val)
    }

    // ── Tags & Pub/Sub ────────────────────────────────────────────

    pub fn by_tag(&self, tags: &[&str], match_all: bool) -> Vec<String> {
        self.store().by_tag(tags, match_all)
    }

    pub fn publish(&self, channel: &str, message: &str) -> usize {
        self.engine.pubsub.publish(channel, message)
    }

    pub fn subscribe(
        &self,
        channel: &str,
    ) -> crossbeam_channel::Receiver<crate::store::PubSubMessage> {
        self.engine.pubsub.subscribe(channel)
    }

    pub fn identity(&self) -> String {
        self.engine.identity_report(&self.name)
    }
}

// ── Engine Stats ──────────────────────────────────────────────────

pub struct EngineStats {
    pub version: String,
    pub namespace_count: usize,
    pub total_live_keys: usize,
    pub pubsub_channels: usize,
    pub last_snapshot: f64,
    pub namespaces: Vec<crate::store::NamespaceStats>,
}
