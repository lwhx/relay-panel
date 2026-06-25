# TLS Simple Ingress (v0.4.1)

relay-node can terminate TLS directly (via rustls) and forward the decrypted
TCP stream to the target. No WebSocket, no reverse proxy — the node IS the
TLS server.

```
client ──TLS──▶ relay-node (rustls terminates) ──plain TCP──▶ target
```

This is the simplest way to get encrypted TCP forwarding. Unlike WSS (which
was removed in v0.4.1), there's no reverse proxy to configure.

> **v0.4.11 PR1 note:** TLS Simple rules now require binding a TLS Simple
> transport template (`GET /tunnel-profiles`). The certificate is still
> configured via node environment variables (`TLS_CERT_PATH`, `TLS_KEY_PATH`);
> template-level certificate/SNI configuration is planned for a future release.

## Prerequisites

- relay-node v0.4.1+ (panel and node must run the same `CONFIG_PROTOCOL_VERSION`; see `crates/shared/src/protocol.rs` for the current value).
- A TLS certificate + private key in PEM format on the relay-node host.
- The rule's protocol must be TCP (TLS Simple does not support UDP).

## Step 1: Obtain a certificate

### Option A: Let's Encrypt (recommended for production)

```bash
# Install certbot (if not already installed)
sudo apt install -y certbot

# Obtain a certificate (standalone mode — temporarily stops anything on port 80)
sudo certbot certonly --standalone -d relay.example.com

# The cert files are at:
#   /etc/letsencrypt/live/relay.example.com/fullchain.pem
#   /etc/letsencrypt/live/relay.example.com/privkey.pem
```

Copy or symlink them to the relay-node certs directory:

```bash
sudo cp /etc/letsencrypt/live/relay.example.com/fullchain.pem /opt/relay-node/certs/
sudo cp /etc/letsencrypt/live/relay.example.com/privkey.pem /opt/relay-node/certs/
sudo chmod 600 /opt/relay-node/certs/privkey.pem
```

### Option B: Self-signed (for testing/internal use)

```bash
openssl req -x509 -newkey rsa:2048 -keyout /opt/relay-node/certs/privkey.pem \
    -out /opt/relay-node/certs/fullchain.pem -days 365 -nodes \
    -subj "/CN=relay.example.com" \
    -addext "subjectAltName=DNS:relay.example.com"

sudo chmod 600 /opt/relay-node/certs/privkey.pem
```

> Self-signed certs must be manually trusted by the client (e.g. add to the
> trust store, or pass `-k`/`--insecure` to skip verification).

## Step 2: Configure relay-node

Edit the env file created by the installer:

```bash
sudo nano /opt/relay-node/relay-node.env
```

Uncomment and set the paths:

```bash
TLS_CERT_PATH=/opt/relay-node/certs/fullchain.pem
TLS_KEY_PATH=/opt/relay-node/certs/privkey.pem
```

Restart relay-node:

```bash
sudo systemctl restart relay-node
```

Check the log for "TLS certificate loaded — tls_simple enabled, hot-reload active":

```bash
sudo journalctl -u relay-node --since "1 min ago" | grep TLS
```

## Step 3: Create a TLS Simple rule

In the panel UI, create a forwarding rule with:
- **Protocol**: TCP
- **Transport Method**: TLS Simple
- **Transport Template**: Select a TLS Simple template (e.g., the builtin "tls-simple" template)
- **Listen Port**: (e.g. 443)

The node will listen on the specified port with TLS and forward decrypted TCP
to the target. Certificate configuration is still done via the node's environment
variables (`TLS_CERT_PATH`, `TLS_KEY_PATH`); template-level certificate/SNI
will be supported in a future release.

## Verifying

### openssl s_client

```bash
openssl s_client -connect relay.example.com:443 -servername relay.example.com
```

You should see the certificate chain and "Verify return code: 0 (ok)" for a
trusted cert. Then type any raw TCP data — it forwards to the target.

### curl (if the target speaks HTTP)

```bash
curl --resolve relay.example.com:443:127.0.0.1 https://relay.example.com/
```

## Hot reload

relay-node watches the cert + key files (mtime polling, every 5 seconds). When
you replace them (e.g. certbot renewal), the new cert takes effect on the next
connection — **no restart needed**.

```bash
# After certbot renewal:
sudo cp /etc/letsencrypt/live/relay.example.com/fullchain.pem /opt/relay-node/certs/
sudo cp /etc/letsencrypt/live/relay.example.com/privkey.pem /opt/relay-node/certs/
sudo chmod 600 /opt/relay-node/certs/privkey.pem
# relay-node logs "TLS cert reloaded successfully" within 5 seconds.
```

If the new cert/key is invalid (bad PEM, cert↔key mismatch), relay-node keeps
the old cert and logs a warning + reports the error to the panel.

## Security notes

- **TLS 1.2+ only.** TLS 1.0/1.1 are rejected (insecure).
- **No client certificate authentication.** The node presents a cert; it does
  not verify the client's identity.
- **Private key permissions.** The key file MUST be `chmod 600` (owner-only).
  relay-node refuses to load a key readable by group/other.
- **Private key never logged.** Only generic error messages appear in logs —
  never the key content, file paths in errors, or PEM data.
- **Handshake timeout.** A TLS handshake that doesn't complete within 10
  seconds is closed (prevents slow-loris resource exhaustion).
- **SNI is the client's responsibility.** The node presents its cert regardless
  of SNI; the client validates that the cert matches the hostname it connected to.

## Common issues

| Symptom | Cause | Fix |
|---|---|---|
| Panel shows "tls_simple skipped: no TLS certificate configured" | `relay-node.env` not configured or paths wrong | Edit `/opt/relay-node/relay-node.env`, restart node. |
| `journalctl` shows "private key file is too open" | Key file mode is not 0600 | `sudo chmod 600 /opt/relay-node/certs/privkey.pem` |
| `openssl s_client` shows "certificate verify failed" | Self-signed or expired cert | Use a trusted cert (Let's Encrypt) or trust the self-signed cert on the client. |
| Panel shows "TLS reload failed (keeping old cert)" | New cert/key invalid | Check the cert+key files are valid PEM and the cert matches the key. |
| TLS listener not starting | Protocol is UDP or TCP+UDP | TLS Simple only supports TCP. |
| `config protocol incompatible` on Nodes page | Node and panel run different `CONFIG_PROTOCOL_VERSION` | Upgrade relay-node to match the panel (see the version in `crates/shared/src/protocol.rs`). Panel and node must always run the same config protocol version. |

## What's NOT supported

- **UDP over TLS Simple.** TLS is TCP-only; UDP rules must use Raw transport.
- **SNI-based multi-cert.** v0.4.1 uses a single cert for all TLS Simple
  listeners. SNI-based cert selection is a future enhancement.
- **Client certificate authentication.** The node does not verify client certs.
  (Mutual TLS is possible but not exposed in v0.4.1.)
- **Template-level certificate/SNI configuration.** v0.4.11 PR1: TLS Simple
  templates do not yet support per-template certificate or SNI settings.
  Certificate configuration uses the node's global `TLS_CERT_PATH`/`TLS_KEY_PATH`
  environment variables. Template-level cert/SNI will be supported in a future release.
