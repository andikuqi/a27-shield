# A27-shield

Hidden-origin reverse proxies for self-hosted game servers (FiveM, Minecraft,
SA-MP, Multi Theft Auto [+ more game servers](#supported-games)), by **Andi Kuqi**.

Two companion tools keep your servers reachable by placing a proxy on a
DDoS-protected public IP in front of a **secret origin IP**. They validate and
rate-limit inbound traffic, drop obvious junk/spoof floods, and hide your real
server address.

| Tool | Protocol | Games / services | Ports |
|------|----------|------------------|-------|
| **A27-UDP-shield** | UDP | FiveM (game side), Minecraft Bedrock, SA-MP, Multi Theft Auto [+ more game servers](#supported-games) | 30120/udp, 19132, 7777, 22003 |
| **A27-TCP-shield** | TCP | FiveM (TCP side), Minecraft Java [+ more game servers](#supported-games) | 30120/tcp, 25565 |

> **FiveM uses both UDP and TCP on 30120.** Run *both* shields for it: a UDP
> entry in A27-UDP-shield and a TCP entry in A27-TCP-shield.

---

## Supported games

A27-shield is protocol-agnostic: it works with **any** game or voice server that
runs over UDP or TCP. Both `config.json` files ship with ready-made entries for
the games below. The **bold** games are enabled by default. To switch one
on or off, set `"enabled": true` / `false` on its entries, and point `origin`
at your hidden server.

> **Ports must be unique per shield.** Many games share defaults (UDP 7777,
> 27015), so if you run two of them, change one server's port and its entry.
> Ports listed are each game's defaults. Match them to your own server config.

### Featured

| Game | UDP | TCP |
|------|-----|-----|
| **FiveM (GTA V)** | 30120 | 30120 |
| **Minecraft Java Edition** | — | 25565, 25575 (RCON, off by default) |
| **Minecraft Bedrock Edition** | 19132 (RakNet) | — |
| **SA-MP / open.mp** | 7777 | — |
| **Multi Theft Auto: San Andreas** | 22003 (game), 22126 | 22005 |

### GTA / RDR multiplayer

| Game | UDP | TCP |
|------|-----|-----|
| RedM (RDR2) | 30120 | 30120 |
| RAGE:MP (GTA V) | 22005 | 22006 (HTTP resource downloads) |
| alt:V (GTA V) | 7788 | 7788 |

### Survival / crafting

| Game | UDP | TCP |
|------|-----|-----|
| Rust | 28015 (game), 28017 (server query) | 28016 (RCON) |
| ARK: Survival Evolved | 7777, 27015 (Steam query) | — |
| ARK: Survival Ascended | 7777 | — |
| Conan Exiles | 7777 (game), 27015 (Steam query) | — |
| DayZ | 2302 (game), 27016 (Steam query) | — |
| Valheim | 2456 (game), 2457 (query) | — |
| Palworld | 8211 | — |
| Enshrouded | 15636 (game), 15637 (query) | — |
| V Rising | 9876 (game), 9877 (query) | — |
| Project Zomboid | 16261 (game), 16262 (direct connection) | — |
| 7 Days to Die | 26900 | 26900 |
| Satisfactory | 7777 (game) | 7777 (HTTPS API), 8888 (reliable messaging) |
| Sons of the Forest | 8766 (game), 27016 (Steam query), 9700 (blob sync) | — |
| The Forest | 8766 (game), 27015 (Steam query) | — |
| Unturned | 27015 (game), 27016 (query) | — |
| Soulmask | 8777 (game), 27015 (Steam query) | — |
| Icarus | 17777 (game), 27015 (Steam query) | — |
| Abiotic Factor | 7777 (game), 27015 (Steam query) | — |
| The Isle (Evrima) | 7777 | — |
| Astroneer | 8777 | — |
| Hurtworld | 12871 (game), 12881 (query) | — |
| Necesse | 14159 | — |
| Eco | 3000 (game) | 3001 (web interface) |
| Hytale | 5520 (QUIC) | — |
| Terraria / tModLoader | — | 7777 |
| Starbound | — | 21025 |
| Vintage Story | — | 42420 |
| Don't Starve Together | 10999 | — |
| Factorio | 34197 | — |
| Luanti (Minetest) | 30000 | — |

### Space / building

| Game | UDP | TCP |
|------|-----|-----|
| Space Engineers | 27016 | — |
| Empyrion: Galactic Survival | 30000 (also uses 30001-30004) | — |
| Avorion | 27000 | 27000 |
| Barotrauma | 27015 (game), 27016 (query) | — |
| Stormworks | 25564 (also uses 25565-25566) | — |
| Space Station 14 | 1212 (game) | 1212 (HTTP status API) |

### Shooters

| Game | UDP | TCP |
|------|-----|-----|
| Counter-Strike 2 / CS:GO / CS 1.6 / Source engine (TF2, Garry's Mod, L4D2) | 27015 (game + query) | 27015 (RCON) |
| Arma 3 | 2302 | — |
| Arma Reforger | 2001 (game), 17777 (A2S query) | — |
| Squad | 7787 (game), 27165 (Steam query) | — |
| Insurgency: Sandstorm | 27102 (game), 27131 (query) | — |
| Mordhau | 7777 (game), 27015 (Steam query) | — |
| Killing Floor 2 | 7777 (game), 27015 (Steam query) | — |
| Pavlov VR | 7777 | — |
| Call of Duty 4: Modern Warfare | 28960 | — |
| Quake III Arena / OpenArena | 27960 | — |
| Teeworlds / DDNet | 8303 | — |

### Racing / sim / strategy

| Game | UDP | TCP |
|------|-----|-----|
| Assetto Corsa | 9600 (game) | 9600, 8081 (HTTP lobby) |
| BeamMP (BeamNG.drive) | 30814 | 30814 |
| Euro Truck Simulator 2 / American Truck Simulator | 27015 (game), 27016 (query) | — |
| Mount & Blade II: Bannerlord | 7210 | — |
| OpenTTD | 3979 | 3979 |

### Party / social

| Game | UDP | TCP |
|------|-----|-----|
| Among Us (Impostor server) | 22023 | — |

### Voice

| Game | UDP | TCP |
|------|-----|-----|
| TeamSpeak 3 | 9987 (voice) | 30033 (file transfer), 10011 (ServerQuery) |
| Mumble | 64738 (voice) | 64738 (control channel) |

Not listed? Add an entry with your game's port. Nothing in the shields is
game-specific.

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
working example. Set `"enabled": false` to keep an entry in the file without
starting it; the shield refuses to start if two enabled services share a listen
address. UDP services support packet-size filtering and per-IP packet
rate limiting; TCP services support per-IP concurrent-connection caps and
new-connection rate limiting.

---

## Firewalling the origin (required)

Once players reach the proxy, lock the origin so it only accepts the proxy.
Replace `PROXY_IP` with your proxy machine's address.

**Linux (nftables / iptables):**
```bash
# UDP example (FiveM 30120) — allow only the proxy, drop the rest
iptables -A INPUT -p udp --dport 30120 -s PROXY_IP -j ACCEPT
iptables -A INPUT -p udp --dport 30120 -j DROP

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
