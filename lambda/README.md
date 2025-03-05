# Setup

```bash
cargo install cargo-lambda
```

# Development

```bash
source ../.env     # `source ../.env.fish` if using fish
cargo lambda watch --env-file ../.env
```

```bash
caddy run --config Caddyfile
```

Accessible on [localhost:8080](http://localhost:8080).