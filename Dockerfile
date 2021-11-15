FROM rust:1.56 as builder
WORKDIR /app
COPY ./cargo.config /root/.cargo/config
COPY ./ /app
RUN cargo build --release
RUN strip target/release/docs-se

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libc6-dev
COPY --from=builder /app/target/release/docs-se .
COPY --from=builder /app/resources/libsimple.so .
COPY --from=builder /app/resources/dict ./dict
COPY --from=builder /app/resources/pingcap.db ./docs.db
ENV RUST_LOG trace
EXPOSE 3000
ENTRYPOINT ["/docs-se"]
CMD ["server"]
