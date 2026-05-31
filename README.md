# Atlassian handler

Dummy key generator.

> [!WARNING]  
> This implementation is deprecated and not compatible with Atlassian products anymore.


## Requirements

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- Standard `cargo`

## Build

```bash
cargo build --release
```

Ready binary: `target/release/handler`

## Usage

```bash

./handler -m admin@example.com -o "My Company" -p crowd -s ABCD-1234-EFGH-5678

./handler -m admin@example.com -n "Admin" -o "My Company" -p jira -s ABCD-1234-EFGH-5678 -d

./handler -m admin@example.com -n "Admin User" -o "My Org" -p conf -s ABCD-1234-EFGH-5678
```

### Flags

| Flag             | Description                  |
| ---------------- | ---------------------------- |
| `-h`             | Help                         |
| `-m <email>`     | License email (required)     |
| `-n <name>`      | Owner name (default = email) |
| `-o <org>`       | Organization (required)      |
| `-p <product>`   | Product (required)           |
| `-s <server-id>` | Server ID (required)         |
| `-d`             | Data Center license          |

## Supported Products

| Key         | Product                              |
| ----------- | ------------------------------------ |
| `crowd`     | Crowd                                |
| `jira`      | JIRA Software                        |
| `conf`      | Confluence                           |
| `bitbucket` | Bitbucket                            |
| `bamboo`    | Bamboo                               |
| `fisheye`   | FishEye                              |
| `crucible`  | Crucible                             |
| `jsm`       | JIRA Service Management              |
| `jc`        | JIRA Core                            |
| `jsd`       | JIRA Service Desk                    |
| `questions` | Questions plugin for Confluence      |
| `capture`   | Capture plugin for JIRA              |
| `training`  | Training plugin for JIRA             |
| `portfolio` | Portfolio plugin for JIRA            |
| `tc`        | Team Calendars plugin for Confluence |

## Algorithm

According to reverse engineering `.jar` files it works like this: 
1. **License data formation** — creates a key-value set with parameters (dates, contacts, version, type)
2. **LicenseHash calculation** — SHA-256 of sorted properties with escaping of special characters
3. **Text formation** — `#<timestamp>\n<key>=<value>\n...`
4. **Compression** — zlib (Deflate)
5. **Signature** — DSA-1024 with SHA-1
6. **Packaging** — `[4-byte length][data][DSA signature]`
7. **Base64 + suffix** — `base64(data) + "X02" + hex(length)`
8. **Split** — 76 characters per line