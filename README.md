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

## Input validation inside Ovey infrastructure and REST communication
- Ovey daemon validates data from cli
- cli validates information by itself too (validates user input)
- coordinator trusts data from daemon (so far because this is a PoC and no production ready software)

## Workflow Device creation
- `$ ovey_cli new ...` -> makes REST-Request to Ovey Daemon
## Workflow Device deletion
- `$ ovey_cli delete ...` -> makes REST-Request to Ovey Daemon
## Workflow Device querying (on current machine)
- `$ ovey_cli list ...` -> makes REST-Request to Ovey Daemon
- Daemon checks all ovey devices that are active on the current machine 
  (TODO via libibverbs or OCP?! probably OCP with kernel; or read from /sys/class/infiniband/)
- checks there state against the coordinator and gives info whether there 
  are problems or not
- advantage of this over `ibv_devinfo` is that Ovey-specific info can be printed
