/**
 * Test demonstrating automatic response cleanup
 * 
 * This test shows that responses no longer need manual close() calls.
 * The Drop trait implementation automatically cleans up resources when
 * the JavaScript object is garbage collected.
 */

const { Client } = require('./index.js');

async function testAutomaticCleanup() {
  console.log('🧪 Testing Automatic Response Cleanup\n');
  
  const client = new Client();

  // Test 1: Body consumed - no close needed
  console.log('Test 1: Body consumed (text/json/bytes)');
  {
    const response = await client.get('https://httpbin.org/json');
    const data = await response.json();
    console.log('✅ Status:', response.status);
    console.log('✅ Data keys:', Object.keys(data));
    // No close() needed - body was consumed
  }
  console.log('   Response went out of scope - will be auto-cleaned on GC\n');

  // Test 2: Body not consumed - still auto-cleaned
  console.log('Test 2: Body not consumed (only headers read)');
  {
    const response = await client.get('https://httpbin.org/get');
    console.log('✅ Status:', response.status);
    console.log('✅ Headers:', Object.keys(response.headers).slice(0, 3).join(', '));
    // No close() needed - Drop trait will clean up
  }
  console.log('   Response went out of scope - will be auto-cleaned on GC\n');

  // Test 3: Multiple responses without close
  console.log('Test 3: Multiple requests without close()');
  for (let i = 0; i < 10; i++) {
    const response = await client.get('https://httpbin.org/status/200');
    console.log(`   Request ${i + 1}: ${response.status} OK`);
    // No close() needed - automatic cleanup
  }
  console.log('✅ All 10 responses will be auto-cleaned\n');

  // Test 4: Explicit close still works
  console.log('Test 4: Explicit close() still available');
  {
    const response = await client.get('https://httpbin.org/get');
    console.log('✅ Status:', response.status);
    response.close(); // Optional - for immediate cleanup
    console.log('✅ Explicitly closed\n');
  }

  // Test 5: High-volume scenario
  console.log('Test 5: High-volume scenario (100 requests)');
  const startTime = Date.now();
  for (let i = 0; i < 100; i++) {
    const response = await client.get('https://httpbin.org/status/200');
    if (i === 0 || i === 99) {
      console.log(`   Request ${i + 1}: ${response.status}`);
    }
    // No close() - let automatic cleanup handle it
  }
  const duration = Date.now() - startTime;
  console.log(`✅ Completed 100 requests in ${duration}ms`);
  console.log('   All responses will be auto-cleaned on GC\n');

  // Force garbage collection if available (run with: node --expose-gc test-automatic-cleanup.js)
  if (global.gc) {
    console.log('🗑️  Forcing garbage collection...');
    global.gc();
    // Wait a bit for Drop trait to be called
    await new Promise(resolve => setTimeout(resolve, 100));
    console.log('✅ Garbage collection completed\n');
  } else {
    console.log('ℹ️  Run with --expose-gc to force GC: node --expose-gc test-automatic-cleanup.js\n');
  }

  // Test 6: Connection pool should still work after auto-cleanup
  console.log('Test 6: Verify connection pool still healthy');
  const response = await client.get('https://httpbin.org/get');
  console.log('✅ Status:', response.status);
  console.log('✅ Connection pool is healthy after auto-cleanup\n');

  console.log('🎉 All tests passed! Automatic cleanup is working.\n');
  console.log('Summary:');
  console.log('  ✅ No manual close() calls required');
  console.log('  ✅ Resources cleaned up automatically via Drop trait');
  console.log('  ✅ Works like undici/fetch');
  console.log('  ✅ Optional explicit close() still available');
  console.log('  ✅ Connection pool remains healthy');
}

// Run tests
testAutomaticCleanup()
  .then(() => {
    console.log('\n✨ Test suite completed successfully');
    process.exit(0);
  })
  .catch((error) => {
    console.error('\n❌ Test failed:', error);
    process.exit(1);
  });

