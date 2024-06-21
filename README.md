# Shadow RAT

[![Rust](https://github.com/vu1nerab13/shadow/actions/workflows/build.yml/badge.svg)](https://github.com/vu1nerab13/shadow/actions/workflows/build.yml)
*A high performance rat server and client written in 100% safe rust*

# Features
1. Restful api interface
2. Socks5 proxy
3. High performance
4. Stability
5. etc.

# Build (WSL & macOS)

Install rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone repository
```bash
git clone https://github.com/vu1nerab13/shadow.git
```

Build
```bash
cd shadow
cargo build
```

Run
```bash
cargo run
```

# Build (Windows with MSVC)

Install chocolatey
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```

Install build tools
```powershell
choco install cmake ninja rust-ms llvm visualstudio2022buildtools
```

Clone repository
```powershell
git clone https://github.com/vu1nerab13/shadow.git
```

Build
```powershell
cd shadow
cargo build
```

Run
```bash
cargo run
```

# Web API Docs
See `README.md` in shadow-server/src/web/

# License

Shadow is distributed under the MIT License.
