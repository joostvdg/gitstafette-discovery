# gitstafette-discovery

Service discovery service for Gitstafette

## TODO

* implement OTEL tracing
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


## Links

* https://blog.logrocket.com/a-practical-guide-to-async-in-rust/
* https://blog.logrocket.com/understanding-rust-string-str
* https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
* https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html
* https://tokio.rs/tokio/tutorial