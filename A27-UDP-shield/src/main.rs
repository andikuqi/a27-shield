//! A27-UDP-shield: a hidden-origin UDP reverse proxy for game servers (Rust/tokio).
//! Created by Andi.
//!
//! Purpose (defensive): keep self-hosted game servers reachable by placing a
//! proxy on a DDoS-protected public IP in front of a secret origin IP. The
//! proxy validates and rate-limits inbound packets, drops obvious junk/spoof
//! floods, hides the origin, and forwards clean traffic to the backend.
//!
//! This runs on a SEPARATE protected machine in front of the game server, not
//! inside it. Firewall the origin to accept UDP only from this proxy's IP.
//! It cannot stop a volumetric flood that saturates your uplink — that must be
//! scrubbed upstream by your provider.
//!
//! Build:  cargo build --release
//! Run:    ./target/release/a27-udp-shield config.json

use serde::Deserialize;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time::{sleep, timeout};

#[derive(Debug, Deserialize, Clone)]
struct Service {
    name: String,
    listen: String,
    origin: String,
    #[serde(default)]
    min_packet_size: usize,
    #[serde(default)]
    max_packet_size: usize, // 0 => defaults to 2048
    #[serde(default)]
    rate_per_sec: u32, // 0 => rate limiting disabled
    #[serde(default)]
    burst_size: u32,
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    session_idle_sec: u64, // 0 => defaults to 60
    services: Vec<Service>,
}

/// Per-source-IP token bucket.
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
            return true; // disabled
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

/// One client -> dedicated upstream socket (NAT-style).
struct Session {
    upstream: Arc<UdpSocket>,
    last_seen: Instant,
    alive: Arc<AtomicBool>,
}

/// Per-game validation hook. Keep conservative: only drop what is clearly not a
/// valid client packet. Refine against real captures from your own servers.
fn validate(game: &str, pkt: &[u8]) -> bool {
    match game {
        "samp" | "fivem" | "bedrock" => !pkt.is_empty(),
        _ => !pkt.is_empty(),
    }
}

async fn proxy_service(svc: Service, idle: Duration) -> std::io::Result<()> {
    let max_pkt = if svc.max_packet_size == 0 {
        2048
    } else {
        svc.max_packet_size
    };
    let origin: SocketAddr = svc
        .origin
        .parse()
        .unwrap_or_else(|_| panic!("[{}] bad origin {}", svc.name, svc.origin));

    let public = Arc::new(UdpSocket::bind(&svc.listen).await?);
    println!("[{}] listening {} -> origin {}", svc.name, svc.listen, svc.origin);

    let limiter = Arc::new(Limiter::new(svc.rate_per_sec, svc.burst_size));
    let sessions: Arc<Mutex<HashMap<SocketAddr, Session>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Idle-session reaper.
    {
        let sessions = sessions.clone();
        tokio::spawn(async move {
            loop {
                sleep(idle).await;
                let cutoff = Instant::now()
                    .checked_sub(idle)
                    .unwrap_or_else(Instant::now);
                let mut map = sessions.lock().unwrap();
                map.retain(|_, s| {
                    if s.last_seen < cutoff {
                        s.alive.store(false, Ordering::Relaxed);
                        false
                    } else {
                        true
                    }
                });
            }
        });
    }

    let mut buf = vec![0u8; 65535];
    loop {
        let (n, client) = match public.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[{}] recv error: {}", svc.name, e);
                continue;
            }
        };

        // Cheap drops first: size sanity.
        if n < svc.min_packet_size || n > max_pkt {
            continue;
        }
        // Per-source-IP rate limit (spoof/flood dampening).
        if !limiter.allow(client.ip()) {
            continue;
        }
        // Per-game validation.
        if !validate(&svc.name, &buf[..n]) {
            continue;
        }

        // Find or create the session for this client.
        let upstream: Arc<UdpSocket> = {
            let existing = {
                let mut map = sessions.lock().unwrap();
                map.get_mut(&client).map(|s| {
                    s.last_seen = Instant::now();
                    s.upstream.clone()
                })
            };
            match existing {
                Some(up) => up,
                None => {
                    // Bind an ephemeral upstream socket and connect to origin.
                    let up = match UdpSocket::bind("0.0.0.0:0").await {
                        Ok(u) => u,
                        Err(e) => {
                            eprintln!("[{}] bind upstream: {}", svc.name, e);
                            continue;
                        }
                    };
                    if let Err(e) = up.connect(origin).await {
                        eprintln!("[{}] connect origin: {}", svc.name, e);
                        continue;
                    }
                    let up = Arc::new(up);
                    let alive = Arc::new(AtomicBool::new(true));

                    // Reply pump: origin -> client.
                    {
                        let up = up.clone();
                        let public = public.clone();
                        let alive = alive.clone();
                        tokio::spawn(async move {
                            let mut rbuf = vec![0u8; 65535];
                            loop {
                                match timeout(Duration::from_secs(5), up.recv(&mut rbuf)).await {
                                    Ok(Ok(rn)) => {
                                        let _ = public.send_to(&rbuf[..rn], client).await;
                                    }
                                    Ok(Err(_)) => break,
                                    Err(_) => {
                                        if !alive.load(Ordering::Relaxed) {
                                            break;
                                        }
                                    }
                                }
                            }
                        });
                    }

                    sessions.lock().unwrap().insert(
                        client,
                        Session {
                            upstream: up.clone(),
                            last_seen: Instant::now(),
                            alive,
                        },
                    );
                    up
                }
            }
        };

        if let Err(e) = upstream.send(&buf[..n]).await {
            eprintln!("[{}] send origin: {}", svc.name, e);
        }
    }
}

#[tokio::main]
async fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "config.json".into());
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read config {}: {}", path, e));
    let mut cfg: Config =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse config: {}", e));
    if cfg.session_idle_sec == 0 {
        cfg.session_idle_sec = 60;
    }
    if cfg.services.is_empty() {
        panic!("no services configured");
    }

    println!("A27-UDP-shield starting — {} service(s)", cfg.services.len());

    let idle = Duration::from_secs(cfg.session_idle_sec);
    let mut handles = Vec::new();
    for svc in cfg.services {
        let idle = idle;
        handles.push(tokio::spawn(async move {
            if let Err(e) = proxy_service(svc.clone(), idle).await {
                eprintln!("[{}] fatal: {}", svc.name, e);
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
}
