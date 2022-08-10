# Traceroute
Network diagnostic tool to capture route and transit delay of packets.

# How to run
```
cargo build && sudo ./target/debug/main  -n 1 eff.org | xdot -
```

## Goals
Use all known methods to discover the route/s to be taken by a packet to a target
Deduce latency of hops and target.
Use collected data to build directed graph of percieved network.

## Non-Goals
- TBD

## Known Related Projects
### [dublin traceroute](https://github.com/insomniacslk/dublin-traceroute) [traceroute] [c++] [go]
Dublin Traceroute is a NAT-aware multipath traceroute

### [traceroute-rs](https://github.com/daniellockyer/traceroute-rs) [traceroute] [rust]
Traceroute implemented in ~165 lines of rust

### [fastping-rs](https://github.com/bparli/fastping-rs) [ping] [rust]
fastping-rs is a Rust ICMP ping library, inspired by [go-fastping](https://github.com/tatsushid/go-fastping)  and the [AnyEvent::FastPing Perl module](http://search.cpan.org/~mlehmann/AnyEvent-FastPing-2.01/), for quickly sending and measuring batches of ICMP ECHO REQUEST packets.

### [trcrt](https://github.com/zeroed/trcrt) [traceroute] [rust]
Toy repo

### [libtraceroute](https://github.com/ilyagrishkov/libtraceroute) [traceroute] [rust]

### [ping.rs](https://gist.github.com/nixpulvis/e2938d03d141990d99db) [ping] [rust]
A simple Ping implementation in Rust. 83 lines of rust

