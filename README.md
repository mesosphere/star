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

## REST API

*Peer Status:*

```http
GET / HTTP/1.1
Accept: */*
Accept-Encoding: gzip, deflate
Host: localhost:9000
User-Agent: HTTPie/0.8.0
```

```http
HTTP/1.1 200 OK
Date: Wed, 10 Jun 2015 01:46:01 GMT
Transfer-Encoding: chunked

{
  "status": {
    "peers": [
      {
        "reachable": true,
        "port": 80,
        "host": "1.2.3.4"
      },
      {
        "reachable": false,
        "port": 88,
        "host": "4.3.2.1"
      }
    ]
  }
}
```
