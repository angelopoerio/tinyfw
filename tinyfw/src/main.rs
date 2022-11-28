use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Bpf};
use clap::Parser;
use client::services::v1::containers_client::ContainersClient;
use client::services::v1::events_client::EventsClient;
use client::services::v1::ListContainersRequest;
use client::services::v1::SubscribeRequest;
use client::with_namespace;
use containerd_client as client;
use containerd_client::tonic::Request;
use log::info;
use network_interface::NetworkInterface;
use network_interface::NetworkInterfaceConfig;
use tokio::signal;

use std::collections::HashMap;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "lo")]
    iface: String,

    #[clap(short, long)]
    cmode: bool,

    #[clap(short, long, default_value = "/run/containerd/containerd.sock")]
    socket_containerd: String,
}

fn attach_xdp_probes_to_containers(
    program: &mut Xdp,
    already_attached_ifaces: &mut HashMap<String, bool>,
) {
    let network_interfaces = NetworkInterface::show().unwrap();

    for itf in network_interfaces.iter() {
        if itf.name.starts_with("veth") {
            match already_attached_ifaces.get(&itf.name) {
                Some(_) => continue,
                _ => (),
            }

            info!("Trying to attach the xdp probe to {}", itf.name);
            match program
                .attach(&itf.name, XdpFlags::default())
                .context(format!(
                    "failed to attach the XDP program to iface {}",
                    itf.name
                )) {
                Ok(_) => {
                    already_attached_ifaces.insert(itf.name.clone(), true);
                    info!("Remembering the iface {}", itf.name);
                }
                Err(err) => info!("{}", err),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    env_logger::init();

    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/tinyfw"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/tinyfw"
    ))?;
    let mut program: &mut Xdp = bpf.program_mut("tinyfw").unwrap().try_into()?;
    program.load()?;

    if opt.cmode {
        // TODO: replace with an LRU cache
        let mut already_attached_ifaces: HashMap<String, bool> = HashMap::new();

        let channel = client::connect(opt.socket_containerd)
            .await
            .expect("Connect Failed");

        /* List running containers */
        let mut containers_client = ContainersClient::new(channel.clone());
        let list_req = ListContainersRequest { filters: vec![] };
        let req_list = with_namespace!(list_req, "default");

        let _resp = containers_client
            .list(req_list)
            .await
            .expect("Failed to list containers");
        let message = _resp.get_ref();

        if message.containers.len() > 0 {
            info!("Found containers at start. Attaching XDP probes");
            attach_xdp_probes_to_containers(&mut program, &mut already_attached_ifaces);
        }

        /* Start listening to containers creation events */
        info!("Listening for spawned containers by containerd");
        let mut client = EventsClient::new(channel);

        /* how to filter only for containers creation events? */
        let req = SubscribeRequest { filters: vec![] };

        let mut resp = client.subscribe(req).await?;
        let stream = resp.get_mut();

        while let Some(event) = stream.message().await? {
            if event.topic == "/tasks/start" {
                attach_xdp_probes_to_containers(&mut program, &mut already_attached_ifaces);
            }
        }
    } else {
        info!("Attaching XDP probe to {}", &opt.iface);
        program
            .attach(&opt.iface, XdpFlags::default())
            .context(format!(
                "failed to attach the XDP program to iface {}",
                &opt.iface
            ))?;
    }

    info!("Waiting for termination signals");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
