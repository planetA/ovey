# Ovey - Userland tools

This repository is a subrepository for the Ovey project/Ovey infrastructure.
It is part of https://github.com/phip1611/tud-grosser-beleg-ss-2020.

It contains tools written in Rust for the following components (binaries):

- **Ovey Coordinator (ovey_coordinator)**
    - knows information about virtual rdma networks
    - can be queried by an ovey daemon
    - uses a simple REST interface 
    - is accessible via HTTP (security can be easily added on top; so far only PoC)
    - can handle zero to n virtual networks
    - runs probably once per data center (per rdma network setup)
- **Ovey Daemon (ovey_daemon)**
    - runs once per host
    - has a configured list of coordinators
        - virtual network uuid to IP mapping (Ovey coordinator)
    - is the entity that talks with Ovey kernel module
    - one device creation: checks if for the specified virtual network
      a physical guid exists with the given virtual guid
    - waits via netlink for requests from kernel (? necesarry? TODO)
    - waits via (??? netlink or REST?) for requests from Ovey CLI
- **Ovey CLI (ovey_cli)**
    - creates/deletes new ovey devices
    - on creation: guid and virtual network uuid must be specified
      => ovey daemon can look up information
    - communicates with Ovey daemon; not with kernel module

## Definition of a virtual network (it's meta data)
- has a UUID (v4)
- knows n virtual guid to physical guid mappings
- knows ... (???) TODO probably QP number and other information
- perhaps also knows per connection information and not only
  per device information.. because of QP number
