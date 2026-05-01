# unbill-server

`unbill-server` is a standalone HTTPS-capable HTTP server that implements the REST API consumed by `HttpStore`. It is the remote persistence backend for a single device: one device key namespace, one set of ledgers.

## Purpose

Provides a hosted storage backend so that a device using `HttpStore` can persist ledger snapshots and device metadata on a remote machine instead of local disk. A single running instance serves one authenticated client identified by a static API key.

## API

All endpoints require `Authorization: Bearer <api_key>`. Requests with a missing or wrong key receive `401 Unauthorized`.

| Method | Path | Request body | Success |
|----------|----------------------------|--------------------------------|----------------|
| `GET` | `/ledgers` | — | 200 JSON array |
| `PUT` | `/ledgers/:id/meta` | JSON `LedgerMeta` | 204 |
| `GET` | `/ledgers/:id/snapshot` | — | 200 bytes / 404|
| `PUT` | `/ledgers/:id/snapshot` | `application/octet-stream` | 204 |
| `DELETE` | `/ledgers/:id` | — | 204 (idempotent)|
| `GET` | `/device/:key` | — | 200 bytes / 404|
| `PUT` | `/device/:key` | `application/octet-stream` | 204 |

Device key names must consist solely of alphanumeric characters, hyphens, underscores, and dots. Any other key is rejected with `400 Bad Request`.

## Configuration

All configuration is read from environment variables at startup. The server exits immediately if a required variable is absent.

| Variable | Required | Default | Description |
|------------|----------|---------|------------------------------------|
| `API_KEY` | yes | — | Bearer token clients must supply |
| `DATA_DIR` | no | `./data`| Root directory handed to `FsStore` |
| `PORT` | no | `8080` | TCP port to listen on |

## Boundaries

- One API key, one device namespace. Multi-tenancy is outside scope.
- TLS termination is expected to happen at a reverse proxy; the server itself speaks plain HTTP.
- The server does not perform ledger-level access control beyond the single API key.
