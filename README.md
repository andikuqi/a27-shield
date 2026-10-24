# A27-shield

Hidden-origin reverse proxies for self-hosted game servers, by **Andi Kuqi**.

Two companion tools keep your servers reachable by placing a proxy on a
DDoS-protected public IP in front of a **secret origin IP**. They validate and
rate-limit inbound traffic, drop obvious junk/spoof floods, and hide your real
server address.

| Tool | Protocol | Games / services | Ports |
|------|----------|------------------|-------|
| **A27-UDP-shield** | UDP | SA-MP, FiveM (game side), Minecraft Bedrock | 7777, 30120/udp, 19132 |
| **A27-TCP-shield** | TCP | Minecraft Java, FiveM (TCP side) | 25565, 30120/tcp |

> **FiveM uses both UDP and TCP on 30120.** Run *both* shields for it: a UDP
> entry in A27-UDP-shield and a TCP entry in A27-TCP-shield.

---

## How it works

```
Players ─▶  PROXY MACHINE                 ─▶  GAME SERVER
            (protected public IP,              (origin, SECRET IP,
             runs A27-shield)                   firewalled to only
             validates + rate-limits            accept the proxy's IP)
             hides the origin
```

This runs on a **separate protected machine in front of** the game server — not
inside it. The protection comes from the *architecture*, and the proxy is one
piece of it. Three steps are required for it to actually work:

1. **Run the shield on a DDoS-protected IP** (OVH, Path.net, Cloudflare
   Spectrum, etc.).
2. **Point players/DNS at the proxy IP**, never at the real server.
3. **Firewall the origin** to accept traffic *only* from the proxy's IP (see
   below) — this is what makes hiding the origin meaningful.

### What it does and does not stop

- ✅ Spoofed, malformed, and query floods; per-IP rate abuse; origin exposure.
- ❌ **Volumetric floods that saturate your uplink.** No userspace proxy in any
  language can fix that — it must be scrubbed **upstream** by your provider. The
  shield reduces your attack surface and hides the origin; it is not a
  replacement for upstream DDoS protection.

---

## Building

Both tools are written in Rust (`tokio`) and build unchanged on **Linux (any
distro), macOS, and Windows**. Install Rust from <https://rustup.rs> first.

```bash
# UDP shield
cd A27-UDP-shield
cargo build --release

# TCP shield
cd ../A27-TCP-shield
cargo build --release
```

Run with a config path (defaults to `config.json`):

| OS | Run |
|----|-----|
| Linux / macOS | `./target/release/a27-udp-shield config.json` |
| Windows | `target\release\a27-udp-shield.exe config.json` |

(Same pattern for `a27-tcp-shield`.) All default ports are above 1024, so no
`root`/`sudo` is needed on Linux/macOS.

---

## Deploying on a Linux VPS (recommended)

A Linux VPS on a DDoS-protected IP is the intended deployment. Example for the
UDP shield (repeat for the TCP shield with its own name/paths):

```bash
# 1. Build (on the VPS, or cross-compile and upload the binary)
cd A27-UDP-shield && cargo build --release

# 2. Install binary + config
sudo mkdir -p /opt/a27-udp-shield
sudo cp target/release/a27-udp-shield /opt/a27-udp-shield/
sudo cp config.json /opt/a27-udp-shield/

# 3. Create an unprivileged service user
sudo useradd --system --no-create-home a27
sudo chown -R a27:a27 /opt/a27-udp-shield

# 4. Install and start the service
sudo cp systemd/a27-udp-shield.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now a27-udp-shield

# 5. Check it
systemctl status a27-udp-shield
journalctl -u a27-udp-shield -f
```

The provided `systemd/*.service` units auto-start on boot, auto-restart on
failure, and run under a locked-down unprivileged user.

---

## Configuration

Each tool reads a JSON file listing the services to protect. Set every `origin`
to your **hidden backend IP:port**. See each project's `config.json` for a
working example. UDP services support packet-size filtering and per-IP packet
rate limiting; TCP services support per-IP concurrent-connection caps and
new-connection rate limiting.

---

## Firewalling the origin (required)

Once players reach the proxy, lock the origin so it only accepts the proxy.
Replace `PROXY_IP` with your proxy machine's address.

**Linux (nftables / iptables):**
```bash
# UDP example (SA-MP 7777) — allow only the proxy, drop the rest
iptables -A INPUT -p udp --dport 7777 -s PROXY_IP -j ACCEPT
iptables -A INPUT -p udp --dport 7777 -j DROP

# TCP example (Minecraft Java 25565)
iptables -A INPUT -p tcp --dport 25565 -s PROXY_IP -j ACCEPT
iptables -A INPUT -p tcp --dport 25565 -j DROP
```

**Windows (origin on Windows) — PowerShell:**
```powershell
New-NetFirewallRule -DisplayName "A27 allow proxy 25565" -Direction Inbound `
  -Protocol TCP -LocalPort 25565 -RemoteAddress PROXY_IP -Action Allow
New-NetFirewallRule -DisplayName "A27 block 25565" -Direction Inbound `
  -Protocol TCP -LocalPort 25565 -Action Block
```

---

## License

MIT © 2026 Andi Kuqi. See each project's `LICENSE` file.

---

## Intended use

A27-shield is a **defensive** tool for operators protecting their **own** game
servers. Deploy it only on infrastructure you own or are authorized to protect.
