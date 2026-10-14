# ⚡ DevDash (`devdash`)

> A beautiful, blazingly fast Terminal Dashboard (TUI) and CLI tool built in Rust to streamline developer workflows. Inspect ports, kill rogue port blockers, manage Docker containers, control background services (`systemctl`), monitor storage disks, and inspect dev processes in real-time.

---

## ⚡ Quick Install

Install the latest precompiled binary in seconds using **curl**:

```bash
curl -fsSL https://raw.githubusercontent.com/Rabin18Glan/devdash/main/install.sh | bash
```

Once installed, simply run:
```bash
devdash
```

*(You can also download standalone tarballs from the [GitHub Releases](https://github.com/Rabin18Glan/devdash/releases) page).*

---

## ✨ Features

- **🔌 [1] Port Manager**:
  - Live discovery of all active TCP & UDP listening ports (`3000`, `5173`, `8080`, `3306`, etc.).
  - Process name, PID, memory consumption, CPU usage, and command line.
  - **Docker Awareness**: Correlates container-mapped ports (e.g. `:3306` labeled `🐳 smart-safety-mysql`).
  - **Browser Launch**: Press `o` on any port to instantly launch `http://localhost:<port>` in your default browser.
  - **One-Key Kill**: Press `x` to terminate the process occupying the port with a safe confirmation dialog (`y` for SIGTERM, `F` for SIGKILL).
  - **Dynamic Sorting**: Sort by Port (`p`), Process Name (`n`), or Memory (`m`).

- **🐳 [2] Docker Container Manager**:
  - Running & stopped containers with state badges, images, names, and port mappings.
  - `Space` / `s`: Start or Stop container.
  - `R`: Restart container.
  - `l` / `Enter`: Stream live container logs in an interactive modal.
  - **Dynamic Sorting**: Sort by State (`s`), Container Name (`n`), or Image (`i`).

- **⚙️ [3] Dev Process Explorer**:
  - Filter dev-relevant processes (Node, Vite, Next.js, Python, Cargo, Rustc, Go, PostgreSQL, Redis, etc.) or toggle all system processes (`a`).
  - **Dynamic Sorting**: Sort by CPU (`c`), Memory (`m`), PID (`p`), or Name (`n`).
  - Terminate hung processes with `x`.

- **🛠️ [4] Background Services Manager (`systemctl`)**:
  - Seamlessly manage background daemons without memorizing long commands!
  - Supports both **User Services** (`systemctl --user`) and **System Services** (`systemctl`), switchable via `u`.
  - **One-Key Controls**:
    - `d`: **Disable and Stop immediately** (`systemctl --user disable --now <service>`) — *ideal for openclaw-gateway, docker, background helpers*.
    - `e`: **Enable and Start immediately** (`systemctl --user enable --now <service>`).
    - `Space` / `s`: Toggle Start / Stop.
    - `R`: Restart service.
    - `l`: View live journal logs (`journalctl -u <service>`).
    - `Enter`: Full `systemctl status` inspector.
  - **Dynamic Sorting**: Sort by Status (`s`) or Name (`n`).

- **💾 [5] Storage & Disk Usage Details**:
  - Complete breakdown of mounted disks, partitions, and filesystems (`/`, `/boot/efi`, `/recovery`, external drives).
  - Visual color-coded usage bars: `████████░░ 76%`.
  - Total capacity, used space, and free space in GB.
  - **Dynamic Sorting**: Sort by Usage % (`u`), Total Size (`t`), or Mount Point (`m`).

- **🔍 Live Search & Filter Everywhere**:
  - Press `/` anywhere to filter ports, containers, processes, services, or storage mounts in real-time.
  - Press `Esc` to clear search or dismiss modals.

- **⚡ Fast Scriptable CLI Subcommands**:
  - `devdash ports`: Print color-coded listening ports table.
  - `devdash kill <port>`: Free up a port in a single command (e.g., `devdash kill 3000`).
  - `devdash services [--system]`: List user or system background services.
  - `devdash disable <service>`: Disable and immediately stop a service (e.g. `devdash disable openclaw-gateway`).
  - `devdash enable <service>`: Enable and immediately start a service.
  - `devdash docker`: Quick Docker container health table.
  - `devdash storage`: Overview of disk space and mount usage.
  - `devdash system`: Instant CPU, memory, swap, and load snapshot.

---

## 🛠️ Alternative Installation Methods

### Install from Crates / Cargo Source:
```bash
cargo install --path .
```

### Build Manually from Source:
```bash
git clone https://github.com/Rabin18Glan/devdash.git
cd devdash
cargo build --release
./target/release/devdash
```

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Description |
| :--- | :--- |
| `Tab` / `1`, `2`, `3`, `4`, `5` | Switch active tab (**Ports**, **Docker**, **Processes**, **Services**, **Storage**) |
| `↑` / `↓` or `j` / `k` | Move cursor up / down (smooth scrolling with scrollbar) |
| `PgUp` / `PgDn` | Scroll by 10 rows (or use your mouse wheel) |
| `Home` / `End` (or `g` / `G`) | Jump to top / bottom |
| `/` | Enter live search & filter mode |
| `s` or `S` | Cycle sort modes in active view |
| `Esc` | Clear search filter or close open modal |
| `r` | Force refresh all metrics and data |
| `?` | Toggle Help cheatsheet |
| `q` or `Ctrl+C` | Quit DevDash |

### Tab-Specific Controls

#### 🔌 Ports Tab
- `o`: Open `http://localhost:<port>` in default browser
- `x`: Kill the process on selected port
- `p` / `n` / `m`: Sort by Port / Name / Memory
- `Enter`: Full process inspector modal

#### 🐳 Docker Tab
- `Space` / `s`: Toggle Start / Stop
- `R`: Restart container
- `l` or `Enter`: View live container logs

#### ⚙️ Processes Tab
- `a`: Toggle Dev-only vs All processes
- `c` / `m` / `p` / `n`: Sort by CPU% / Memory / PID / Name
- `x`: Kill process (SIGTERM or SIGKILL)

#### 🛠️ Services Tab
- `d`: **Disable & stop immediately** (`disable --now`)
- `e`: **Enable & start immediately** (`enable --now`)
- `Space` / `s`: Start / Stop service
- `R`: Restart service
- `l`: View journalctl logs
- `u`: Toggle `--user` services vs System services
- `Enter`: View full `systemctl status`

#### 💾 Storage Tab
- `u`: Sort by Usage %
- `t`: Sort by Total Size
- `m`: Sort by Mount Point
