# gitstafette-discovery

Service discovery service for Gitstafette

## TODO

* implement OTEL tracing
  * add tracing to client
  * do we get a proper distributed trace?
  * configure it via environment variables
* update container images
  * server
  * client (for Sidecar)
* create Helm package
  * for Discovery server
* support TLS with custom CA
* add as a sidecar to 
  * Gitstafette server
  * Gitstafette relay
* support authentication
  * OAUTH token with GRPC?


## Autometrics Dashboard

```shell
brew install autometrics-dev/tap/am
```

```shell
am start :8080
```

## OTEL Tracing

* https://tokio-rs.github.io/tracing/tracing/
* https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs
* https://broch.tech/posts/rust-tracing-opentelemetry/
* https://github.com/tekul/rust-tracing-otlp/blob/main/Cargo.toml
* https://github.com/autometrics-dev/autometrics-rs/blob/main/examples/exemplars-tracing-opentelemetry/src/main.rs
* https://www.aspecto.io/blog/distributed-tracing-with-opentelemetry-rust/
* https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples/tracing-grpc

```shell
OpenTelemetry trace error occurred. Exporter otlp encountered the following error(s): the grpc server returns error (Unknown error): , detailed error message: h2 protocol error: http2 error: connection error detected: frame with invalid size
```

## Links

* https://blog.logrocket.com/a-practical-guide-to-async-in-rust/
* https://blog.logrocket.com/understanding-rust-string-str
* https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
* https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html
* https://tokio.rs/tokio/tutorial
* https://autometrics.dev/blog/adding-observability-to-rust-grpc-services-using-tonic-and-autometrics
* https://docs.autometrics.dev/rust/quickstart