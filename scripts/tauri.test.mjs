import assert from 'node:assert/strict'
import { spawn } from 'node:child_process'
import { once } from 'node:events'
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import net from 'node:net'
import os from 'node:os'
import path from 'node:path'
import { test } from 'node:test'

import { runTauri } from './tauri.mjs'

async function occupy(port) {
  const server = net.createServer()
  server.listen(port, 'localhost')
  try {
    await once(server, 'listening')
    return server
  } catch (error) {
    if (error.code !== 'EADDRINUSE') throw error
    return undefined
  }
}

async function fixture(t, exitCode = 0) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'dyd-dev-test-'))
  t.after(() => rm(directory, { recursive: true, force: true }))
  const result = path.join(directory, 'result.json')
  const cli = path.join(directory, 'cli.mjs')
  await writeFile(
    cli,
    `import { writeFile } from 'node:fs/promises'
const args = process.argv.slice(2)
const index = args.lastIndexOf('--config')
const config = index < 0 ? null : JSON.parse(args[index + 1])
const html = config ? await (await fetch(config.build.devUrl)).text() : null
await writeFile(${JSON.stringify(result)}, JSON.stringify({ args, config, html }))
process.exitCode = ${exitCode}
`,
  )
  return { cli, read: async () => JSON.parse(await readFile(result, 'utf8')) }
}

test('occupied ports: Tauri receives the live Vite URL; failure releases its port', async (t) => {
  for (const port of [1420, 1421, 1422]) {
    const server = await occupy(port)
    if (server) t.after(() => new Promise((resolve) => server.close(resolve)))
  }
  const fake = await fixture(t, 7)
  const args = [
    'dev',
    '--no-watch',
    '--config',
    '{"app":{"withGlobalTauri":true}}',
    '--',
    '--example',
  ]
  assert.equal(await runTauri(args, fake.cli), 7)
  const { config, html, args: received } = await fake.read()
  const url = new URL(config.build.devUrl)
  assert.ok(Number(url.port) >= 1423)
  assert.equal(config.build.beforeDevCommand, '')
  assert.match(html, /\/src\/main.tsx/)
  assert.deepEqual(received.slice(0, 4), args.slice(0, 4))
  assert.deepEqual(received.slice(-2), ['--', '--example'])
  await assert.rejects(fetch(url, { signal: AbortSignal.timeout(2000) }))
})

test('available default port is retained and released after success', async (t) => {
  const probe = await occupy(1420)
  if (!probe) return t.skip('Port 1420 is already used by another application')
  await new Promise((resolve) => probe.close(resolve))
  const fake = await fixture(t)
  assert.equal(await runTauri(['dev'], fake.cli), 0)
  const { config } = await fake.read()
  assert.equal(new URL(config.build.devUrl).port, '1420')
  await assert.rejects(fetch(config.build.devUrl, { signal: AbortSignal.timeout(2000) }))
})

test('build and help commands pass through without starting Vite', async (t) => {
  const fake = await fixture(t)
  for (const args of [['build', '--debug'], ['dev', '--help'], ['--version']]) {
    assert.equal(await runTauri(args, fake.cli), 0)
    const result = await fake.read()
    assert.deepEqual(result.args, args)
    assert.equal(result.config, null)
  }
})

test('termination is forwarded to the CLI and closes Vite', { timeout: 10000 }, async (t) => {
  const fake = await fixture(t)
  const source = await readFile(fake.cli, 'utf8')
  await writeFile(
    fake.cli,
    `${source}\nsetInterval(() => {}, 1000)\nprocess.kill(process.ppid, 'SIGTERM')\n`,
  )
  const launcher = path.join(path.dirname(fake.cli), 'launcher.mjs')
  await writeFile(
    launcher,
    `import { runTauri } from ${JSON.stringify(new URL('./tauri.mjs', import.meta.url).href)}
process.exitCode = await runTauri(['dev'], ${JSON.stringify(fake.cli)})
`,
  )
  const child = spawn(process.execPath, [launcher], { stdio: 'inherit' })
  t.after(() => child.kill())
  const [code] = await once(child, 'exit')
  assert.equal(code, 143)
  const { config } = await fake.read()
  await assert.rejects(fetch(config.build.devUrl, { signal: AbortSignal.timeout(2000) }))
})
