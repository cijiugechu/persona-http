import http from 'node:http'
import { AddressInfo } from 'node:net'

import test from 'ava'

import { Client, get } from '../index.js'

type HeaderPayload = { headers: http.IncomingHttpHeaders }

type TestServer = {
  url: string
  close: () => Promise<void>
}

async function startHeaderServer(): Promise<TestServer> {
  return await new Promise<TestServer>((resolve, reject) => {
    const server = http.createServer((req, res) => {
      res.setHeader('content-type', 'application/json')
      res.end(JSON.stringify({ headers: req.headers }))
    })

    server.listen(0, '127.0.0.1', () => {
      const address = server.address() as AddressInfo
      resolve({
        url: `http://127.0.0.1:${address.port}`,
        close: () =>
          new Promise<void>((resolveClose, rejectClose) => {
            server.close((err) => (err ? rejectClose(err) : resolveClose()))
          }),
      })
    })

    server.on('error', reject)
  })
}

test('get', async (t) => {
  const server = await startHeaderServer()
  try {
    const response = await get(server.url)
    const body = (await response.json()) as HeaderPayload
    response.close()
    t.truthy(body.headers.host)
  } finally {
    await server.close()
  }
})

test('tls', async (t) => {
  const url = 'https://google.com'
  const client = new Client({
    emulation: 'chrome_133',
  })
  const response = await client.get(url)
  const body = await response.text()
  response.close()
  t.truthy(body)
})

test('client emulation applies chrome preset', async (t) => {
  const server = await startHeaderServer()
  try {
    const client = new Client({ emulation: 'chrome_105' })
    const response = await client.get(server.url)
    const body = (await response.json()) as HeaderPayload
    response.close()
    const userAgent = body.headers['user-agent']
    t.truthy(userAgent && userAgent.includes('Chrome/105'))
    t.truthy(body.headers['sec-ch-ua'])
  } finally {
    await server.close()
  }
})

test('request emulation overrides client preset', async (t) => {
  const server = await startHeaderServer()
  try {
    const client = new Client({ emulation: 'chrome_105' })
    const response = await client.get(server.url, { emulation: 'chrome_101' })
    const body = (await response.json()) as HeaderPayload
    response.close()
    const userAgent = body.headers['user-agent']
    t.truthy(userAgent && userAgent.includes('Chrome/101'))
  } finally {
    await server.close()
  }
})

test('skipHeaders disables client hint headers', async (t) => {
  const server = await startHeaderServer()
  try {
    const client = new Client()

    const responseWithHeaders = await client.get(server.url, { emulation: 'chrome_105' })
    const bodyWithHeaders = (await responseWithHeaders.json()) as HeaderPayload
    responseWithHeaders.close()

    const responseWithoutHeaders = await client.get(server.url, {
      emulation: { preset: 'chrome_105', skipHeaders: true },
    })
    const bodyWithoutHeaders = (await responseWithoutHeaders.json()) as HeaderPayload
    responseWithoutHeaders.close()

    t.truthy(bodyWithHeaders.headers['sec-ch-ua'])
    t.is(bodyWithoutHeaders.headers['sec-ch-ua'], undefined)
  } finally {
    await server.close()
  }
})
