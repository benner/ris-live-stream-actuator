# ris-live-stream-actuator
Initial idea is to have some actions based on BGP network changes. Tool expects two tables (due `ipset` limits): `test4` for IPv4 and `test6` for IPv6 networks/prefixes.
To use this we must run this tool as `root` and precreate `test4` and `test6` ipset tables:

```sh
% sudo ipset create test4 hash:net family inet
% sudo ipset create test6 hash:net family inet6
```

To have something in action add acordint `iptables` rule like

```sh
% sudo iptables -I INPUT -m set --match-set test4 dst -j DROP
```


