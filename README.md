# Traceroute
Network diagnostic tool to capture route and transit delay of packets.

My motivation for building this was to explore oddities in internet routing.
Specifically providers which send traffic through multiple routes for the same
destination, unoptimal routing, and bad performance routers along the path.

# Usage

For Nix users
```
# jump into development shell with tools and environment setup
nix develop

# for a typical traceroute experience
cargo run -- yahoo.com

# for a graph view
cargo run -- --graph yahoo.com | xdot -
```

For general linux users
```
# command needs to sudo to receive ICMP packets
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER = "sudo -E";

# for a typical traceroute experience
cargo run -- yahoo.com

# for a graph view
cargo run -- --graph yahoo.com | xdot -
```

## Goals
Use all known methods to discover the route/s to be taken by a packet to a target
Deduce latency of hops and target.
Use collected data to build directed graph of perceived network.

## Related Projects
### [dublin traceroute](https://github.com/insomniacslk/dublin-traceroute) [traceroute] [c++] [go]
Dublin Traceroute is a NAT-aware multipath traceroute

### [traceroute-rs](https://github.com/daniellockyer/traceroute-rs) [traceroute] [rust]
Traceroute implemented in ~165 lines of rust

### [fastping-rs](https://github.com/bparli/fastping-rs) [ping] [rust]
fastping-rs is a Rust ICMP ping library, inspired by [go-fastping](https://github.com/tatsushid/go-fastping)  and the [AnyEvent::FastPing Perl module](http://search.cpan.org/~mlehmann/AnyEvent-FastPing-2.01/), for quickly sending and measuring batches of ICMP ECHO REQUEST packets.

### [libtraceroute](https://github.com/ilyagrishkov/libtraceroute) [traceroute] [rust]

### [ping.rs](https://gist.github.com/nixpulvis/e2938d03d141990d99db) [ping] [rust]
A simple Ping implementation in Rust. 83 lines of rust

# Example Output
```
cargo run -- --graph yahoo.com 
```

```mermaid
graph TB
0([""10.16.1.1""])
1([""10.16.1.11""])
2([""Hidden""])
3([""173.219.221.188""])
4([""173.219.224.90""])
5([""173.219.201.108""])
6([""208.115.136.16""])
7([""209.191.68.3""])
8([""209.191.64.236""])
9([""66.196.67.101""])
10([""67.195.4.99""])
11([""68.180.235.4""])
12([""67.195.34.71""])
13([""98.137.11.163""])
14([""66.196.67.127""])
15([""98.136.158.167""])
16([""68.180.235.8""])
17([""98.137.11.164""])
18([""209.191.64.73""])
19([""74.6.227.143""])
20([""74.6.122.45""])
21([""74.6.123.243""])
22([""74.6.98.138""])
23([""74.6.143.25""])
24([""209.191.65.133""])
25([""209.191.64.167""])
26([""74.6.227.145""])
27([""74.6.98.227""])
28([""74.6.123.237""])
29([""74.6.143.26""])
30([""209.191.64.212""])
31([""209.191.65.115""])
32([""98.138.97.73""])
33([""98.138.51.6""])
34([""98.138.97.156""])
35([""74.6.231.21""])
36([""216.115.105.25""])
37([""98.138.2.13""])
38([""98.138.51.0""])
39([""98.138.97.157""])
40([""74.6.231.20""])
1 --> 0
0 --> 2
2 --> 3
3 --> 4
4 --> 5
5 --> 6
6 --> 7
7 --> 8
8 --> 9
9 --> 10
10 --> 11
11 --> 12
12 --> 13
8 --> 14
14 --> 15
15 --> 16
16 --> 12
12 --> 17
6 --> 18
18 --> 19
19 --> 20
20 --> 21
21 --> 22
22 --> 23
6 --> 24
24 --> 25
25 --> 26
26 --> 27
27 --> 28
28 --> 22
22 --> 29
6 --> 30
30 --> 31
31 --> 32
32 --> 33
33 --> 34
34 --> 35
30 --> 36
36 --> 37
37 --> 38
38 --> 39
39 --> 40
```
