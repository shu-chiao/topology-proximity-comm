//! Watch for zenohd (remote router) connect and disconnect.
//! Local bridge peers on :7411 are handled in `bridge::spawn_peer_watch`.

use crate::wire::{Watch, format_tag};
use std::time::{Duration, Instant};
use zenoh::config::WhatAmI;

/// How often to poll for routers.
const POLL_INTERVAL: Duration = Duration::from_secs(1);
/// Min interval between repeated "no router yet" logs.
const NO_ROUTER_LOG_COOLDOWN: Duration = Duration::from_secs(5);
const ROUTER_WATCH_STARTUP_GRACE: Duration = Duration::from_secs(5);

/// Remote TCP address for a router, if known.
async fn router_remote_locators(
    session: &zenoh::Session,
    router_zid: &zenoh::config::ZenohId,
) -> Option<String> {
    let transports = session.info().transports().await;
    let mut locators: Vec<String> = Vec::new();
    for transport in transports {
        if transport.whatami() != WhatAmI::Router {
            continue;
        }
        if transport.zid() != router_zid {
            continue;
        }
        let links = session.info().links().transport(transport.clone()).await;
        for link in links {
            locators.push(link.dst().to_string());
        }
    }
    if locators.is_empty() {
        None
    } else {
        locators.sort();
        locators.dedup();
        Some(locators.join(", "))
    }
}

/// Log when zenohd connects or disconnects.
pub fn spawn_router_watch(session: zenoh::Session) {
    tokio::spawn(async move {
        let watch_started = Instant::now();
        let mut interval = tokio::time::interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut had_router = false;
        let mut last_wait_log = Instant::now() - NO_ROUTER_LOG_COOLDOWN;

        loop {
            interval.tick().await;

            let mut routers_it = session.info().routers_zid().await;
            let routers: Vec<_> = std::iter::from_fn(|| routers_it.next()).collect();

            if routers.is_empty() {
                if had_router {
                    eprintln!(
                        "{} All Zenoh routers disconnected.",
                        format_tag(Watch::Router, false)
                    );
                    had_router = false;
                }
                if watch_started.elapsed() >= ROUTER_WATCH_STARTUP_GRACE
                    && last_wait_log.elapsed() >= NO_ROUTER_LOG_COOLDOWN
                {
                    let poll_hz =
                        Duration::from_secs(1).as_secs_f64() / POLL_INTERVAL.as_secs_f64();
                    eprintln!(
                        "{} No Zenoh router yet — polling ~{poll_hz:.0}/s; no Zenoh router in ~{} s (scouting / connect still trying).",
                        format_tag(Watch::Router, false),
                        NO_ROUTER_LOG_COOLDOWN.as_secs(),
                    );
                    last_wait_log = Instant::now();
                }
            } else if !had_router {
                // Collect router lines first so peer-watch logs do not interleave.
                let mut router_lines: Vec<String> = Vec::new();
                for zid in &routers {
                    let line = match router_remote_locators(&session, zid).await {
                        Some(ref s) if !s.is_empty() => format!("  {zid}  ({s})"),
                        _ => format!("  {zid}"),
                    };
                    router_lines.push(line);
                }
                println!(
                    "{} Attached to Zenoh router(s):",
                    format_tag(Watch::Router, true)
                );
                for line in router_lines {
                    println!("{line}");
                }
                had_router = true;
            }
        }
    });
}
