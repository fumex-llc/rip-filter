use crate::{
    args::Args,
    ip::{IpKind, IpMap, IpRangeSet, IpSet, ProxyCheckResponse},
};
use anyhow::Result;
use clap::Parser;
use ipnet::IpNet;
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddrV4},
    str::FromStr,
    sync::Arc,
};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
};

mod args;
mod ip;

async fn dial(dest: SocketAddrV4, stream: &mut TcpStream) {
    match TcpStream::connect(&dest).await {
        Ok(mut dial) => {
            let _ = copy_bidirectional(stream, &mut dial).await;
        }
        Err(e) => {
            eprintln!("Dial failed cause: {e}");
            let _ = stream.set_zero_linger();
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    ip: IpAddr,
    dest: SocketAddrV4,
    ip_set: IpMap,
    incoming_connections: IpSet,
    excluded_range: IpRangeSet,
) {
    for range in excluded_range.iter() {
        if range.contains(&ip) {
            dial(dest, &mut stream).await;
            return;
        }
    }
    match ip_set.get(&ip) {
        Some(record) => {
            if !record.is_allowed() {
                let _ = stream.set_zero_linger();
            } else {
                dial(dest, &mut stream).await;
            }
        }

        _ => {
            incoming_connections.insert(ip);
            dial(dest, &mut stream).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(args.listen)
        .await
        .map_err(|e| eprintln!("Couldn't bind to listen address. Reason -{e}"))
        .unwrap();
    let path = args
        .mount_path
        .unwrap_or("/opt/rip-filter/set.json".to_owned());
    let ip_set = if args.persistent {
        let raw = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| eprintln!("Couldn't to open IP storage file. Reason - {e}"))
            .expect("Couldn't to open IP storage file");
        serde_json::from_str::<HashMap<IpAddr, IpKind>>(&raw)
            .map_err(|e| {
                eprintln!("Serde deserialization error - {e}, IP storage will be ignoring")
            })
            .unwrap_or_default()
    } else {
        HashMap::new()
    };
    let ip_ranges = if let Some(path) = args.exluded_ip {
        let raw = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| eprintln!("Couldn't to open IP Ranges file. Reason - {e}"))
            .expect("Couldn't to open IP Ranges file");
        raw.lines()
            .filter_map(|l| match IpNet::from_str(l) {
                Ok(range) => Some(range),
                _ => {
                    eprintln!("Invalid CIDR format - {l}");
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    let excluded_ip_ranges: IpRangeSet = Arc::new(ip_ranges);
    let ip_set: IpMap = Arc::new(ip_set.into_iter().collect());
    let ip_set_copy = ip_set.clone();
    let ip_set_copy_w = ip_set.clone();
    let incoming_connections: IpSet = Arc::new(HashSet::new().into_iter().collect());
    let incoming_connections_copy = incoming_connections.clone();
    let ip_info_collector: tokio::task::JoinHandle<Result<()>> = tokio::task::spawn(async move {
        let http_client = reqwest::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()?;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(args.poll_period as u64)).await;
            let ips: Vec<_> = incoming_connections_copy.iter().map(|i| *i).collect();
            incoming_connections_copy.clear();
            for ip in ips {
                let response: ProxyCheckResponse = http_client
                    .get(format!(
                        "https://proxycheck.io/v2/{}?key=${}&vpn=1&asn=1",
                        ip, &args.api_key
                    ))
                    .bearer_auth(&args.api_key)
                    .send()
                    .await?
                    .json()
                    .await?;
                ip_set_copy.insert(ip, response.ip_kind());
            }
        }
    });
    let connection_handler: tokio::task::JoinHandle<Result<()>> = tokio::task::spawn(async move {
        loop {
            if let Ok((stream, address)) = listener.accept().await {
                let ip_set = ip_set.clone();
                let incoming_connections = incoming_connections.clone();
                let excluded_ip = excluded_ip_ranges.clone();
                let dest = args.dest;

                tokio::spawn(async move {
                    let _ = handle_connection(
                        stream,
                        address.ip(),
                        dest,
                        ip_set,
                        incoming_connections,
                        excluded_ip,
                    )
                    .await;
                });
            }
        }
    });
    let store_ip_set = if args.persistent {
        tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600_u64)).await;
                let plain_map = ip_set_copy_w
                    .iter()
                    .map(|r| (*r.pair().0, *r.pair().1))
                    .collect::<HashMap<_, _>>();
                let str = serde_json::to_string(&plain_map).unwrap_or_default();
                let _ = tokio::fs::write(format!("{path}.tmp"), &str).await;
                let _ = tokio::fs::rename(format!("{path}.tmp"), &path).await;
            }
        })
    } else {
        tokio::task::spawn(async move {})
    };
    let _ = tokio::join!(connection_handler, ip_info_collector, store_ip_set);
    Ok(())
}
