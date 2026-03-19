# Quickstart: NetNinja Web Interface

## Prerequisites

Same as the existing project: Rust 1.77.2+, Node.js 18+ (for Tailwind CSS build),
and Cargo.

Additionally:
- `cargo-leptos` CLI: `cargo install cargo-leptos`
- Tailwind CSS CLI (for asset compilation): included in the npm dev workflow

---

## First-Time Setup

### 1. Create the Cargo workspace (one-time)

The root `Cargo.toml` (created by this feature) makes `src/backend` and `apps/web`
workspace members. No action needed after the PR merges.

### 2. Install web app dependencies

```bash
cd apps/web
cargo build   # resolves Cargo dependencies
```

### 3. First-run password setup

On the very first run, the web app creates the default admin account (`admin`/`admin`).
No manual setup is required — just start the server and log in.

---

## Development

### Run in dev mode (hot-reload)

```bash
cd apps/web
cargo leptos watch
```

The app starts at `http://localhost:8080` by default.
Leptos hot-reload recompiles server functions and client components on save.

### Run backend (Tauri desktop) alongside web

Both binaries can run simultaneously — they share the same SQLite database.
Start the Tauri app normally in a separate terminal:

```bash
cd src/frontend
npm run tauri:dev
```

---

## Production Build

```bash
cd apps/web
cargo leptos build --release
```

Output binary: `target/release/netninja-web` (Linux) or `target/release/netninja-web.exe` (Windows).

### Run the production binary

```bash
./target/release/netninja-web
```

Or with a custom port:

```bash
NETNINJA_WEB_PORT=9090 ./target/release/netninja-web
```

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `NETNINJA_WEB_PORT` | `8080` | HTTP listen port |
| `NETNINJA_WEB_HOST` | `0.0.0.0` | HTTP listen address |
| `RUST_LOG` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

---

## Admin Password Reset

If you are locked out of the web interface:

1. Navigate to the application data directory:
   - **Windows**: `%ProgramData%\NetNinja\`
   - **Linux**: `~/.local/share/netninja/` (or `$XDG_DATA_HOME/netninja/`)

2. Create a file named **`reset_admin_password.bat`** in that directory.
   Content does not matter — only the file's presence is checked.

   **Windows** (Command Prompt):
   ```cmd
   type nul > "%ProgramData%\NetNinja\reset_admin_password.bat"
   ```

   **Linux**:
   ```bash
   touch ~/.local/share/netninja/reset_admin_password.bat
   ```

3. Restart `netninja-web`.

4. The password is reset to `"admin"`. All active sessions are invalidated.
   Log in at `/login` with `admin` / `admin`.

5. **Change the default password immediately** — the UI will display a warning
   banner until you do.

---

## Validation

After startup, verify the following:

- [ ] `http://localhost:8080/login` renders the login page
- [ ] Login with `admin` / `admin` redirects to `/dashboard`
- [ ] Dashboard shows configured internet lines (if any)
- [ ] Navigating to `/dashboard` without a session redirects to `/login`
- [ ] Logout redirects to `/login` and back-navigation to `/dashboard` re-redirects
- [ ] `cargo check` passes in `src/backend/` with no new errors
- [ ] `npx tsc --noEmit` passes in `src/frontend/` (Tauri desktop unchanged)

---

## Reverse Proxy (Production)

For external access, place a reverse proxy (Nginx, Caddy, Traefik) in front of
`netninja-web` to handle TLS termination.

**Minimal Nginx snippet**:

```nginx
server {
    listen 443 ssl;
    server_name netninja.example.com;

    ssl_certificate     /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

The `Secure` flag on session cookies requires HTTPS to function correctly.
For local-network-only use, HTTP is acceptable and the `Secure` flag can be
disabled via configuration.
