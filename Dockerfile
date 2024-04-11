FROM domwst/scrum-poker-ci-image as build

COPY . .

RUN rm /app/Cargo.lock

RUN cargo leptos build --release

FROM debian:bullseye-slim as final

WORKDIR /app

COPY --from=build /build/target/release/scrum-poker scrum-poker
COPY --from=build /build/target/site site

ENV LEPTOS_OUTPUT_NAME="scrum-poker"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_RELOAD_PORT="3001"

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "1234" \
    appuser

RUN chown -R appuser:appuser /app
RUN chmod -R 755 /app

USER appuser

EXPOSE 3000

CMD ["/app/scrum-poker"]
