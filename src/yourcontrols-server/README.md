# YourControls Server (Linux VPS)

This document explains how to build and run the YourControls server on a Linux VPS, including required environment variables, PM2 setup, and the client app .env changes.

## Prerequisites

- A Linux VPS with a public IPv4 or IPv6 address
- UDP inbound access to the server ports (see Ports section)
- Rust toolchain (stable) and build tools
- Git
- PM2 (Node.js process manager)

## Environment Variables (Server)

The server reads environment variables from a .env file in the working directory (dotenv).

Required variables:

- SERVER_HOSTNAME: Public hostname clients use to reach the rendezvous server
- SERVER_PORT: UDP port for rendezvous server
- HOSTER_PORT: UDP port for hoster server
- MINIMUM_VERSION: Minimum client version allowed to connect (semver, e.g. 2.7.5)
- MAX_CLIENT_CONNECTIONS: Maximum concurrent clients for hosted sessions
- HOSTER_IP: Fallback IP for hoster resolution (used if DNS lookup fails)

Example .env for the VPS:

```bash
SERVER_HOSTNAME=your.domain.example
SERVER_PORT=27015
HOSTER_PORT=27016
MINIMUM_VERSION=2.7.5
MAX_CLIENT_CONNECTIONS=100
HOSTER_IP=203.0.113.10 <-- replace with your VPS public IP
```

## Build Instructions

From the repo root:

1. Install Rust (if needed):
   curl --proto '=https' --tlsv1.2 -sSf <https://sh.rustup.rs> | sh
2. Build the server:
   cargo build -p yourcontrols-server --release

Binary path after build:

- target/release/yourcontrols-server

## Run as a Daemon with PM2

1. Install Node.js and PM2:
   curl -fsSL <https://deb.nodesource.com/setup_20.x> | sudo -E bash -
   sudo apt-get install -y nodejs
   sudo npm install -g pm2

2. From the repo root (where .env lives), start the server:
   pm2 start ./target/release/yourcontrols-server --name yourcontrols-server --time

3. Save and enable startup on boot:
   pm2 save
   pm2 startup

Notes:

- PM2 uses the current working directory, so ensure the .env file is in the same directory where you run pm2 start.
- Use pm2 logs yourcontrols-server to view logs.

## Main Application .env Changes

The client app uses SERVER_HOSTNAME and SERVER_PORT to locate the rendezvous server. Update the main app .env in the repo root (create it from .env.example if needed):

SERVER_HOSTNAME=your.domain.example
SERVER_PORT=27015

Because the client uses compile-time dotenv for these values, rebuild the main application after changing the .env:

cargo build -p yourcontrols --release

## Ports / Firewall

Open UDP ports on the VPS:

- SERVER_PORT (rendezvous)
- HOSTER_PORT (hoster)

## Troubleshooting

- Connection denied due to version: check MINIMUM_VERSION
- Clients cannot resolve hoster: confirm SERVER_HOSTNAME DNS and HOSTER_IP fallback
- No connections: verify UDP firewall rules and that PM2 is running the process
