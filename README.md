# ris-live-stream-actuator

[![Build Status](https://travis-ci.com/travis-ci/travis-web.svg?branch=master)](https://travis-ci.com/travis-ci/travis-web)
![Coveralls github branch](https://img.shields.io/coveralls/github/benner/ris-live-stream-actuator/master)

Initial idea is to have some actions based on BGP network changes. Tool expects two tables (due `ipset` limits): `ris-ipv4` for IPv4 and `ris-ipv6` for IPv6 networks/prefixes.
To use this we must run this tool as `root` and precreate `ris-ipv4` and `ris-ipv6` ipset tables:

```sh
% sudo ipset create ris-ipv4 hash:net family inet
% sudo ipset create ris-ipv6 hash:net family inet6
```

To have something in action add acordint `iptables` rule like

```sh
% sudo iptables -I INPUT -m set --match-set ris-ipv4 dst -j DROP
```


