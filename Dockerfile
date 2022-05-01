FROM rust:1.60 as builder
WORKDIR /app
COPY ./ /app
RUN cargo build --release
RUN strip target/release/docs-se

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libc6-dev
RUN mkdir -p /search
COPY --from=builder /app/target/release/docs-se /search/se
COPY --from=builder /app/libsimple.so ./search/
COPY --from=builder /app/dict ./search/dict
COPY --from=builder /app/docs.db ./docs.db
RUN rm -rf /app
ENV RUST_LOG trace
EXPOSE 3030
ENTRYPOINT ["/search/se"]
CMD ["server"]
