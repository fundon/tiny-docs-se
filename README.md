# docs-se

Generates indexes from markdown files.

## Required

* HTTP2 and TLS
* sqlite3
* [libsimple](https://github.com/wangfenjin/simple)

## Commands

* `cat sample.sql | sqlite3 docs.db`

* `tiny-docs-se build --help`

```shell
tiny-docs-se 0.1.0
Build sqlite db indexes from markdown files

USAGE:
    tiny-docs-se build --path <PATH> --locale <LOCALE> --version <VERSION>

OPTIONS:
    -h, --help                 Print help information
    -l, --locale <LOCALE>      locale
    -p, --path <PATH>
    -v, --version <VERSION>    version
```

* `tiny-docs-se server --help`

```shell
tiny-docs-se 0.1.0
Run a search server for web

USAGE:
    tiny-docs-se server [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -p, --port <PORT>    port [default: 3030]
    -V, --version        Print version information
```

```shell
curl -X POST 127.0.0.1:3030 -H "Content-Type: application/json" -d '{"search": "rust", "locale": "cn", "version": "v1.0"}'
```

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
