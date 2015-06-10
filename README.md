# star ![Build Status](https://travis-ci.org/mesosphere/star.svg?branch=master)

## Synopsis

```
   _____ _____ ___  ______
  /  ___|_   _/ _ \ | ___ \
  \ `--.  | |/ /_\ \| |_/ /
   `--. \ | ||  _  ||    /
  /\__/ / | || | | || |\ \
  \____/  \_/\_| |_/\_| \_|

star - test program for network policies

Usage:
    star --help
    star [--http-address=<address>] [--http-port=<port>]
```

## Build with Cargo

```shell
$ cargo build
```

You can run the build result directly from Cargo, too:

```shell
$ cargo run -- --http-address=127.0.0.1 --http-port=9001
```
