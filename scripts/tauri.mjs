import { spawn } from 'node:child_process'
import { fileURLToPath, pathToFileURL } from 'node:url'

import { createServer } from 'vite'

const projectRoot = fileURLToPath(new URL('../', import.meta.url))
const defaultCli = fileURLToPath(import.meta.resolve('@tauri-apps/cli/tauri.js'))

export async function runTauri(args, cliPath = defaultCli) {
  let server
  let child
  let stoppingSignal
  const stop = (signal) => {
    stoppingSignal = signal
    child?.kill(signal)
  }
  const onInterrupt = () => stop('SIGINT')
  const onTerminate = () => stop('SIGTERM')
  process.on('SIGINT', onInterrupt)
  process.on('SIGTERM', onTerminate)

  try {
    const separator = args.indexOf('--')
    const cliArgs = separator < 0 ? args : args.slice(0, separator)
    if (args[0] === 'dev' && !cliArgs.includes('--help') && !cliArgs.includes('-h')) {
      server = await createServer({ root: projectRoot })
      await server.listen()
      const devUrl = server.resolvedUrls?.local[0] ?? server.resolvedUrls?.network[0]
      if (!devUrl) throw new Error('Vite did not provide a development URL')
      server.printUrls()
      const config = JSON.stringify({ build: { devUrl, beforeDevCommand: '' } })
      args = [...cliArgs, '--config', config, ...(separator < 0 ? [] : args.slice(separator))]
    }

    if (stoppingSignal) return stoppingSignal === 'SIGINT' ? 130 : 143
    return await new Promise((resolve, reject) => {
      child = spawn(process.execPath, [cliPath, ...args], {
        cwd: projectRoot,
        stdio: 'inherit',
      })
      child.once('error', reject)
      child.once('exit', (code, signal) => {
        resolve(code ?? (signal === 'SIGINT' ? 130 : 143))
      })
    })
  } finally {
    await server?.close()
    process.off('SIGINT', onInterrupt)
    process.off('SIGTERM', onTerminate)
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    process.exitCode = await runTauri(process.argv.slice(2))
  } catch (error) {
    console.error(error)
    process.exitCode = 1
  }
}
