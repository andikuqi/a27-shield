//! A27-TCP-shield: a hidden-origin TCP reverse proxy for game servers (Rust/tokio).
//! Created by Andi.
//!
//! Companion to A27-UDP-shield. Use this one for TCP-based services:
//!   - Minecraft Java  (TCP 25565)
//!   - FiveM TCP side  (TCP 30120 — FiveM uses BOTH udp+tcp on 30120; the UDP
//!                      side goes to A27-UDP-shield, the TCP side goes here)
//!
//! Purpose (defensive): place a proxy on a DDoS-protected public IP in front of
//! a secret origin IP. It hides the origin, caps concurrent connections per
//! source IP, and rate-limits new connections per IP to dampen connection
//! floods, then streams clean traffic to the backend.
//!
//! Runs on a SEPARATE protected machine in front of the game server, not inside
//! it. Firewall the origin to accept TCP only from this proxy's IP. It cannot
//! stop a volumetric flood that saturates your uplink — scrub that upstream.
//!
//! Cross-platform: builds on Linux, macOS, and Windows unchanged.
//! Build:  cargo build --release
//! Run:    ./target/release/a27-tcp-shield config.json

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Deserialize, Clone)]
struct Service {
    name: String,
    #[serde(default = "default_true")]
    enabled: bool, // false => entry is kept in the config but not started
    listen: String,
    origin: String,
    #[serde(default)]
    max_conns_per_ip: usize, // 0 => unlimited concurrent conns per IP
    #[serde(default)]
    conn_rate_per_sec: u32, // 0 => new-connection rate limiting disabled
    #[serde(default)]
    conn_burst: u32,
}

#[derive(Debug, Deserialize)]
struct Config {
    services: Vec<Service>,
}

fn default_true() -> bool {
    true
}

/// Per-source-IP token bucket for NEW connections.
struct Bucket {
    tokens: f64,
    last: Instant,
}

struct Limiter {
    buckets: Mutex<HashMap<IpAddr, Bucket>>,
    rate: f64,
    capacity: f64,
}

impl Limiter {
    fn new(rate: u32, burst: u32) -> Self {
        Limiter {
            buckets: Mutex::new(HashMap::new()),
            rate: rate as f64,
            capacity: burst as f64,
        }
    }

    fn allow(&self, ip: IpAddr) -> bool {
        if self.rate <= 0.0 {
            return true;
        }
        let now = Instant::now();
        let mut map = self.buckets.lock().unwrap();
        let b = map.entry(ip).or_insert(Bucket {
            tokens: self.capacity,
            last: now,
        });
        let elapsed = now.duration_since(b.last).as_secs_f64();
        b.last = now;
        b.tokens = (b.tokens + elapsed * self.rate).min(self.capacity);
        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Tracks concurrent connection counts per source IP.
struct ConnCounter {
    counts: Mutex<HashMap<IpAddr, usize>>,
    max_per_ip: usize,
}

impl ConnCounter {
    fn new(max_per_ip: usize) -> Self {
        ConnCounter {
            counts: Mutex::new(HashMap::new()),
            max_per_ip,
        }
    }

    /// Try to reserve a slot for `ip`. Returns false if the per-IP cap is hit.
    fn acquire(&self, ip: IpAddr) -> bool {
        let mut map = self.counts.lock().unwrap();
        let c = map.entry(ip).or_insert(0);
        if self.max_per_ip != 0 && *c >= self.max_per_ip {
            return false;
        }
        *c += 1;
        true
    }

    fn release(&self, ip: IpAddr) {
        let mut map = self.counts.lock().unwrap();
        if let Some(c) = map.get_mut(&ip) {
            *c = c.saturating_sub(1);
            if *c == 0 {
                map.remove(&ip);
            }
        }
    }
}

async fn proxy_service(svc: Service) -> std::io::Result<()> {
    let listener = TcpListener::bind(&svc.listen).await?;
    println!("[{}] listening {} -> origin {}", svc.name, svc.listen, svc.origin);

    let limiter = Arc::new(Limiter::new(svc.conn_rate_per_sec, svc.conn_burst));
    let counter = Arc::new(ConnCounter::new(svc.max_conns_per_ip));
    let origin = Arc::new(svc.origin.clone());
    let name = Arc::new(svc.name.clone());

    loop {
        let (client, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[{}] accept error: {}", svc.name, e);
                continue;
            }
        };
        let ip = peer.ip();

        // New-connection rate limit (dampen connection floods).
        if !limiter.allow(ip) {
            continue; // drop by closing immediately
        }
        // Concurrent-connection cap per IP.
        if !counter.acquire(ip) {
            continue;
        }

        let limiter = limiter.clone();
        let _ = &limiter; // kept alive for the task's lifetime
        let counter = counter.clone();
        let origin = origin.clone();
        let name = name.clone();

        tokio::spawn(async move {
            let result = async {
                let mut client = client;
                let mut upstream = TcpStream::connect(origin.as_str()).await?;
                // Disable Nagle for lower latency on game traffic.
                let _ = client.set_nodelay(true);
                let _ = upstream.set_nodelay(true);
                copy_bidirectional(&mut client, &mut upstream).await
            }
            .await;

            if let Err(e) = result {
                // Normal disconnects show up here too; log quietly.
                eprintln!("[{}] conn from {} closed: {}", name, ip, e);
            }
            counter.release(ip);
        });
    }
}

#[tokio::main]
async fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "config.json".into());
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read config {}: {}", path, e));
    let mut cfg: Config =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse config: {}", e));
    cfg.services.retain(|s| s.enabled);
    if cfg.services.is_empty() {
        panic!("no enabled services configured");
    }
    // Two services cannot bind the same address; fail loudly instead of
    // silently losing one of them.
    let mut seen = HashSet::new();
    for s in &cfg.services {
        if !seen.insert(s.listen.clone()) {
            panic!(
                "duplicate listen address {} (service '{}') — each enabled service needs its own port",
                s.listen, s.name
            );
        }
    }

    println!("A27-TCP-shield starting — {} service(s)", cfg.services.len());

    let mut handles = Vec::new();
    for svc in cfg.services {
        handles.push(tokio::spawn(async move {
            if let Err(e) = proxy_service(svc.clone()).await {
                eprintln!("[{}] fatal: {}", svc.name, e);
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
}
