const { Client } = require('./index')

const client = new Client({
  emulation: 'chrome_133',
})

client.get('https://www.google.com').then(r => r.text()).then(console.log)
