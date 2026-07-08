# Changelog

All notable changes to ru-share will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [1.1.0] - 2026-07-05

### Added
- **Multi-origin CORS support**: `RU_SHARE_CORS_ORIGIN` now accepts a comma-separated list of origins (e.g., `http://localhost,https://yourname.playit.gg`). This enables accessing ru-share from both local network and external tunnels simultaneously.

### Changed
- CORS layer now uses `AllowOrigin::list()` instead of `AllowOrigin::exact()`

### Documentation
- Added `install.sh` for system-wide installation with systemd service
- Added `start.sh` for quick local one-shot startup

## [1.0.6] - 2026-07-04

### Security Fixes
- **Critical**: Fixed unbounded rate-limiter memory exhaustion (HashMap → LRU cache with 4096 entries)
- **High**: Fixed public profile endpoint leaking sensitive data (is_admin, quota_bytes, used_bytes, last_login)
- **High**: Strengthened Argon2 password hashing (64MB memory cost, 2 iterations)
- **High**: Added session invalidation on password change
- **Medium**: Added 500-item limit to bulk delete operations
- **Medium**: Added username validation (max 64 chars, alphanumeric/dash/underscore only)
- **Medium**: Extended rate limiting to file upload and admin endpoints
- **Low**: Clamped audit log limit to max 500 results
- **Low**: Reduced session cleanup interval from 1 hour to 5 minutes

### Added
- `created_at` timestamp for user accounts
- `PublicProfile` DTO for safe public profile data

## [1.0.0] - Initial Release

### Added
- Client-side AES-GCM-256 encryption via Web Crypto API
- Zero-knowledge share links with URL hash fragment key delivery
- Single binary deployment with embedded static assets
- SQLite WAL mode for MicroSD performance
- Argon2id password hashing (SIMD disabled for ARMv6)
- Token bucket rate limiting (60 req/min per IP on auth endpoints)
- Admin panel with user/quota/extension management
- Session-based authentication with 1-week expiry
- Pre-flight quota checking on upload
- Banned extension filtering for uploads
- Dashboard with drag-drop upload and storage progress
- Cross-compilation support via `cross`:
  - `arm-unknown-linux-gnueabi` (ARMv6)
  - `armv7-unknown-linux-gnueabihf` (ARMv7)
  - `aarch64-unknown-linux-gnu` (ARM64)
  - `x86_64-unknown-linux-gnu` (x86_64)
- GitHub Actions CI/CD for multi-target builds
