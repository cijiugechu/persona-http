# rnet-node

![CI](https://github.com/cijiugechu/rnet-node/workflows/CI/badge.svg)

A blazing-fast HTTP client with TLS fingerprinting for Node.js. Native Rust bindings via napi-rs.

Based on [rnet](https://github.com/0x676e67/rnet) - the original Python implementation.

## Installation

```bash
npm install rnet-node
```

## Usage

### Simple GET request

```typescript
import { get } from 'rnet-node'

const response = await get('https://example.com')
const body = await response.text()
```

### Client with browser emulation

```typescript
import { Client } from 'rnet-node'

const client = new Client({
  emulation: 'chrome_133'
})

const response = await client.get('https://google.com')
const body = await response.text()
```

### Override emulation per request

```typescript
const client = new Client({ emulation: 'chrome_105' })

// Override with different preset
const response = await client.get(url, { emulation: 'chrome_101' })

// Skip client hint headers
const response2 = await client.get(url, { 
  emulation: { preset: 'chrome_105', skipHeaders: true } 
})
```

## Platform Support

| Platform | Architectures | Node.js |
|----------|--------------|---------|
| macOS | x64, arm64 | 20, 22 |
| Windows | x64, x86, arm64 | 20, 22 |
| Linux (glibc) | x64, arm64 | 20, 22 |
| Linux (musl) | x64, arm64 | 20, 22 |
| FreeBSD | x64 | 20, 22 |
| Android | arm64, armv7 | - |

## Development

Requirements:
- Rust (latest stable)
- Node.js 20+
- pnpm

Build and test:
```bash
pnpm install
pnpm build
pnpm test
```

## License

MIT
