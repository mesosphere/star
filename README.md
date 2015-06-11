# star ![Build Status](https://travis-ci.org/mesosphere/star.svg?branch=master)

## Synopsis

```
   _____ _____ ___  ______
  /  ___|_   _/ _ \ | ___ \
  \ `--.  | |/ /_\ \| |_/ /
   `--. \ | ||  _  ||    /
  /\__/ / | || | | || |\ \
  \____/  \_/\_| |_/\_| \_|

star-probe - Test program for network policies.

This program periodically attempts to connect to each configured peer URL and
saves state about which ones are reachable.  It provides a REST API for
querying the most recent reachability data for its peer set.

Usage:
    star-probe --help
    star-probe [--http-address=<address>]
         [--http-port=<port>]
         [--http-probe-seconds=<seconds>]
         --peers=<peers>

Options:
    --help                          Show this help message.
    --http-address=<address>        Address to listen on for HTTP requests
                                    [default: 0.0.0.0].
    --http-port=<port>              Port to listen on for HTTP requests
                                    [default: 9000].
    --http-probe-seconds=<seconds>  Seconds between peer connection attempts
                                    [default: 5].
    --peers=<peers>                 List of comma-delimited peer URLs, e.g:
                                    foo.baz.com:80,bar.baz.com:80
```

## REST API

*GET /status*: Peer Status

```http
GET /status HTTP/1.1
Accept: */*
Accept-Encoding: gzip, deflate
Host: localhost:9000
```

```http
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
Date: Wed, 10 Jun 2015 23:11:53 GMT
Transfer-Encoding: chunked

{
    "status": {
        "peers": [
            {
                "reachable": true,
                "url": "bar.baz.com:80"
            },
            {
                "reachable": false,
                "url": "foo.baz.com:80"
            }
        ]
    }
}
```

## Build (with Cargo)

Compile and link:

```shell
$ cargo build
```

You can run the build result directly from Cargo, too:

```shell
$ cargo run --bin star-probe -- --peers=localhost:9000
```

Generate and view the docs:

```shell
$ cargo doc
$ open target/doc/star/index.html
```
