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
