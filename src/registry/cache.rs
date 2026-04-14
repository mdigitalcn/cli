use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{Registry, RegistryKind};

const DEFAULT_TTL_SECS: u64 = 3600;

#[derive(Debug, Serialize, Deserialize)]
struct CacheMeta {
    fetched_at: u64,
}

pub enum CacheState {
    Fresh(Registry),
    Stale(Registry),
    Miss,
}

pub struct RegistryCache {
    base_dir: PathBuf,
    ttl: Duration,
}

impl RegistryCache {
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        Self {
            base_dir: PathBuf::from(home).join(".mdigitalcn").join("cache"),
            ttl: Duration::from_secs(DEFAULT_TTL_SECS),
        }
    }

    fn kind_dir(&self, kind: RegistryKind) -> PathBuf {
        self.base_dir.join(format!("{}s", kind.label()))
    }

    pub fn get(&self, kind: RegistryKind) -> CacheState {
        let dir = self.kind_dir(kind);
        let meta_path = dir.join("meta.json");
        let registry_path = dir.join("registry.json");

        let meta_content = match std::fs::read_to_string(&meta_path) {
            Ok(c) => c,
            Err(_) => return CacheState::Miss,
        };
        let meta: CacheMeta = match serde_json::from_str(&meta_content) {
            Ok(m) => m,
            Err(_) => return CacheState::Miss,
        };

        let registry_content = match std::fs::read_to_string(&registry_path) {
            Ok(c) => c,
            Err(_) => return CacheState::Miss,
        };
        let registry: Registry = match serde_json::from_str(&registry_content) {
            Ok(r) => r,
            Err(_) => return CacheState::Miss,
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if now.saturating_sub(meta.fetched_at) > self.ttl.as_secs() {
            CacheState::Stale(registry)
        } else {
            CacheState::Fresh(registry)
        }
    }

    pub fn set(&self, kind: RegistryKind, registry: &Registry) -> Result<()> {
        let dir = self.kind_dir(kind);
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create cache dir: {}", dir.display()))?;

        let registry_json = serde_json::to_string_pretty(registry)?;
        std::fs::write(dir.join("registry.json"), registry_json)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let meta = CacheMeta { fetched_at: now };
        let meta_json = serde_json::to_string(&meta)?;
        std::fs::write(dir.join("meta.json"), meta_json)?;

        Ok(())
    }
}
