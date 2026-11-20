#!/usr/bin/env node
import { spawn } from 'node:child_process'
import { constants } from 'node:os'
import { fileURLToPath } from 'node:url'
import { executablePath } from './platforms.mjs'

try {
  const executable = executablePath(fileURLToPath(new URL('../', import.meta.url)))
  const child = spawn(executable, process.argv.slice(2), { stdio: 'inherit' })
  for (const signal of ['SIGINT', 'SIGTERM']) {
    process.on(signal, () => child.kill(signal))
  }
  child.on('error', error => {
    console.error(`Cannot start sds-site-generator: ${error.message}. Reinstall the package, or run npm run build:generator in a source checkout.`)
    process.exitCode = 1
  })
  child.on('exit', (code, signal) => {
    process.exitCode = code ?? (128 + (constants.signals[signal] ?? 1))
  })
} catch (error) {
  console.error(error.message)
  process.exitCode = 1
}