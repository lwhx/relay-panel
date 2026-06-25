use serde::Deserialize;

/// Sentinel value: if NODE_TOKEN is unset (or still the built-in default), the
/// node must refuse to start. A node with the default token either can't
/// authenticate to the panel at all, or worse, binds to a group someone
/// happened to create with token "default-token" — a silent misconfiguration
/// that's much worse than a loud startup failure. Mirrors the panel's
/// JWT_SECRET guard.
const INSECURE_NODE_TOKEN: &str = "default-token";

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub panel_url: String,
    pub token: String,
    pub poll_interval: u64,
    /// v0.4.1: TLS Simple certificate path. Optional — if unset, tls_simple
    /// rules are skipped (the node can't serve TLS without a cert). Set via
    /// the TLS_CERT_PATH environment variable.
    pub tls_cert_path: Option<String>,
    /// v0.4.1: TLS Simple private key path. Paired with tls_cert_path.
    pub tls_key_path: Option<String>,
    /// v0.4.6: which NIC to count for machine-wide traffic stats.
    /// "auto" (default) = auto-detect the default-route interface;
    /// any other value = that exact interface name (e.g. "eth0").
    pub network_interface: String,
}

impl NodeConfig {
    pub fn load() -> Self {
        let panel_url =
            std::env::var("PANEL_URL").unwrap_or_else(|_| "http://127.0.0.1:18888".into());
        // No fallback default for the token: an unset NODE_TOKEN is a
        // misconfiguration, not "use a known value". Refuse to boot.
        let token = std::env::var("NODE_TOKEN").unwrap_or_else(|_| String::new());
        let poll_interval = std::env::var("POLL_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let cfg = Self {
            panel_url,
            token,
            poll_interval,
            tls_cert_path: std::env::var("TLS_CERT_PATH")
                .ok()
                .filter(|s| !s.is_empty()),
            tls_key_path: std::env::var("TLS_KEY_PATH").ok().filter(|s| !s.is_empty()),
            // v0.4.6: default "auto" picks the default-route NIC; empty also
            // means auto so an unset/blank value behaves safely.
            network_interface: std::env::var("NETWORK_INTERFACE")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "auto".to_string()),
        };
        cfg.validate();
        cfg
    }

    /// Refuse to start with an unset or default token. The operator MUST set
    /// NODE_TOKEN to a real group token from the panel UI. An empty value
    /// (env unset) or the sentinel "default-token" is treated as insecure.
    fn validate(&self) {
        if self.token.trim().is_empty() {
            eprintln!(
                "FATAL: NODE_TOKEN is not set.\n  \
                 Set it to a real inbound-group token from the panel UI, e.g.:\n  \
                 NODE_TOKEN=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
            );
            std::process::exit(1);
        }
        if self.token == INSECURE_NODE_TOKEN {
            eprintln!(
                "FATAL: NODE_TOKEN is still set to the insecure default \"{}\".\n  \
                 Set it to a real inbound-group token from the panel UI.",
                INSECURE_NODE_TOKEN
            );
            std::process::exit(1);
        }
    }
}
