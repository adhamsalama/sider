# Sider

A Multithreaded Redis clone written from scratch in Rust.

## Build

Sider doesn't have any external dependencies.
You can either run it directly:

```
cargo run --release
```

Or you can build it and use -p to specify the port and -t to specify a conenction timeout in milliseconds.

```
cargo build -- release
./target/debug/sider -p 3000 -t 10
```

## Install

Sider is published on crates.io, you can install it using cargo.

```
cargo install sider
```

You can also install it using Docker.

```
docker pull adham99/sider
docker run -p 6379:6379 adham99/sider
```

## Implemented commands (so far):

- SET
- GET
- DEL
- RPUSH
- LRANGE
- INCR
- INCRBY
- DECR
- DECRBY
- EXPIRE
- PUBLISH
- SUBSCRIBE

## Benchmarks

On my machine which has an i5-9300H Intel CPU.

Sider performance:
```
redis-benchmark -n 100000 -c 100 -t set,get

SET: 79365.08 requests per second
GET: 82034.45 requests per second
```
Official Redis performance:
```
redis-benchmark -n 100000 -c 100 -t set,get

SET: 56433.41 requests per second
GET: 57077.62 requests per second
```

Performance may vary depending on the machine you run the benchmarks on.
