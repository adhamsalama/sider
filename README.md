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

## Benchmarks

On my machine which has an i5-9300H Intel CPU.

```
redis-benchmark -n 100000 -c 100 -t set,get

SET: 42844.90 requests per second
GET: 43840.42 requests per second
```

```
redis-benchmark -n 500000 -c 1000 -t set,get

SET: 40041.64 requests per second
GET: 40650.41 requests per second
```

In comparsion to Redis (also on my machine):

```
redis-benchmark -n 100000 -c 100 -t set,get

SET: 34246.57 requests per second
GET: 34364.26 requests per second
```

```
redis-benchmark -n 500000 -c 1000 -t set,get

SET: 31527.84 requests per second
GET: 32032.80 requests per second
```

Performance may vary depending on the machine you run the benchmarks on.
