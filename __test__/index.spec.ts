import test from 'ava'

import { get } from '../index.js'

test('get', async (t) => {
  const r = await get('https://httpbin.org/get')
  t.is(r.status, 200)
  const text = await r.text()
  t.is(text.includes('httpbin.org'), true)
})
