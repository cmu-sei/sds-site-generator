import assert from 'node:assert/strict'
import { execFile, execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { chmodSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs'
import { createServer } from 'node:http'
import { tmpdir } from 'node:os'
import { delimiter, join } from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'
import { parse } from 'smol-toml'
import { metadata, root } from '../scripts/package.mjs'
import { platforms } from '../scripts/platforms.mjs'
import { checkDistributionTag, resolveVersion, synchronizeVersions, validatePullRequest, verifyArtifacts, verifyIntegrity } from '../scripts/release.mjs'

test('release versions support first release, sequenced betas, and stable promotion', () => {
  assert.equal(resolveVersion('0.1.0', 'stable', '0.1.0', []), '0.1.0')
  assert.equal(resolveVersion('0.2.0', 'beta', '0.1.0', ['v0.1.0', 'v0.2.0-beta.1', 'v0.2.0-beta.3']), '0.2.0-beta.4')
  assert.equal(resolveVersion('0.2.0', 'stable', '0.2.0-beta.4', ['v0.1.0', 'v0.2.0-beta.4']), '0.2.0')
  for (const base of ['v0.2.0', '0.2.0-beta.1', '0.2.0+build', 'garbage']) {
    assert.throws(() => resolveVersion(base, 'stable', '0.1.0', []), /base version/)
  }
  assert.throws(() => resolveVersion('0.1.0', 'stable', '0.1.0', ['v0.1.0']), /exceed/)
  assert.throws(() => resolveVersion('0.2.0', 'stable', '0.3.0-beta.1', []), /older/)
  assert.throws(() => resolveVersion('0.2.0', 'next', '0.1.0', []), /stable or beta/)
})

for (const [label, newline] of [['LF', '\n'], ['CRLF', '\r\n']]) {
  test(`preparation synchronizes all four manifests without modifying dependencies (${label})`, () => {
    const directory = mkdtempSync(join(tmpdir(), 'site-generator-release-'))
    try {
      const names = ['package.json', 'package-lock.json', 'Cargo.toml', 'Cargo.lock']
      for (const name of names) cpSync(join(root, name), join(directory, name))
      for (const name of ['Cargo.toml', 'Cargo.lock']) {
        const path = join(directory, name)
        writeFileSync(path, readFileSync(path, 'utf8').replace(/\r?\n/g, newline))
      }
      const originalCargo = readFileSync(join(directory, 'Cargo.toml'), 'utf8')
      const originalLock = parse(readFileSync(join(directory, 'Cargo.lock'), 'utf8'))
      const originalNpm = JSON.parse(readFileSync(join(directory, 'package-lock.json'), 'utf8'))
      synchronizeVersions(directory, '0.2.0-beta.1')
      assert.equal(metadata(directory).version, '0.2.0-beta.1')
      const cargo = readFileSync(join(directory, 'Cargo.toml'), 'utf8')
      assert.equal(cargo, originalCargo.replace(`version = "${metadata().version}"`, 'version = "0.2.0-beta.1"'))
      const lockText = readFileSync(join(directory, 'Cargo.lock'), 'utf8')
      assert.equal(lockText, lockText.replace(/\r?\n/g, newline), 'Cargo.lock must retain its line endings')
      const lock = parse(lockText)
      originalLock.package.find(pkg => pkg.name === 'sds-site-generator' && !pkg.source).version = '0.2.0-beta.1'
      assert.deepEqual(lock, originalLock)
      const npm = JSON.parse(readFileSync(join(directory, 'package-lock.json'), 'utf8'))
      originalNpm.version = '0.2.0-beta.1'
      originalNpm.packages[''].version = '0.2.0-beta.1'
      assert.deepEqual(npm, originalNpm)
    } finally {
      rmSync(directory, { recursive: true, force: true })
    }
  })
}

test('publication rejects fork, mismatched, unmerged, and cancelled release PRs', () => {
  const pr = {
    head: { ref: 'release/v0.2.0', repo: { full_name: 'cmu-sei/sds-site-generator' } },
    base: { ref: 'main' }, merged: true, merge_commit_sha: 'a'.repeat(40), labels: [],
  }
  const validate = value => validatePullRequest(value, 'cmu-sei/sds-site-generator', 'main', '0.2.0')
  validate(pr)
  assert.throws(() => validate({ ...pr, merged: false }), /active/)
  assert.throws(() => validate({ ...pr, labels: [{ name: 'release-cancelled' }] }), /active/)
  assert.throws(() => validate({ ...pr, head: { ...pr.head, repo: { full_name: 'fork/repo' } } }), /active/)
  assert.throws(() => validate({ ...pr, head: { ...pr.head, ref: 'release/v0.3.0' } }), /active/)
  assert.throws(() => validate({ ...pr, base: { ref: 'other' } }), /active/)
  assert.throws(() => validate({ ...pr, merge_commit_sha: '' }), /commit/)
})

test('published package retries require identical integrity and never move channels backward', () => {
  const bytes = Buffer.from('tested tarball')
  const published = { dist: { integrity: `sha512-${createHash('sha512').update(bytes).digest('base64')}` } }
  verifyIntegrity(bytes, published)
  assert.throws(() => verifyIntegrity(Buffer.from('different tarball'), published), /differ/)
  assert.throws(() => verifyIntegrity(bytes, null), /differ/)
  checkDistributionTag(null, '0.2.0', 'latest')
  checkDistributionTag({ latest: '0.2.0' }, '0.2.0', 'latest')
  checkDistributionTag({ beta: '0.2.0-beta.1' }, '0.2.0-beta.2', 'beta')
  assert.throws(() => checkDistributionTag({ latest: '0.3.0' }, '0.2.0', 'latest'), /backward/)
  assert.throws(() => checkDistributionTag({ beta: '0.2.0-beta.3' }, '0.2.0-beta.2', 'beta'), /backward/)
})

test('release bundles reject missing, extra, and tampered artifacts before publishing', () => {
  const directory = mkdtempSync(join(tmpdir(), 'site-generator-artifacts-'))
  try {
    const version = '0.2.0'
    const names = ['cmu-sei-sds-site-generator-0.2.0.tgz', ...platforms.map(platform =>
      `sds-site-generator-${platform.platform}-${platform.arch}-v${version}.${platform.platform === 'win32' ? 'zip' : 'tar.gz'}`)].sort()
    for (const name of names) writeFileSync(join(directory, name), name)
    writeFileSync(join(directory, 'SHA256SUMS'), names.map(name =>
      `${createHash('sha256').update(name).digest('hex')}  ${name}\n`).join(''))
    writeFileSync(join(directory, 'unexpected.zip'), '')
    assert.throws(() => verifyArtifacts(directory, version, '@cmu-sei/sds-site-generator'), /filenames or checksums/)
    rmSync(join(directory, 'unexpected.zip'))
    writeFileSync(join(directory, names[1]), 'tampered')
    assert.throws(() => verifyArtifacts(directory, version, '@cmu-sei/sds-site-generator'), /filenames or checksums/)
    rmSync(join(directory, names[1]))
    assert.throws(() => verifyArtifacts(directory, version, '@cmu-sei/sds-site-generator'), /ENOENT/)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test('release lifecycle prepares PRs, resumes publication, verifies Pages, and safely recovers', { skip: process.platform === 'win32' }, async () => {
  const directory = mkdtempSync(join(tmpdir(), 'site-generator-publication-'))
  const stateFile = join(directory, 'state.json')
  const readState = () => JSON.parse(readFileSync(stateFile, 'utf8'))
  const save = state => writeFileSync(stateFile, JSON.stringify(state))
  let sha
  const manifest = metadata()
  const version = '0.1.0'
  const tag = `v${version}`
  let origin
  const server = createServer(async (request, response) => {
    try {
      const state = readState()
      const url = new URL(request.url, origin)
      const path = url.pathname.replace('/repos/cmu-sei/sds-site-generator', '')
      let body = ''
      for await (const chunk of request) body += chunk
      const payload = body ? JSON.parse(body) : undefined
      const pr = {
        number: 7, head: { ref: `release/${tag}`, sha, repo: { full_name: 'cmu-sei/sds-site-generator' } },
        base: { ref: 'main' }, merged: state.prMerged ?? true, merge_commit_sha: sha, state: 'closed',
        labels: [...(state.cancelled ? [{ name: 'release-cancelled' }] : []), ...(state.requested ? [{ name: 'release-cancel-requested' }] : [])],
      }
      let result
      if (path === '') result = { default_branch: 'main' }
      else if (path === '/pulls/7') result = pr
      else if (path === '/pulls' && request.method === 'POST') {
        if (payload.head.startsWith('revert-release/')) {
          state.recoveryPr = {
            ...payload, number: 8, state: 'open', merged_at: null,
            head: { ref: payload.head, repo: { full_name: 'cmu-sei/sds-site-generator' } },
            base: { ref: payload.base }, html_url: 'https://github.com/cmu-sei/sds-site-generator/pull/8',
          }
          result = state.recoveryPr
        } else {
          state.preparedPr = { ...payload, number: 7, html_url: 'https://github.com/cmu-sei/sds-site-generator/pull/7' }
          result = state.preparedPr
        }
      }
      else if (path === '/pulls') {
        const { merged, ...listedPr } = pr
        listedPr.merged_at = merged ? '2026-11-20T19:00:00Z' : null
        const head = url.searchParams.get('head')?.replace(/^cmu-sei:/, '')
        const requestedState = url.searchParams.get('state')
        result = [...(state.noPr ? [] : [listedPr]), ...(state.recoveryPr ? [state.recoveryPr] : [])]
          .filter(candidate => (!head || candidate.head.ref === head) && (requestedState === 'all' || candidate.state === requestedState))
      }
      else if (path === '/pulls/7/files') result = ['package.json', 'package-lock.json', 'Cargo.toml', 'Cargo.lock'].map(filename => ({ filename }))
      else if (path === '/releases' && request.method === 'POST') {
        state.release = { ...payload, id: 1, html_url: `https://github.com/cmu-sei/sds-site-generator/releases/tag/${payload.tag_name}` }
        result = state.release
      }
      else if (path === '/releases') result = state.release ? [state.release] : []
      else if (path === '/releases/generate-notes') result = { body: 'Generated notes' }
      else if (path === '/releases/1/assets') result = state.assets.map((asset, index) => ({ name: asset.name, url: `${origin}/assets/${index}` }))
      else if (path.startsWith('/assets/')) {
        response.end(Buffer.from(state.assets[Number(path.split('/').pop())].bytes, 'base64'))
        return
      }
      else if (path === `/git/ref/tags/${tag}`) result = state.tagged ? { object: { type: 'commit', sha } } : null
      else if (path.startsWith('/git/ref/heads/')) result = state.branch && path === `/git/ref/heads/release/${tag}` ? { object: { type: 'commit', sha } } : null
      else if (path === '/git/matching-refs/heads/release/') result = state.branch ? [{ ref: `refs/heads/release/${tag}`, object: { sha } }] : []
      else if (path.startsWith('/git/refs/heads/') && request.method === 'DELETE') {
        state.branch = false
      }
      else if (path === '/tags') result = state.tagged ? [{ name: tag }] : []
      else if (path === '/releases/1' && request.method === 'PATCH') {
        Object.assign(state.release, payload)
        state.tagged = true
        result = state.release
      } else if (path === '/releases/1' && request.method === 'DELETE') {
        state.release = null
      } else if (path.startsWith('/labels/')) result = null
      else if (path === '/labels' && request.method === 'POST') result = payload
      else if (path === '/issues/7/labels' && request.method === 'POST') {
        if (payload.labels.includes('release-cancel-requested')) state.requested = true
        if (payload.labels.includes('release-cancelled')) state.cancelled = true
        result = payload.labels
      } else if (path === '/issues/7/labels/release-cancel-requested' && request.method === 'DELETE') {
        state.requested = false
      } else if (path === '/actions/workflows/release.yml/runs') {
        result = {
          workflow_runs: state.runsOnSecondPage && url.searchParams.get('page') !== '2'
            ? Array.from({ length: 100 }, (_, index) => ({ id: index + 1000, display_title: 'Other release', status: 'completed' }))
            : state.activeRun ? [{ id: 9, display_title: 'Publish release PR #7', status: 'in_progress' }] : [],
        }
      } else if (path === '/actions/runs/9/cancel') {
        state.activeRun = false
        const raced = state.cancelRace
        state.cancelRace = false
        save(state)
        response.writeHead(raced ? 409 : 202).end(raced ? '{"message":"Run already completed"}' : '')
        return
      } else if (path === '/actions/runs/9') {
        result = { status: state.activeRun ? 'in_progress' : 'completed' }
      } else if (path === '/actions/workflows/check.yml/dispatches') {
        if (state.failDispatch) {
          state.failDispatch = false
          save(state)
          response.writeHead(503).end('{"message":"Simulated dispatch outage"}')
          return
        }
        state.dispatched = payload.ref
      } else {
        response.writeHead(500).end(`Unexpected request: ${request.method} ${path}`)
        return
      }
      save(state)
      if (result === undefined) {
        response.writeHead(204).end()
        return
      }
      response.writeHead(result === null ? 404 : 200, { 'Content-Type': 'application/json' })
      response.end(JSON.stringify(result))
    } catch (error) {
      response.writeHead(500).end(error.message)
    }
  })
  try {
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
    origin = `http://127.0.0.1:${server.address().port}`
    for (const name of ['scripts/package.mjs', 'scripts/platforms.mjs', 'scripts/release.mjs', 'package.json', 'package-lock.json', 'Cargo.toml', 'Cargo.lock']) {
      mkdirSync(join(directory, name, '..'), { recursive: true })
      cpSync(join(root, name), join(directory, name))
    }
    symlinkSync(join(root, 'node_modules'), join(directory, 'node_modules'), 'dir')
    const git = (...args) => execFileSync('git', args, { cwd: directory, encoding: 'utf8', stdio: 'pipe' }).trim()
    const remote = join(directory, 'remote.git')
    git('init', '--initial-branch=main')
    git('config', 'user.name', 'Release Test')
    git('config', 'user.email', 'release-test@example.org')
    git('config', 'commit.gpgsign', 'false')
    mkdirSync(join(directory, 'no-hooks'))
    git('config', 'core.hooksPath', join(directory, 'no-hooks'))
    synchronizeVersions(directory, '0.0.9')
    git('add', 'package.json', 'package-lock.json', 'Cargo.toml', 'Cargo.lock')
    git('commit', '-m', 'Previous version')
    synchronizeVersions(directory, version)
    git('add', 'package.json', 'package-lock.json', 'Cargo.toml', 'Cargo.lock')
    git('commit', '-m', `release: prepare ${tag}`)
    sha = git('rev-parse', 'HEAD')
    git('init', '--bare', remote)
    git('--git-dir', remote, 'config', 'core.hooksPath', join(directory, 'no-hooks'))
    git('remote', 'add', 'origin', remote)
    git('push', '--set-upstream', 'origin', 'main')
    const bin = join(directory, 'bin')
    const bundle = join(directory, 'release-artifacts')
    mkdirSync(bin)
    mkdirSync(bundle)
    const tarSource = join(directory, 'packed')
    mkdirSync(join(tarSource, 'package'), { recursive: true })
    cpSync(join(directory, 'package.json'), join(tarSource, 'package/package.json'))
    const tarballName = `${manifest.name.slice(1).replace('/', '-')}-${version}.tgz`
    execFileSync('tar', ['-czf', join(bundle, tarballName), '-C', tarSource, 'package'])
    const archives = platforms.map(platform => `sds-site-generator-${platform.platform}-${platform.arch}-v${version}.${platform.platform === 'win32' ? 'zip' : 'tar.gz'}`)
    for (const name of archives) writeFileSync(join(bundle, name), `binary ${name}`)
    const files = [tarballName, ...archives].sort()
    writeFileSync(join(bundle, 'SHA256SUMS'), files.map(name =>
      `${createHash('sha256').update(readFileSync(join(bundle, name))).digest('hex')}  ${name}\n`).join(''))
    const initialState = { release: { id: 1, tag_name: tag, draft: true, prerelease: false, target_commitish: 'main', html_url: `https://github.com/cmu-sei/sds-site-generator/releases/tag/${tag}` }, assets: [], tags: {}, published: null, publishes: 0 }
    save({ ...initialState, failPublish: true })
    const shim = `
import { readFileSync, writeFileSync } from 'node:fs'
import { createHash } from 'node:crypto'
import { basename } from 'node:path'
const file = process.env.RELEASE_TEST_STATE
const state = JSON.parse(readFileSync(file, 'utf8'))
const args = process.argv.slice(2)
const tool = basename(process.argv[1])
if (tool === 'gh') {
  if (args[0] !== 'release' || args[1] !== 'upload') throw new Error('Unexpected gh command')
  state.assets.push({ name: basename(args[3]), bytes: readFileSync(args[3]).toString('base64') })
} else if (args[0] === 'view') {
  const value = args.includes('dist-tags') ? state.tags : args[1].endsWith('@${version}') ? state.published : null
  console.log(JSON.stringify(value ?? { error: { code: 'E404' } }))
  if (!value) process.exitCode = 1
} else if (args[0] === 'publish' && !args.includes('--dry-run')) {
  if (state.failPublish) {
    state.failPublish = false
    writeFileSync(file, JSON.stringify(state))
    console.error('Simulated registry outage')
    process.exit(1)
  }
  if (state.published) throw new Error('Duplicate immutable publication')
  state.published = { dist: { integrity: 'sha512-' + createHash('sha512').update(readFileSync(args[1])).digest('base64') } }
  state.publishes++
  state.tags[args[args.indexOf('--tag') + 1]] = '${version}'
} else if (args[0] === 'dist-tag') {
  state.tags[args[3]] = '${version}'
} else if (args[0] !== 'publish' || !args.includes('--dry-run')) throw new Error('Unexpected npm command')
writeFileSync(file, JSON.stringify(state))
`
    for (const tool of ['gh', 'npm']) {
      const path = join(bin, tool)
      writeFileSync(path, `#!${process.execPath}\n${shim}`)
      chmodSync(path, 0o755)
    }
    const run = async (action, extra = {}) => promisify(execFile)(process.execPath, ['scripts/release.mjs', action], {
      cwd: directory,
      env: {
        ...process.env, PATH: `${bin}${delimiter}${process.env.PATH}`, GITHUB_API_URL: origin,
        GITHUB_REPOSITORY: 'cmu-sei/sds-site-generator', GH_TOKEN: 'test-token', NODE_AUTH_TOKEN: 'test-token',
        GITHUB_REF: 'refs/heads/main',
        GITHUB_OUTPUT: join(directory, 'outputs'),
        GITHUB_STEP_SUMMARY: join(directory, 'summary'),
        RELEASE_PR_NUMBER: '7', RELEASE_TEST_STATE: stateFile, ...extra,
      },
    })
    await assert.rejects(run('publish'), /Simulated registry outage/)
    assert.equal(readState().release.draft, false, 'GitHub publication can precede a registry failure')
    assert.equal(readState().published, null)
    await run('publish')
    assert.equal(readState().publishes, 1)
    assert.equal(readState().assets.length, 8)
    assert.equal(readState().release.draft, false)
    assert.deepEqual(JSON.parse(readFileSync(join(directory, 'pages-release.json'), 'utf8')), { tag, sha })
    await run('publish')
    assert.equal(readState().publishes, 1, 'a successful retry must not republish npm')
    assert.equal(readState().assets.length, 8, 'a successful retry must not replace assets')
    await run('pages')
    await assert.rejects(run('pages', { PAGES_TAG: 'v0.2.0-beta.1' }), /published stable tag/)
    const publishedState = readState()
    save({ ...publishedState, published: { dist: { integrity: 'sha512-wrong' } } })
    await assert.rejects(run('publish'), /Published package bytes differ/)
    save({ ...publishedState, tags: { latest: '99.0.0' } })
    await assert.rejects(run('publish'), /backward/)
    const skipped = await run('pages')
    assert.match(skipped.stdout, /no longer npm latest/)
    save({ ...publishedState, cancelled: true })
    await assert.rejects(run('publish'), /active, same-repository release PR/)
    save({ ...publishedState, assets: publishedState.assets.map((asset, index) => index ? asset : { ...asset, bytes: Buffer.from('tampered').toString('base64') }) })
    await assert.rejects(run('publish'), /differs from the tested artifact/)
    assert.equal(readState().publishes, 1)

    const recoveryEnv = { RELEASE_VERSION: version, RELEASE_CONFIRMATION: version, RECOVERY_MODE: 'cancel' }
    save({ ...publishedState, requested: true })
    await assert.rejects(run('recover', recoveryEnv), /published artifacts/)
    assert.equal(readState().requested, false, 'a publication/cancellation race must leave partial publication resumable')
    assert.equal(readState().release.draft, false)

    save({ ...initialState, activeRun: true, runsOnSecondPage: true, cancelRace: true, failDispatch: true })
    await assert.rejects(run('stop', { ...recoveryEnv, RELEASE_CONFIRMATION: 'wrong' }), /Retype/)
    await run('stop', recoveryEnv)
    assert.equal(readState().activeRun, false)
    assert.equal(readState().requested, true)
    save({ ...readState(), activeRun: true, cancelRace: false })
    await run('stop', recoveryEnv)
    assert.equal(readState().activeRun, false, 'all run pages and empty 202 cancellation responses are handled')
    await assert.rejects(run('recover', recoveryEnv), /Simulated dispatch outage/)
    assert.equal(readState().cancelled, true)
    assert.equal(readState().requested, false)
    assert.equal(readState().release, null)
    assert.equal(readState().recoveryPr.head.ref, 'revert-release/7')
    assert.equal(metadata(directory).version, '0.0.9', 'recovery reverts the actual release preparation commit')
    await run('recover', recoveryEnv)
    assert.equal(readState().dispatched, 'revert-release/7', 'retrying an existing recovery PR redispatches required checks')
    await assert.rejects(run('publish'), /active, same-repository release PR/)

    git('branch', `release/${tag}`, sha)
    git('push', 'origin', `release/${tag}`)
    save({ ...initialState, branch: true, noPr: true })
    await run('recover', { ...recoveryEnv, RECOVERY_MODE: 'discard' })
    assert.equal(readState().branch, false, 'partial preparation can be discarded without a PR')
    assert.equal(readState().release, null)

    git('switch', 'main')
    save({ ...initialState, noPr: true })
    await assert.rejects(run('prepare', { BASE_VERSION: '0.2.0', RELEASE_CHANNEL: 'beta' }), /draft|partial preparation/)
    save({ ...initialState, release: null, branch: true, noPr: true })
    await assert.rejects(run('prepare', { BASE_VERSION: '0.2.0', RELEASE_CHANNEL: 'beta' }), /partial preparation/)
    save({ ...publishedState, branch: true, assets: publishedState.assets.map((asset, index) => index ? asset : { ...asset, name: 'wrong-archive.tar.gz' }) })
    await assert.rejects(run('prepare', { BASE_VERSION: '0.2.0', RELEASE_CHANNEL: 'beta' }), /Finish release #7/)
    save({ ...publishedState, branch: true })
    await run('prepare', { GITHUB_REF: 'refs/heads/main', BASE_VERSION: '0.2.0', RELEASE_CHANNEL: 'beta' })
    assert.equal(metadata(directory).version, '0.2.0-beta.1')
    assert.equal(readState().release.draft, true)
    assert.equal(readState().release.prerelease, true)
    assert.equal(readState().preparedPr.head, 'release/v0.2.0-beta.1')
    assert.equal(readState().dispatched, 'release/v0.2.0-beta.1')
  } finally {
    await new Promise(resolve => server.close(resolve))
    rmSync(directory, { recursive: true, force: true })
  }
})
