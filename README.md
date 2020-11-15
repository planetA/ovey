# Ovey - Userland tools

This repository is a subrepository for the Ovey project/Ovey infrastructure.
It is part of https://github.com/phip1611/tud-grosser-beleg-ss-2020.

It contains tools written in Rust for the following components (binaries):

- **Ovey Coordinator (ovey_coordinator)**
    - knows information about virtual rdma networks
    - can be queried by an ovey daemon
    - offers a REST API for Ovey Daemons
    - is accessible via HTTP (security can be easily added on top; so far only PoC)
    - can handle n virtual networks
    - runs probably once per data center (per rdma network setup)
- **Ovey Daemon (ovey_daemon)**
    - runs once per host
    - has a configured list of coordinators
        - virtual network uuid to IP mapping (Ovey coordinator)
    - is the entity that talks with Ovey kernel module
    - one device creation: checks if for the specified virtual network
      a physical guid exists with the given virtual guid
    - Netlink (OCP) communication with Kernel
        - to-kernel: forward device creation command
        - from-kernel (listens): wait to verify connections (and other stuff if necessary)
    - offers a REST API for Ovey cLI
- **Ovey CLI (ovey_cli)**
    - creates/deletes new ovey devices
    - on creation: guid and virtual network uuid must be specified
      => ovey daemon can look up information
    - communicates with Ovey daemon; not with kernel module
      via REST Interface

## Definition of a virtual network (it's meta data)
- has a UUID (v4)
- knows all virtual guids (=virtual devices) that are allowed in that network
