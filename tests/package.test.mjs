import assert from 'node:assert/strict'
import { join } from 'node:path'
import { cpSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'
import { parse as parseYaml } from 'yaml'
import test from 'node:test'
import { executablePath, platforms, selectPlatform } from '../scripts/platforms.mjs'
import { buildPages, metadata, releaseIdentity, root, run, verifyBundle } from '../scripts/package.mjs'

test('npm launcher selects the native binary for all supported platforms', () => {
  assert.equal(platforms.length, 6)
  assert.equal(new Set(platforms.map(selected => `${selected.platform}-${selected.arch}`)).size, 6)
  for (const platform of ['darwin', 'linux', 'win32']) {
    for (const arch of ['x64', 'arm64']) {
      const selected = selectPlatform(platform, arch)
      assert.equal(executablePath('package', selected), join('package', 'native', `${platform}-${arch}`, platform === 'win32' ? 'sds-site-generator.exe' : 'sds-site-generator'))
    }
  }
})

test('unsupported platforms fail explicitly rather than running the wrong binary', () => {
  assert.throws(() => selectPlatform('freebsd', 'x64'), /Unsupported platform/)
  assert.throws(() => selectPlatform('linux', 'ia32'), /Unsupported platform/)
})

test('release identity checks version, tag, scope, and prerelease channel', () => {
  const manifest = metadata()
  const owner = manifest.name.split('/')[0].slice(1)
  assert.deepEqual(releaseIdentity(`v${manifest.version}`, owner), { version: manifest.version, distTag: 'latest' })
  assert.throws(() => releaseIdentity('v9.9.9', owner), /tag must match/)
  assert.throws(() => releaseIdentity(`v${manifest.version}`, 'another-owner'), /scope must match/)
  const temporary = mkdtempSync(join(tmpdir(), 'site-generator-metadata-'))
  try {
    writeFileSync(join(temporary, 'package.json'), JSON.stringify({ ...manifest, version: '0.2.0-beta.1' }))
    writeFileSync(join(temporary, 'Cargo.toml'), '[package]\nversion = "0.2.0-beta.1"\n')
    assert.equal(releaseIdentity('v0.2.0-beta.1', owner, temporary).distTag, 'next')
    writeFileSync(join(temporary, 'Cargo.toml'), '[package]\nversion = "0.1.0"\n')
    assert.throws(() => metadata(temporary), /same valid release version/)
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test('packing fails closed when release binaries are missing', () => {
  const temporary = mkdtempSync(join(tmpdir(), 'site-generator-incomplete-'))
  try {
    for (const file of ['package.json', 'Cargo.toml']) cpSync(join(root, file), join(temporary, file))
    assert.throws(() => verifyBundle(temporary), /Missing release binary/)
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test('native launcher preserves spaced paths and the CLI failure status', { skip: !existsSync(executablePath(root)) }, () => {
  const temporary = mkdtempSync(join(tmpdir(), 'site-generator-cli-'))
  const launcher = fileURLToPath(new URL('../scripts/cli.mjs', import.meta.url))
  try {
    const site = join(temporary, 'docs with spaces')
    run(process.execPath, [launcher, 'new', site, '--name', 'Package test', '--url', 'https://example.org/docs/'])
    run(process.execPath, [launcher, 'check', site, '--strict'])
    run(process.execPath, [launcher, 'build', site])
    assert.ok(existsSync(join(site, 'dist/index.html')))
    assert.throws(() => run(process.execPath, [launcher, 'check', join(temporary, 'missing')], { stdio: 'pipe' }), error => error.status === 1)
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test('Pages publishes the documentation at root and project URLs without changing its configuration', { skip: !existsSync(executablePath(root)) }, () => {
  const temporary = mkdtempSync(join(tmpdir(), 'site-generator-pages-test-'))
  try {
    for (const file of ['docs-site', 'scripts/cli.mjs', 'scripts/platforms.mjs', 'native']) cpSync(join(root, file), join(temporary, file), { recursive: true })
    const config = readFileSync(join(temporary, 'docs-site/site.toml'), 'utf8')
    for (const url of ['https://example.org/project/', 'https://example.org/']) {
      buildPages(url, temporary)
      const html = readFileSync(join(temporary, 'pages-dist/index.html'), 'utf8')
      assert.ok(html.includes(`href="${url}_site/`) || html.includes(`href="${new URL(url).pathname}_site/`))
      assert.ok(html.includes('Publish documentation from Markdown'))
      assert.ok(html.includes(`href="${new URL(url).pathname}guides/start/getting-started/"`))
      const assets = readFileSync(join(temporary, 'pages-dist/guides/authoring/assets/index.html'), 'utf8')
      assert.ok(assets.includes(`src="${new URL(url).pathname}images/authoring-flow.svg"`))
      assert.ok(existsSync(join(temporary, 'pages-dist/downloads/author-checklist.txt')))
      const nested = readFileSync(join(temporary, 'pages-dist/guides/publishing/team-workflows/index.html'), 'utf8')
      const sidebar = nested.match(/<nav aria-label="Documentation">([\s\S]*?)<\/nav>/)?.[1]
      assert.ok(sidebar, 'nested guide has a documentation sidebar')
      const groups = [...sidebar.matchAll(/<details( open)?><summary><span class="sds-sr-only">Toggle (.*?) navigation<\/span><\/summary>/g)]
      assert.deepEqual(groups.map(group => [group[2], Boolean(group[1])]), [
        ['Start here', false],
        ['Write content', false],
        ['Publish and operate', true],
        ['Maintain and migrate', false],
      ])
      assert.ok(sidebar.includes(`href="${new URL(url).pathname}guides/publishing/team-workflows/" aria-current="page"`))
      assert.deepEqual(JSON.parse(readFileSync(join(temporary, 'pages-dist/_site/diagnostics.json'), 'utf8')), [])
      assert.ok(existsSync(join(temporary, 'pages-dist/.nojekyll')))
      assert.equal(readFileSync(join(temporary, 'docs-site/site.toml'), 'utf8'), config)
    }
    assert.throws(() => buildPages('http://example.org/', temporary), /HTTPS/)
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test('npm tarball supports offline installation and repo-local documentation scripts', { skip: !existsSync(executablePath(root)) || !process.env.npm_execpath }, () => {
  const temporary = mkdtempSync(join(tmpdir(), 'site-generator-install-'))
  try {
    const source = join(temporary, 'source')
    const manifest = metadata()
    for (const file of ['package.json', ...manifest.files]) cpSync(join(root, file), join(source, file), { recursive: true })
    const report = JSON.parse(run(process.execPath, [process.env.npm_execpath, 'pack', '--ignore-scripts', '--json'], { cwd: source, encoding: 'utf8', stdio: 'pipe' }))
    const entries = Array.isArray(report) ? report : Object.values(report)
    assert.equal(entries.length, 1)
    const packed = entries[0]
    const files = new Set(packed.files.map(file => file.path))
    assert.ok(files.has(`native/${process.platform}-${process.arch}/${selectPlatform().executable}`))
    for (const file of ['LICENSE', 'scripts/cli.mjs', 'scripts/platforms.mjs', 'vendor/sds-lite/LICENSE', 'vendor/lucide-LICENSE', 'assets/highlight-LICENSE.txt']) assert.ok(files.has(file))
    assert.ok(!files.has('Cargo.toml'))
    const installation = join(temporary, 'installation')
    run(process.execPath, [join(root, 'scripts/cli.mjs'), 'new', installation, '--url', 'https://example.org/my-docs/'], { stdio: 'pipe' })
    writeFileSync(join(installation, 'package.json'), JSON.stringify({
      name: 'my-docs',
      private: true,
      scripts: {
        dev: 'sds-site-generator preview .',
        check: 'sds-site-generator check . --strict',
        build: 'sds-site-generator build .',
      },
    }))
    run(process.execPath, [process.env.npm_execpath, 'install', '--prefix', installation, '--save-dev', '--save-exact', '--ignore-scripts', '--offline', '--no-audit', '--no-fund', join(source, packed.filename)], { stdio: 'pipe' })
    const version = run(process.execPath, [process.env.npm_execpath, 'exec', '--offline', '--', 'sds-site-generator', '--version'], { cwd: installation, encoding: 'utf8', stdio: 'pipe' }).trim()
    assert.equal(version, `sds-site-generator ${manifest.version}`)
    const consumer = JSON.parse(readFileSync(join(installation, 'package.json'), 'utf8'))
    assert.equal(readFileSync(join(installation, 'node_modules', manifest.name, 'LICENSE'), 'utf8'), readFileSync(join(root, 'LICENSE'), 'utf8'))
    assert.ok(Object.hasOwn(consumer.devDependencies, manifest.name))
    for (const command of ['check', 'build']) run(process.execPath, [process.env.npm_execpath, 'run', command], { cwd: installation, stdio: 'pipe' })
    const previewHelp = run(process.execPath, [process.env.npm_execpath, 'run', 'dev', '--', '--help'], { cwd: installation, encoding: 'utf8', stdio: 'pipe' })
    assert.match(previewHelp, /--port/)
    assert.ok(existsSync(join(installation, 'dist/index.html')))
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test('workflows use direct PR checks, tested release artifacts, and isolated Pages permissions', () => {
  const workflows = Object.fromEntries(['check', 'release', 'pages'].map(name => [name, parseYaml(readFileSync(join(root, `.github/workflows/${name}.yml`), 'utf8'))]))
  assert.ok(Object.hasOwn(workflows.check.on, 'pull_request'))
  assert.ok(Object.hasOwn(workflows.check.on, 'workflow_call'))
  assert.equal(workflows.release.jobs.build.uses, './.github/workflows/check.yml')
  assert.equal(workflows.release.jobs.publish.permissions.packages, 'write')
  assert.equal(workflows.pages.jobs.build.permissions.pages, 'read')
  assert.equal(workflows.pages.jobs.deploy.permissions.pages, 'write')
  assert.equal(workflows.pages.jobs.deploy.permissions['id-token'], 'write')
  for (const workflow of Object.values(workflows)) {
    assert.ok(!Object.hasOwn(workflow.on, 'pull_request_target'))
    for (const job of Object.values(workflow.jobs)) {
      for (const step of job.steps || []) {
        if (step.uses) assert.match(step.uses, /@[a-f0-9]{40}$/)
      }
    }
  }
})