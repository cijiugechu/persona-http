const { get } = require('./index')

get('https://www.google.com').then(r => r.text()).then(console.log)
