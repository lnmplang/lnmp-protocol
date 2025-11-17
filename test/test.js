const { version } = require('../src')
const assert = require('assert')

try {
  const v = version()
  assert.ok(typeof v === 'string' && v.length > 0)
  console.log('OK', v)
} catch (e) {
  console.error('Test failed:', e)
  process.exit(1)
}
