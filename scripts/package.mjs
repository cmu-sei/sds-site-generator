import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { chmodSync, copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
import { delimiter, isAbsolute, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import semver from 'semver'
import { parse, stringify } from 'smol-toml'
import { executablePath, platforms, selectPlatform } from './platforms.mjs'

export const root = fileURLToPath(new URL('../', import.meta.url))

export function metadata(directory = root) {
  const manifest = JSON.parse(readFileSync(join(directory, 'package.json'), 'utf8'))
  const cargo = parse(readFileSync(join(directory, 'Cargo.toml'), 'utf8'))
  if (semver.valid(manifest.version) !== manifest.version || manifest.version !== cargo.package.version) {
    throw new Error('package.json and Cargo.toml must have the same valid release version.')
  }
  if (!/^@[a-z0-9][a-z0-9-]*\/[a-z0-9][a-z0-9-]*$/.test(manifest.name)) {
    throw new Error('Use a scoped GitHub Packages name, such as @cmu-sei/sds-site-generator.')
  }
  if (manifest.publishConfig?.registry !== 'https://npm.pkg.github.com') {
    throw new Error('The package registry must be https://npm.pkg.github.com.')
  }
  return manifest
}

export function releaseIdentity(tag, owner, directory = root) {
  const manifest = metadata(directory)
  if (tag !== `v${manifest.version}`) throw new Error('The release tag must match the Cargo and npm version exactly.')
  if (!manifest.name.startsWith(`@${owner.toLowerCase()}/`)) throw new Error('The npm scope must match the GitHub repository owner.')
  const prerelease = semver.prerelease(manifest.version)
  if (prerelease && !/^beta\.[1-9]\d*$/.test(prerelease.join('.'))) throw new Error('Only beta.N prereleases are supported.')
  return { version: manifest.version, distTag: prerelease ? 'beta' : 'latest' }
}

export function run(command, args, options = {}) {
  return execFileSync(command, args, { cwd: root, stdio: 'inherit', ...options })
}

export function runCargo(args, options = {}) {
  try {
    return run('cargo', args, options)
  } catch (error) {
    if (error.code !== 'ENOENT') throw error
  }
  const env = options.env || process.env
  const extension = process.platform === 'win32' ? '.exe' : ''
  let bin = join(env.CARGO_HOME || join(homedir(), '.cargo'), 'bin')
  let standalone = false
  if (!existsSync(join(bin, `cargo${extension}`))) {
    const rustupHome = env.RUSTUP_HOME || join(homedir(), '.rustup')
    const toolchains = join(rustupHome, 'toolchains')
    const settings = join(rustupHome, 'settings.toml')
    const selected = env.RUSTUP_TOOLCHAIN || (existsSync(settings) ? parse(readFileSync(settings, 'utf8')).default_toolchain : undefined)
    if (selected !== undefined && (typeof selected !== 'string' || !selected)) throw new Error('Invalid default Rust toolchain in rustup settings.toml.')
    const directory = selected && (isAbsolute(selected) ? selected : join(toolchains, selected))
    const channel = selected || 'stable'
    let candidates = []
    if (directory && existsSync(directory)) candidates = [join(directory, 'bin')]
    else if (!isAbsolute(channel) && existsSync(toolchains)) {
      candidates = readdirSync(toolchains)
        .filter(name => name === channel || name.startsWith(`${channel}-`))
        .map(name => join(toolchains, name, 'bin'))
    }
    const installed = candidates.filter(bin => ['cargo', 'rustc'].every(name => existsSync(join(bin, `${name}${extension}`))))
    if (installed.length > 1) throw new Error(`Multiple ${channel} Rust toolchains found. Select one with RUSTUP_TOOLCHAIN or add rustup to PATH.`)
    if (!installed.length) throw new Error(selected
      ? `Rust toolchain "${selected}" is not installed. Install it with rustup toolchain install ${selected}, then run npm run build:generator again.`
      : 'Rust is required to build from source. Install it from https://rustup.rs, then run npm run build:generator again.')
    bin = installed[0]
    standalone = true
  }
  const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') || 'PATH'
  const childEnv = { ...env, [pathKey]: [bin, env[pathKey]].filter(Boolean).join(delimiter) }
  if (standalone) {
    const library = join(bin, '..', 'lib')
    if (process.platform === 'win32') childEnv[pathKey] = [bin, library, env[pathKey]].filter(Boolean).join(delimiter)
    else {
      const libraryKey = process.platform === 'darwin' ? 'DYLD_FALLBACK_LIBRARY_PATH' : 'LD_LIBRARY_PATH'
      const defaults = process.platform === 'darwin' ? [join(homedir(), 'lib'), '/usr/local/lib', '/usr/lib'].join(delimiter) : ''
      childEnv[libraryKey] = [library, childEnv[libraryKey] ?? defaults].filter(Boolean).join(delimiter)
    }
  }
  return run(join(bin, `cargo${extension}`), args, { ...options, env: childEnv })
}

function verifyHostBinary(binary) {
  const expected = `sds-site-generator ${metadata().version}`
  const actual = run(binary, ['--version'], { encoding: 'utf8', stdio: 'pipe' }).trim()
  if (actual !== expected) throw new Error(`Expected ${expected}; binary reports ${actual}.`)
}

export function verifyBundle(directory = root) {
  metadata(directory)
  for (const selected of platforms) {
    const binary = executablePath(directory, selected)
    if (!existsSync(binary) || !statSync(binary).isFile() || statSync(binary).size === 0) {
      throw new Error(`Missing release binary: ${selected.platform}/${selected.arch}. Build all six targets before packing.`)
    }
    if (selected.platform !== 'win32' && !(statSync(binary).mode & 0o111)) {
      throw new Error(`Binary is not executable: ${selected.platform}/${selected.arch}.`)
    }
  }
  for (const file of ['LICENSE', 'scripts/cli.mjs', 'scripts/platforms.mjs', 'vendor/sds-lite/LICENSE', 'vendor/lucide-LICENSE', 'assets/highlight-LICENSE.txt']) {
    if (!existsSync(join(directory, file))) throw new Error(`Missing package file: ${file}.`)
  }
  if (directory === root) verifyHostBinary(executablePath(root))
}

function stage(binary) {
  verifyHostBinary(binary)
  const destination = executablePath(root)
  mkdirSync(join(destination, '..'), { recursive: true })
  const temporary = mkdtempSync(join(destination, '..', '.stage-'))
  try {
    const staged = join(temporary, selectPlatform().executable)
    copyFileSync(binary, staged)
    chmodSync(staged, 0o755)
    renameSync(staged, destination)
    verifyHostBinary(destination)
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
}

export function buildPages(baseUrl, directory = root) {
  const url = new URL(baseUrl)
  if (url.protocol !== 'https:' || url.search || url.hash) throw new Error('Pages needs a clean HTTPS deployment URL.')
  if (!url.pathname.endsWith('/')) url.pathname += '/'
  const output = join(directory, 'pages-dist')
  if (existsSync(output) && !existsSync(join(output, '.site-generator-output'))) {
    throw new Error('Refusing to replace a pages-dist directory not owned by the generator.')
  }
  const temporary = mkdtempSync(join(tmpdir(), 'site-generator-pages-'))
  try {
    const site = join(temporary, 'site')
    const documentation = join(directory, 'docs-site')
    cpSync(documentation, site, { recursive: true, filter: source => source !== join(documentation, 'dist') })
    const config = parse(readFileSync(join(site, 'site.toml'), 'utf8'))
    config.url = url.href
    writeFileSync(join(site, 'site.toml'), stringify(config))
    const launcher = join(directory, 'scripts/cli.mjs')
    run(process.execPath, [launcher, 'check', site, ...(config.compatibility === true ? [] : ['--strict'])])
    run(process.execPath, [launcher, 'build', site])
    rmSync(output, { recursive: true, force: true })
    cpSync(join(site, 'dist'), output, { recursive: true })
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
}

function archives() {
  verifyBundle()
  const manifest = metadata()
  const output = join(root, 'release-artifacts')
  mkdirSync(output, { recursive: true })
  const tarball = `${manifest.name.slice(1).replace('/', '-')}-${manifest.version}.tgz`
  copyFileSync(join(root, tarball), join(output, tarball))
  for (const selected of platforms) {
    const temporary = mkdtempSync(join(tmpdir(), 'site-generator-archive-'))
    try {
      const files = [selected.executable, 'README.md', 'SDS-Lite-LICENSE.txt', 'Lucide-LICENSE.txt', 'highlight-LICENSE.txt', 'LICENSE']
      copyFileSync(executablePath(root, selected), join(temporary, selected.executable))
      chmodSync(join(temporary, selected.executable), 0o755)
      copyFileSync(join(root, 'README.md'), join(temporary, files[1]))
      copyFileSync(join(root, 'vendor/sds-lite/LICENSE'), join(temporary, files[2]))
      copyFileSync(join(root, 'vendor/lucide-LICENSE'), join(temporary, files[3]))
      copyFileSync(join(root, 'assets/highlight-LICENSE.txt'), join(temporary, files[4]))
      copyFileSync(join(root, 'LICENSE'), join(temporary, files[5]))
      const extension = selected.platform === 'win32' ? 'zip' : 'tar.gz'
      const archive = join(output, `sds-site-generator-${selected.platform}-${selected.arch}-v${manifest.version}.${extension}`)
      if (existsSync(archive)) rmSync(archive)
      run(selected.platform === 'win32' ? 'zip' : 'tar', selected.platform === 'win32' ? ['-q', archive, ...files] : ['-czf', archive, ...files], { cwd: temporary })
    } finally {
      rmSync(temporary, { recursive: true, force: true })
    }
  }
  const files = [tarball, ...platforms.map(selected => `sds-site-generator-${selected.platform}-${selected.arch}-v${manifest.version}.${selected.platform === 'win32' ? 'zip' : 'tar.gz'}`)].sort()
  writeFileSync(join(output, 'SHA256SUMS'), files.map(file => `${createHash('sha256').update(readFileSync(join(output, file))).digest('hex')}  ${file}\n`).join(''))
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [command, ...args] = process.argv.slice(2)
    const [argument] = args
    metadata()
    if (command === 'matrix') console.log(JSON.stringify({ include: platforms.map(selected => ({ ...selected, name: `${selected.platform}-${selected.arch}` })) }))
    else if (command === 'version') console.log(metadata().version)
    else if (command === 'release') console.log(JSON.stringify(releaseIdentity(argument, process.env.GITHUB_REPOSITORY_OWNER || '')))
    else if (command === 'build') {
      runCargo(['build', '--release', '--locked'])
      stage(join(root, 'target/release', selectPlatform().executable))
    } else if (command === 'test') runCargo(['test', '--all-targets', '--locked', ...args])
    else if (command === 'stage' && argument) stage(resolve(argument))
    else if (command === 'permissions') {
      for (const selected of platforms) chmodSync(executablePath(root, selected), 0o755)
    } else if (command === 'verify') verifyBundle()
    else if (command === 'archives') archives()
    else if (command === 'pages' && argument) buildPages(argument)
    else throw new Error('Usage: package.mjs build | test [cargo test arguments] | stage <binary> | permissions | verify | matrix | version | release <tag> | archives | pages <url>')
  } catch (error) {
    console.error(error.message)
    process.exitCode = 1
  }
}