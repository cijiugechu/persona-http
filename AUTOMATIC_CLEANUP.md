# Automatic Response Cleanup

## Overview

The `ResponseHandle` now implements automatic cleanup similar to undici and the Fetch API. Users no longer need to manually call `response.close()` - the response body will be automatically cleaned up when the JavaScript object is garbage collected.

## How It Works

### 1. **Drop Trait Implementation**

When the JavaScript `ResponseHandle` object is garbage collected, NAPI-RS automatically calls the Rust `Drop` trait's `drop()` method:

```rust
impl Drop for ResponseHandle {
  fn drop(&mut self) {
    if !self.consumed.load(Ordering::Acquire) {
      self.inner.close();
    }
  }
}
```

### 2. **Consumption Tracking**

The implementation tracks whether the response body has been consumed:

- **Consumed**: Body was read via `text()`, `json()`, or `bytes()` - no cleanup needed
- **Not Consumed**: Body was never read - automatically close to prevent resource leaks

### 3. **Similar to Undici/Fetch**

This mirrors how undici implements automatic cleanup using JavaScript's `FinalizationRegistry`:

**Undici's approach (JavaScript side):**

```javascript
const streamRegistry = new FinalizationRegistry((weakRef) => {
  const stream = weakRef.deref()
  if (stream && !stream.locked && !isDisturbed(stream)) {
    stream.cancel('Response object has been garbage collected').catch(noop)
  }
})
```

**Our approach (Rust/NAPI side):**

```rust
// NAPI-RS automatically calls Drop when JS object is GC'd
impl Drop for ResponseHandle {
  fn drop(&mut self) {
    if !self.consumed.load(Ordering::Acquire) {
      self.inner.close();
    }
  }
}
```

## Usage Examples

### Before: Manual Cleanup Required ❌

```javascript
const response = await client.get('https://api.example.com/data')
console.log(response.status)
// Oops! Forgot to call response.close() - resource leak!
```

### After: Automatic Cleanup ✅

```javascript
// Option 1: Consume the body - automatically cleaned up
const response = await client.get('https://api.example.com/data')
const data = await response.json()
// No need to call close() - body was consumed

// Option 2: Don't consume the body - still automatically cleaned up
const response = await client.get('https://api.example.com/data')
console.log(response.status, response.headers)
// When 'response' goes out of scope and is GC'd,
// the Drop trait automatically closes it

// Option 3: Explicit close still works for immediate cleanup
const response = await client.get('https://api.example.com/data')
console.log(response.status)
response.close() // Optional - immediate cleanup
```

## Benefits

1. **Prevents Resource Leaks**: No more forgetting to close responses
2. **Better Developer Experience**: Matches behavior of undici and Fetch API
3. **Backward Compatible**: The `close()` method still exists for explicit cleanup
4. **Safe**: Tracks consumption to avoid double-free issues

## Implementation Details

### Consumption State Tracking

```rust
pub struct ResponseHandle {
  inner: Arc<Response>,
  consumed: Arc<AtomicBool>,  // Tracks if body was consumed
}
```

The `consumed` flag is set to `true` when:

- `text()` is called and successfully returns
- `json()` is called and successfully returns
- `bytes()` is called and successfully returns
- `close()` is explicitly called

### Thread Safety

Uses `AtomicBool` with `Ordering::Acquire`/`Release` for thread-safe consumption tracking across async contexts.

## Why Manual Close Was Previously Needed

In NAPI implementations, Rust resources don't automatically clean up when JavaScript objects are garbage collected unless:

1. **Drop trait is implemented** - We now do this ✅
2. **NAPI finalizers are used** - Drop trait is the idiomatic Rust way ✅

Without these, the underlying HTTP connection and body stream would remain open, causing:

- Connection pool exhaustion
- Memory leaks
- File descriptor leaks

## Comparison with Other Libraries

| Library            | Cleanup Method            | Manual Close Required |
| ------------------ | ------------------------- | --------------------- |
| **undici**         | FinalizationRegistry      | No ❌                 |
| **node-fetch**     | FinalizationRegistry      | No ❌                 |
| **Fetch API**      | Browser GC + Stream locks | No ❌                 |
| **nitai (before)** | Manual `close()`          | Yes ⚠️                |
| **nitai (now)**    | Drop trait                | No ✅                 |

## Migration Guide

### Removing Explicit `close()` Calls

If you have existing code with `close()` calls, they can be safely removed:

```javascript
// Old code - still works but not necessary
const response = await client.get(url)
const data = await response.json()
response.close() // This is now optional

// New code - simpler
const response = await client.get(url)
const data = await response.json()
// That's it! Automatic cleanup when response is GC'd
```

### When to Still Use `close()`

You might still want to use `close()` explicitly in these scenarios:

1. **Long-running processes**: Force cleanup before GC runs
2. **High request volume**: Free resources immediately
3. **Explicit resource management**: Make cleanup timing predictable

```javascript
// High-volume scenario
for (let i = 0; i < 10000; i++) {
  const response = await client.get(urls[i])
  console.log(response.status)
  response.close() // Explicit cleanup to avoid accumulation
}
```

## Performance Considerations

- **No performance overhead**: The Drop trait has zero runtime cost
- **Memory efficient**: Only adds one `AtomicBool` (1 byte) per response
- **GC-friendly**: Works with JavaScript's garbage collection naturally
- **Connection pooling**: Properly returns connections to the pool on cleanup

## Testing Automatic Cleanup

To verify automatic cleanup works:

```javascript
// Create and discard responses without calling close()
async function testAutoCleanup() {
  for (let i = 0; i < 1000; i++) {
    const response = await client.get('https://httpbin.org/get')
    console.log(response.status)
    // No close() - let GC handle it
  }

  // Force garbage collection (in Node.js with --expose-gc)
  if (global.gc) {
    global.gc()
  }

  // Wait a bit for cleanup to happen
  await new Promise((resolve) => setTimeout(resolve, 1000))

  // Check connection pool - should not be exhausted
  const response = await client.get('https://httpbin.org/get')
  console.log('Still works:', response.status)
}
```

## Related Resources

- [Node.js N-API Documentation](https://nodejs.org/api/n-api.html)
- [NAPI-RS Documentation](https://napi.rs/)
- [Undici Response Implementation](https://github.com/nodejs/undici/blob/main/lib/web/fetch/response.js)
- [Rust Drop Trait](https://doc.rust-lang.org/std/ops/trait.Drop.html)
- [JavaScript FinalizationRegistry](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/FinalizationRegistry)
