# tinyfw

## Introduction
**tinyfw** is a toy firewall built in Rust and leveraging the [eBPF](https://ebpf.io/what-is-ebpf) technology. The network policies can be attached to any supported network interface. It is also containers aware, the [containerd](https://containerd.io/) runtime is supported.

**IMPORTANT**: at the moment the network policy is hardcoded, it simply drops all the TCP traffic directed to the following ports: **80, 20, 21,23,25, 110,143,139,445,1521,161,162, 70**. If you want to change it, take a look [here](https://github.com/angelopoerio/tinyfw/blob/main/tinyfw-ebpf/src/main.rs).


## Prerequisites

1. Install a rust stable toolchain: `rustup install stable`
1. Install a rust nightly toolchain: `rustup install nightly`
1. Install bpf-linker: `cargo install bpf-linker`


## Build eBPF

```bash
cargo xtask build-ebpf
```

To perform a release build you can use the `--release` flag.
You may also change the target architecture with the `--target` flag


## Build Userspace

```bash
cargo build
```

To perform a release build you can use the `--release` flag.


## Run

```bash
cargo xtask run
```


## How to use
The tool can be used in two ways:
* attach to a user specified network interface. Flag is **--iface**
* **containers mode**: it will try to connect to the unix socket of the local running **containerd** daemon listening for new spawned containers, so that XDP/eBPF rules can be attached on demand. You can use the flag **--cmode** for this.


## Run on Kubernetes
It is possible to run the tool as a Kubernetes [daemonset](https://kubernetes.io/docs/concepts/workloads/controllers/daemonset/). In order for it to work the following conditions must be met:
* Run as a privileged daemonset. This is required to list the network interfaces on the host and attach ebpf code to them. Learn more [here](https://kubernetes.io/docs/tasks/configure-pod-container/security-context/)
* Mount the containerd unix socket as an **hostpath**. Learn more [here](https://kubernetes.io/docs/concepts/storage/volumes/#hostpath).


# Troubleshooting
A useful tool to trobleshoot **tinyfw** (and any ebpf based networking tool) is bpftool. For example to inspect the ebpf programs attached to the [veth](https://developers.redhat.com/blog/2018/10/22/introduction-to-linux-interfaces-for-virtual-networking#iveth) interfaces (the network interfaces used by the containers) of an host you can type the following command:
```bash

bpftool net show
xdp:
veth7c2e2820(19) driver id 295
veth183ae1ae(23) driver id 295
veth8c690d33(24) driver id 295
vethff389ec(31) driver id 295

tc:

flow_dissector:
```
**IMPORTANT**: Remember to set the env variable **RUST_LOG=info** before running tinyfw to have runtime informations of what it's going on!

## Why support only containerd?
Containerd is becoming the standard de facto for containers runtime. [AWS EKS](https://docs.aws.amazon.com/eks/latest/userguide/dockershim-deprecation.html) is for example moving to it as the default runtime.


## Misc & Links
The author of this project is **Angelo Poerio <angelo.poerio@gmail.com>**

**IMPORTANT**: THIS TOOL IS NOT **PRODUCTION READY**. Use it at your own risks!

Useful links:
* [Aya](https://aya-rs.dev/)
* [eBPF](https://ebpf.io/)

TODO:
* Better way to correlate veth(s) interfaces to containers
* Make network policies configurable and not hardcoded
