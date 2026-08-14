//! Zenoh subscriber client (`main_sub`).

use crate::ResolvedSubConfig;
use std::collections::{HashMap, hash_map::Entry};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use zenoh::Wait;

/// Topic name tails to skip (e.g. rosout, parameter_events).
const IGNORE_SAMPLE_TOPIC_TAILS: &[&str] = &["parameter_events", "rosout"];

fn sample_key_expr_is_ignored(ke: &str) -> bool {
    let tail = ke.rsplit_once('/').map(|(_, tail)| tail).unwrap_or(ke);
    IGNORE_SAMPLE_TOPIC_TAILS.iter().any(|&n| tail == n)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitPolicy {
    UntilCtrlC,
}

pub struct SubscribersArgs {
    pub config_path: PathBuf,
    /// Print discover lines for new and stale topics.
    pub discover: bool,
    pub keyexpr: String,
    pub wait: WaitPolicy,
    /// Zenoh router address override (e.g. `tcp/host:7447`).
    pub router_connect_override: Option<String>,
    /// Mark a topic stale after this long with no samples (`None` disables).
    pub topic_stale_after: Option<Duration>,
}

pub fn subscriber_args(sub: &ResolvedSubConfig, wait: WaitPolicy) -> SubscribersArgs {
    SubscribersArgs {
        config_path: std::path::PathBuf::new(),
        discover: sub.discover,
        keyexpr: sub.keyexpr.clone(),
        wait,
        router_connect_override: None,
        topic_stale_after: None,
    }
}

fn sample_time_or_recv_ms(sample: &zenoh::sample::Sample) -> String {
    if let Some(ts) = sample.timestamp() {
        format!("{ts:?}")
    } else {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("recv_unix_ms={ms}")
    }
}

fn print_sample_line(sample: &zenoh::sample::Sample) {
    let key = sample.key_expr();
    if sample_key_expr_is_ignored(key.as_str()) {
        return;
    }
    let enc = sample.encoding();
    let body_len = sample.payload().len();
    let kind = sample.kind();
    let time = sample_time_or_recv_ms(sample);
    println!("[info] {key}: {{time={time}, body_len={body_len}, encoding={enc:?}, kind={kind}}}");
}

fn log_alive_keys_roll_up(alive: &Arc<Mutex<HashMap<String, Instant>>>, print_when_empty: bool) {
    let keys = {
        let g = alive.lock().expect("alive Mutex poisoned");
        let mut v: Vec<String> = g.keys().cloned().collect();
        v.sort();
        v
    };
    if keys.is_empty() {
        if print_when_empty {
            println!("(discover) No keys currently alive.");
        }
        return;
    }
    println!(
        "(discover) Alive keys ({}) — observed recently, not stale yet:",
        keys.len()
    );
    for k in keys {
        println!("(discover)   • {k}");
    }
}

async fn stale_topic_watchdog(alive: Arc<Mutex<HashMap<String, Instant>>>, stale_after: Duration) {
    let mut iv = tokio::time::interval(Duration::from_secs(1));
    iv.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        iv.tick().await;
        let now = Instant::now();
        let mut g = alive.lock().expect("alive Mutex poisoned");
        let stale_keys: Vec<String> = g
            .iter()
            .filter_map(|(k, t)| (now.duration_since(*t) >= stale_after).then(|| k.clone()))
            .collect();
        for k in stale_keys {
            g.remove(&k);
            println!(
                "(discover) stale `{k}` — no sample for {:?} (heuristic; publisher may still exist)",
                stale_after
            );
        }
    }
}

async fn subscribe_parallel_discover_and_samples(
    session: &zenoh::Session,
    ke: &str,
    topic_stale_after: Option<Duration>,
) -> anyhow::Result<()> {
    let alive: Arc<Mutex<HashMap<String, Instant>>> = Arc::new(Mutex::new(HashMap::new()));

    if let Some(stale_after) = topic_stale_after {
        let av = Arc::clone(&alive);
        tokio::spawn(stale_topic_watchdog(av, stale_after));
    }

    let a_cb = Arc::clone(&alive);
    session
        .declare_subscriber(ke)
        .callback(move |sample| {
            let ks = sample.key_expr().to_string();
            if sample_key_expr_is_ignored(&ks) {
                return;
            }

            let (just_became_alive, n_alive) = {
                let mut g = a_cb.lock().expect("alive Mutex poisoned");
                let just_became_alive = match g.entry(ks.clone()) {
                    Entry::Occupied(mut e) => {
                        e.insert(Instant::now());
                        false
                    }
                    Entry::Vacant(e) => {
                        e.insert(Instant::now());
                        true
                    }
                };
                (just_became_alive, g.len())
            };

            if just_became_alive {
                println!(
                    "(discover) alive `{}` encoding={:?} ({n_alive} alive)",
                    ks,
                    sample.encoding(),
                );
            }

            print_sample_line(&sample);
        })
        .background()
        .await
        .map_err(|e| anyhow::anyhow!("declare_subscriber: {e}"))?;

    let staleness = topic_stale_after
        .map(|d| format!("{d:?}"))
        .unwrap_or_else(|| "(off)".into());
    println!("(Sub) `{ke}` — discover + `[info]` in parallel; stale watchdog {staleness}");

    tokio::signal::ctrl_c().await?;
    log_alive_keys_roll_up(&alive, true);

    Ok(())
}

async fn subscribe_on_session(
    session: &zenoh::Session,
    discover: bool,
    ke: &str,
    topic_stale_after: Option<Duration>,
) -> anyhow::Result<()> {
    if discover {
        return subscribe_parallel_discover_and_samples(session, ke, topic_stale_after).await;
    }

    session
        .declare_subscriber(ke)
        .callback(move |sample| {
            print_sample_line(&sample);
        })
        .background()
        .await
        .map_err(|e| anyhow::anyhow!("declare_subscriber: {e}"))?;
    println!("(Sub) `{ke}` — waiting…");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

pub async fn run(args: SubscribersArgs) -> anyhow::Result<()> {
    let SubscribersArgs {
        config_path,
        discover,
        keyexpr,
        wait,
        router_connect_override,
        topic_stale_after,
    } = args;

    let ke = keyexpr.trim();
    if ke.is_empty() {
        anyhow::bail!("subscriber keyexpr is empty");
    }

    let mut config = zenoh_config_for_subscriber_from_file(&config_path)?;

    if let Some(ref ep_raw) = router_connect_override {
        let ep = ep_raw.trim();
        if !ep.is_empty() {
            apply_router_connect_override(&mut config, ep)?;
            println!("(sub) connect → `{ep}`");
        }
    }

    let session = zenoh::open(config)
        .await
        .map_err(|e| anyhow::anyhow!("zenoh open: {e}"))?;

    log_zenoh_attachment(&session);

    println!(
        "(sub) ZID {} `{}` {} {:?}",
        session.zid(),
        ke,
        if discover { "discover+info" } else { "samples" },
        wait
    );

    subscribe_on_session(&session, discover, ke, topic_stale_after).await
}

fn log_zenoh_attachment(session: &zenoh::Session) {
    let n_r = session.info().routers_zid().wait().count();
    let n_p = session.info().peers_zid().wait().count();
    println!("(sub) routers={n_r} peers={n_p}");
    if n_r == 0 {
        println!("(sub) no router — check MAIN_SUB_ROUTER / zenohd on :7447 / firewall.");
    }
}

pub fn apply_router_connect_override(
    cfg: &mut zenoh::Config,
    endpoint: &str,
) -> anyhow::Result<()> {
    if endpoint.contains('"') || endpoint.contains('\n') {
        anyhow::bail!("router connect override endpoint contains invalid characters: `{endpoint}`");
    }
    let json = format!(r#"["{endpoint}"]"#);
    cfg.insert_json5("connect/endpoints", &json)
        .map_err(|e| anyhow::anyhow!("connect/endpoints override: {e}"))?;
    cfg.insert_json5("scouting/multicast/enabled", "false")
        .map_err(|e| anyhow::anyhow!("disable multicast (router override): {e}"))?;
    Ok(())
}

fn sub_has_explicit_router_endpoint(cfg: &zenoh::Config) -> bool {
    cfg.get_json("connect/endpoints").map_or(false, |s| {
        let t = s.trim();
        !(t.is_empty() || t == "[]" || t == "null")
    })
}

/// Load Zenoh config and reshape it for a subscriber client.
pub fn zenoh_config_for_subscriber_from_file(
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<zenoh::Config> {
    let cfg_path = path.as_ref();

    let mut c = zenoh::Config::from_file(cfg_path)
        .map_err(|e| anyhow::anyhow!("load Zenoh config {}: {e}", cfg_path.display()))?;

    c.insert_json5("mode", r#""client""#)
        .map_err(|e| anyhow::anyhow!("subscriber reshape (mode client): {e}"))?;

    c.insert_json5("listen/endpoints", "[]")
        .map_err(|e| anyhow::anyhow!("subscriber reshape (clear listen endpoints): {e}"))?;

    // Wait for router before declaring subscribers.
    c.insert_json5("open/return_conditions/connect_scouted", "true")
        .map_err(|e| anyhow::anyhow!("subscriber open.connect_scouted: {e}"))?;
    c.insert_json5("open/return_conditions/declares", "true")
        .map_err(|e| anyhow::anyhow!("subscriber open.declares: {e}"))?;

    if sub_has_explicit_router_endpoint(&c) {
        c.insert_json5("scouting/multicast/enabled", "false")
            .map_err(|e| anyhow::anyhow!("subscriber disable multicast (explicit connect): {e}"))?;
    }

    Ok(c)
}
