import { execFileSync, spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { appendFileSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import semver from 'semver'
import { parse } from 'smol-toml'
import { metadata, releaseIdentity, root } from './package.mjs'
import { platforms } from './platforms.mjs'

const releaseBranch = /^release\/v(\d+\.\d+\.\d+(?:-beta\.[1-9]\d*)?)$/
const cancelledLabel = 'release-cancelled'
const cancellationRequestedLabel = 'release-cancel-requested'
const registry = 'https://npm.pkg.github.com'
const versionFiles = ['package.json', 'package-lock.json', 'Cargo.toml', 'Cargo.lock']

class PublishedReleaseError extends Error {}

class GitHubRequestError extends Error {
  constructor(status, message) {
    super(message)
    this.status = status
  }
}

function command(program, args, options = {}) {
  return execFileSync(program, args, { cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], ...options }).trim()
}

function output(name, value) {
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`)
}

function summary(text) {
  console.log(text)
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${text}\n`)
}

export function resolveVersion(base, channel, current, tags) {
  if (!['stable', 'beta'].includes(channel)) throw new Error('Choose stable or beta.')
  if (semver.valid(base) !== base || semver.prerelease(base) || semver.parse(base).build.length) throw new Error('Enter a base version such as 0.2.0, without v or a suffix.')
  const released = tags.map(tag => tag.replace(/^v/, '')).filter(version => semver.valid(version))
  const stable = released.filter(version => !semver.prerelease(version))
  if (stable.some(version => semver.gte(version, base))) throw new Error('The base version must exceed every published stable version.')
  const betas = released.filter(version => version.startsWith(`${base}-beta.`))
  const sequence = Math.max(0, ...betas.map(version => Number(semver.prerelease(version)?.[1]) || 0)) + 1
  const version = channel === 'beta' ? `${base}-beta.${sequence}` : base
  if (semver.lt(version, current) || released.includes(version)) throw new Error('The requested version is older than the source or already released.')
  return version
}

export function synchronizeVersions(directory, version) {
  if (semver.valid(version) !== version) throw new Error('Invalid release version.')
  const manifest = metadata(directory)
  const lock = JSON.parse(readFileSync(join(directory, 'package-lock.json'), 'utf8'))
  const cargoFile = readFileSync(join(directory, 'Cargo.toml'), 'utf8')
  const cargoLockFile = readFileSync(join(directory, 'Cargo.lock'), 'utf8')
  const cargoName = parse(cargoFile).package.name
  const update = block => block.replace(/^version = "[^"]+"$/m, `version = "${version}"`)
  const cargo = cargoFile.replace(/^\[package\]\n[\s\S]*?(?=^\[|(?![\s\S]))/m, update)
  let matches = 0
  const cargoLock = cargoLockFile.replace(/^\[\[package\]\]\n[\s\S]*?(?=^\[\[package\]\]|(?![\s\S]))/gm, block => {
    if (parse(block).package[0].name !== cargoName || /^source = /m.test(block)) return block
    matches++
    return update(block)
  })
  if (matches !== 1 || parse(cargo).package.version !== version) throw new Error('Could not locate the Cargo package versions.')
  manifest.version = version
  lock.version = version
  lock.packages[''].version = version
  writeFileSync(join(directory, 'package.json'), `${JSON.stringify(manifest, null, 2)}\n`)
  writeFileSync(join(directory, 'package-lock.json'), `${JSON.stringify(lock, null, 2)}\n`)
  writeFileSync(join(directory, 'Cargo.toml'), cargo)
  writeFileSync(join(directory, 'Cargo.lock'), cargoLock)
}

export function validatePullRequest(pr, repository, defaultBranch, version, merged = true) {
  if (pr.head?.repo?.full_name !== repository || pr.base?.ref !== defaultBranch ||
      pr.head?.ref !== `release/v${version}` || !releaseBranch.test(pr.head.ref) ||
      (merged && !pr.merged) || pr.labels.some(label => [cancelledLabel, cancellationRequestedLabel].includes(label.name))) {
    throw new Error('Expected an active, same-repository release PR targeting the default branch.')
  }
  if (merged && !/^[a-f0-9]{40}$/.test(pr.merge_commit_sha)) throw new Error('Missing merged release commit.')
}

function archiveNames(version, packageName) {
  return [`${packageName.slice(1).replace('/', '-')}-${version}.tgz`, ...platforms.map(platform =>
    `sds-site-generator-${platform.platform}-${platform.arch}-v${version}.${platform.platform === 'win32' ? 'zip' : 'tar.gz'}`)].sort()
}

export function verifyArtifacts(directory, version, packageName) {
  const tarball = `${packageName.slice(1).replace('/', '-')}-${version}.tgz`
  const names = archiveNames(version, packageName)
  const checksums = readFileSync(join(directory, 'SHA256SUMS'), 'utf8')
  const expected = names.map(name => `${createHash('sha256').update(readFileSync(join(directory, name))).digest('hex')}  ${name}\n`).join('')
  if (checksums !== expected || readdirSync(directory).sort().join('\n') !== [...names, 'SHA256SUMS'].sort().join('\n')) {
    throw new Error('Release bundle filenames or checksums do not match the expected seven archives.')
  }
  const packed = JSON.parse(command('tar', ['-xOf', join(directory, tarball), 'package/package.json']))
  if (packed.name !== packageName || packed.version !== version) throw new Error('The tested tarball has the wrong package identity.')
  return { tarball: join(directory, tarball), names: [...names, 'SHA256SUMS'] }
}

export function verifyIntegrity(bytes, published) {
  const integrity = `sha512-${createHash('sha512').update(bytes).digest('base64')}`
  if (published?.dist?.integrity !== integrity) throw new Error('Published package bytes differ from the tested tarball; refusing to overwrite or continue.')
}

export function checkDistributionTag(tags, version, distTag) {
  const existing = tags?.[distTag]
  if (existing && (!semver.valid(existing) || semver.gt(existing, version))) throw new Error(`Refusing to move ${distTag} backward from ${existing} to ${version}.`)
}

function npmView(spec, field) {
  const result = spawnSync('npm', ['view', spec, ...(field ? [field] : []), '--json', `--registry=${registry}`], { cwd: root, encoding: 'utf8' })
  if (result.error) throw result.error
  let data
  try {
    data = JSON.parse(result.stdout || '{}')
  } catch {
    throw new Error(`Invalid npm response: ${result.stderr}`)
  }
  if (result.status !== 0) {
    if (data.error?.code === 'E404') return null
    throw new Error(`npm view failed: ${result.stderr || result.stdout}`)
  }
  return data
}

function github() {
  const repository = process.env.GITHUB_REPOSITORY
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository || '')) throw new Error('GITHUB_REPOSITORY is required.')
  const token = process.env.GH_TOKEN
  if (!token) throw new Error('GH_TOKEN is required.')
  const base = `${process.env.GITHUB_API_URL || 'https://api.github.com'}/repos/${repository}`
  async function request(path, method = 'GET', body) {
    const response = await fetch(`${base}${path}`, {
      method,
      headers: { Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28', ...(body ? { 'Content-Type': 'application/json' } : {}) },
      ...(body ? { body: JSON.stringify(body) } : {}),
    })
    if (response.status === 404 && method === 'GET') return null
    if (!response.ok) throw new GitHubRequestError(response.status, `GitHub ${method} ${path}: ${response.status} ${await response.text()}`)
    const text = await response.text()
    return text ? JSON.parse(text) : null
  }
  async function list(path) {
    const values = []
    for (let page = 1; ; page++) {
      const batch = await request(`${path}${path.includes('?') ? '&' : '?'}per_page=100&page=${page}`)
      if (!Array.isArray(batch)) throw new Error(`Expected a GitHub list: ${path}`)
      values.push(...batch)
      if (batch.length < 100) return values
    }
  }
  return { repository, request, list }
}

async function pullRequest(api, number) {
  if (!/^[1-9]\d*$/.test(String(number))) throw new Error('A release PR number is required.')
  const pr = await api.request(`/pulls/${number}`)
  if (!pr) throw new Error(`Release PR #${number} was not found.`)
  return pr
}

async function context(api, number = process.env.RELEASE_PR_NUMBER, merged = true) {
  const pr = await pullRequest(api, number)
  const version = releaseBranch.exec(pr.head.ref)?.[1]
  const repo = await api.request('')
  validatePullRequest(pr, api.repository, repo.default_branch, version, merged)
  return { pr, version, tag: `v${version}`, defaultBranch: repo.default_branch }
}

async function assertActive(api) {
  const state = await context(api)
  const manifest = metadata()
  releaseIdentity(state.tag, api.repository.split('/')[0])
  if (manifest.version !== state.version || command('git', ['rev-parse', 'HEAD']) !== state.pr.merge_commit_sha) {
    throw new Error('Release must use the exact merged commit and matching manifests.')
  }
  command('git', ['merge-base', '--is-ancestor', 'HEAD', `origin/${state.defaultBranch}`])
  await validateReleaseFiles(api, state.pr, manifest.version)
  return state
}

async function validateReleaseFiles(api, pr, version) {
  const changed = await api.list(`/pulls/${pr.number}/files`)
  if (changed.some(file => !versionFiles.includes(file.filename))) throw new Error('Release PRs may change only the four version files.')
  const lock = JSON.parse(readFileSync(join(root, 'package-lock.json'), 'utf8'))
  const cargoLock = parse(readFileSync(join(root, 'Cargo.lock'), 'utf8'))
  if (lock.version !== version || lock.packages[''].version !== version ||
      cargoLock.package.find(pkg => pkg.name === 'sds-site-generator' && !pkg.source)?.version !== version) {
    throw new Error('Both lockfiles must match the release version.')
  }
}

async function checkPr(api) {
  const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'))
  let pr = event.pull_request
  if (!pr && process.env.GITHUB_REF_NAME?.startsWith('release/v')) {
    const prs = await api.list(`/pulls?state=open&head=${api.repository.split('/')[0]}:${process.env.GITHUB_REF_NAME}`)
    pr = prs[0]
  }
  if (!pr || !releaseBranch.test(pr.head.ref)) return
  const repo = await api.request('')
  const version = metadata().version
  validatePullRequest(pr, api.repository, repo.default_branch, version, false)
  await validateReleaseFiles(api, pr, version)
}

async function tagCommit(api, tag) {
  const reference = await api.request(`/git/ref/tags/${tag}`)
  if (!reference) return null
  let object = reference.object
  for (let depth = 0; object.type === 'tag' && depth < 5; depth++) object = (await api.request(`/git/tags/${object.sha}`)).object
  if (object.type !== 'commit') throw new Error('Release tag does not resolve to a commit.')
  return object.sha
}

async function releaseState(api, tag) {
  const releases = await api.list('/releases')
  const matching = releases.filter(release => release.tag_name === tag)
  if (matching.length > 1) throw new Error(`Multiple GitHub releases exist for ${tag}.`)
  return matching[0] || null
}

async function assertUnpublished(api, version, name) {
  const release = await releaseState(api, `v${version}`)
  if ((release && !release.draft) || await tagCommit(api, `v${version}`) || npmView(`${name}@${version}`)) {
    throw new PublishedReleaseError('This release has published artifacts and cannot be discarded or cancelled. Resume publication instead.')
  }
  return release
}

async function assertPreviousComplete(api, name) {
  const prs = (await api.list('/pulls?state=all')).filter(pr => pr.head.repo?.full_name === api.repository && releaseBranch.test(pr.head.ref))
  const draft = (await api.list('/releases')).find(release => release.draft && releaseBranch.test(`release/${release.tag_name}`))
  if (draft) throw new Error(`Finish or discard draft ${draft.tag_name} before preparing another release.`)
  const open = prs.find(pr => pr.state === 'open')
  if (open) throw new Error(`Release PR #${open.number} is already open.`)
  const references = await api.request('/git/matching-refs/heads/release/')
  if (!Array.isArray(references)) throw new Error('Could not inspect release preparation branches.')
  const partial = references.find(reference => {
    const branch = reference.ref.replace(/^refs\/heads\//, '')
    return releaseBranch.test(branch) && !prs.some(pr => pr.head.ref === branch && pr.merged_at)
  })
  if (partial) throw new Error(`Discard partial preparation ${partial.ref} before preparing another release.`)
  const distributionTags = npmView(name, 'dist-tags')
  for (const candidate of prs) {
    const pr = await pullRequest(api, candidate.number)
    if (!releaseBranch.test(pr.head.ref)) continue
    if (pr.state === 'open') throw new Error(`Release PR #${pr.number} is already open.`)
    if (!pr.merged) continue
    if (pr.labels.some(label => label.name === cancellationRequestedLabel)) throw new Error(`Finish recovery for release #${pr.number} first.`)
    if (pr.labels.some(label => label.name === cancelledLabel)) {
      const recoveries = await api.list(`/pulls?state=closed&head=${api.repository.split('/')[0]}:revert-release/${pr.number}`)
      if (!recoveries.some(recovery => recovery.merged_at && recovery.head.repo?.full_name === api.repository && recovery.base.ref === pr.base.ref)) throw new Error(`Merge the recovery PR for cancelled release #${pr.number} first.`)
      continue
    }
    const version = releaseBranch.exec(pr.head.ref)[1]
    const channel = semver.prerelease(version) ? 'beta' : 'latest'
    const release = await releaseState(api, `v${version}`)
    const assets = release ? await api.list(`/releases/${release.id}/assets`) : []
    const expectedAssets = [...archiveNames(version, name), 'SHA256SUMS'].sort()
    if (!release || release.draft || release.prerelease !== (channel === 'beta') ||
        await tagCommit(api, `v${version}`) !== pr.merge_commit_sha ||
        !npmView(`${name}@${version}`) || assets.map(asset => asset.name).sort().join('\n') !== expectedAssets.join('\n') ||
        !semver.valid(distributionTags?.[channel]) || semver.lt(distributionTags[channel], version)) {
      throw new Error(`Finish release #${pr.number} before preparing another.`)
    }
  }
}

async function prepare(api) {
  const repo = await api.request('')
  if (process.env.GITHUB_REF !== `refs/heads/${repo.default_branch}`) throw new Error('Run preparation from the default branch.')
  const manifest = metadata()
  await assertPreviousComplete(api, manifest.name)
  const tags = (await api.list('/tags')).map(tag => tag.name)
  const version = resolveVersion(process.env.BASE_VERSION, process.env.RELEASE_CHANNEL, manifest.version, tags)
  const tag = `v${version}`
  const branch = `release/${tag}`
  summary(`Preparing **${tag}**. If preparation fails, use **Recovery - Discard or Cancel Release** with **discard** and version **${version}** before retrying.`)
  const previous = await api.list(`/pulls?state=closed&head=${api.repository.split('/')[0]}:${branch}`)
  if (previous.some(pr => pr.merged_at)) throw new Error('This version already belonged to a merged release. Choose a newer version.')
  if (await api.request(`/git/ref/heads/${branch}`) || await releaseState(api, tag) || npmView(`${manifest.name}@${version}`)) {
    throw new Error('This version already has release preparation or publication state. Recover that release first.')
  }
  synchronizeVersions(root, version)
  releaseIdentity(tag, api.repository.split('/')[0])
  command('git', ['switch', '--create', branch])
  command('git', ['config', 'user.name', 'github-actions[bot]'])
  command('git', ['config', 'user.email', '41898282+github-actions[bot]@users.noreply.github.com'])
  command('git', ['add', ...versionFiles])
  command('git', ['commit', '--allow-empty', '-m', `release: prepare ${tag}`])
  command('git', ['push', '--set-upstream', 'origin', branch])
  const notes = await api.request('/releases/generate-notes', 'POST', { tag_name: tag, target_commitish: branch })
  const release = await api.request('/releases', 'POST', {
    tag_name: tag, target_commitish: repo.default_branch, name: `SDS Site Generator ${tag}`,
    body: notes.body, draft: true, prerelease: process.env.RELEASE_CHANNEL === 'beta',
  })
  if (!await api.request('/labels/release')) await api.request('/labels', 'POST', { name: 'release', color: '5319e7', description: 'Release preparation' })
  const pr = await api.request('/pulls', 'POST', {
    title: `Release ${tag}`, head: branch, base: repo.default_branch,
    body: `## Release ${tag}\n\nReview the four version files and [draft release notes](${release.html_url}). Do not publish the draft or create a tag yourself.\n\nWait for **Build and test**, obtain review, and merge normally. Publication and stable documentation deployment follow automatically. No local release commands are needed.\n\nIf publication fails, rerun failed jobs in **Automatic - Publish Release**. Do not rerun successful build jobs or run npm publish locally.`,
  })
  await api.request(`/issues/${pr.number}/labels`, 'POST', { labels: ['release'] })
  // Explicit dispatch avoids depending on events suppressed for GITHUB_TOKEN writes.
  await api.request('/actions/workflows/check.yml/dispatches', 'POST', { ref: branch })
  summary(`### Prepared ${tag}\n- Review and merge: ${pr.html_url}\n- Draft notes: ${release.html_url}`)
}

async function validate(api) {
  const state = await assertActive(api)
  output('version', state.version)
  output('sha', state.pr.merge_commit_sha)
  output('dist-tag', releaseIdentity(state.tag, api.repository.split('/')[0]).distTag)
}

async function assetBytes(api, asset) {
  const response = await fetch(asset.url, { headers: { Authorization: `Bearer ${process.env.GH_TOKEN}`, Accept: 'application/octet-stream' } })
  if (!response.ok) throw new Error(`Cannot verify release asset ${asset.name}: ${response.status}`)
  return Buffer.from(await response.arrayBuffer())
}

async function publish(api) {
  const state = await assertActive(api)
  const manifest = metadata()
  const { distTag } = releaseIdentity(state.tag, api.repository.split('/')[0])
  const directory = join(root, 'release-artifacts')
  const bundle = verifyArtifacts(directory, state.version, manifest.name)
  const tarballBytes = readFileSync(bundle.tarball)
  const existingPackage = npmView(`${manifest.name}@${state.version}`)
  if (existingPackage) verifyIntegrity(tarballBytes, existingPackage)
  checkDistributionTag(npmView(manifest.name, 'dist-tags'), state.version, distTag)
  let release = await releaseState(api, state.tag)
  if (!release || release.prerelease !== (distTag === 'beta')) throw new Error('Expected the prepared draft/release with the correct channel.')
  const tagged = await tagCommit(api, state.tag)
  if ((tagged && tagged !== state.pr.merge_commit_sha) || (!release.draft && !tagged)) throw new Error('Release tag does not match the reviewed merged commit.')
  const expectedTarget = [state.defaultBranch, state.pr.merge_commit_sha]
  if (release.draft && !expectedTarget.includes(release.target_commitish)) throw new Error('Draft release target was changed outside automation.')
  const assets = await api.list(`/releases/${release.id}/assets`)
  if (assets.some(asset => !bundle.names.includes(asset.name))) throw new Error('Release contains unexpected assets.')
  for (const name of bundle.names) {
    const asset = assets.find(asset => asset.name === name)
    if (asset) {
      if (!readFileSync(join(directory, name)).equals(await assetBytes(api, asset))) throw new Error(`Existing asset ${name} differs from the tested artifact.`)
    } else {
      if (!release.draft) throw new Error(`Published release is missing ${name}; refusing to mutate it.`)
      await assertActive(api)
      command('gh', ['release', 'upload', state.tag, join(directory, name), '--repo', api.repository])
    }
  }
  if (!existingPackage) command('npm', ['publish', bundle.tarball, '--ignore-scripts', '--dry-run', '--tag', distTag, `--registry=${registry}`])
  await assertActive(api)
  const newer = (await api.list('/tags')).map(tag => tag.name.slice(1)).filter(version => semver.valid(version) && (distTag === 'beta' || !semver.prerelease(version)))
  if (newer.some(version => semver.gt(version, state.version))) throw new Error('A newer release exists; refusing stale publication.')
  if (release.draft) {
    const current = await releaseState(api, state.tag)
    if (!current.draft || current.prerelease !== release.prerelease) throw new Error('Draft release state changed during publication.')
    await api.request(`/releases/${release.id}`, 'PATCH', { target_commitish: state.pr.merge_commit_sha, draft: false, make_latest: distTag === 'latest' ? 'true' : 'false' })
  }
  if (await tagCommit(api, state.tag) !== state.pr.merge_commit_sha) throw new Error('Published GitHub tag has the wrong commit.')
  await assertActive(api)
  checkDistributionTag(npmView(manifest.name, 'dist-tags'), state.version, distTag)
  if (!existingPackage) command('npm', ['publish', bundle.tarball, '--ignore-scripts', '--tag', distTag, `--registry=${registry}`])
  else command('npm', ['dist-tag', 'add', `${manifest.name}@${state.version}`, distTag, `--registry=${registry}`])
  verifyIntegrity(tarballBytes, npmView(`${manifest.name}@${state.version}`))
  if (npmView(manifest.name, 'dist-tags')?.[distTag] !== state.version) throw new Error('npm distribution tag verification failed.')
  release = await releaseState(api, state.tag)
  if (release.draft) throw new Error('GitHub release is still a draft.')
  const finalAssets = await api.list(`/releases/${release.id}/assets`)
  if (finalAssets.length !== bundle.names.length) throw new Error('Published release asset count is wrong.')
  for (const name of bundle.names) {
    const asset = finalAssets.find(asset => asset.name === name)
    if (!asset || !readFileSync(join(directory, name)).equals(await assetBytes(api, asset))) throw new Error(`Published asset verification failed: ${name}`)
  }
  output('dist-tag', distTag)
  output('tag', state.tag)
  output('sha', state.pr.merge_commit_sha)
  writeFileSync('pages-release.json', `${JSON.stringify({ tag: state.tag, sha: state.pr.merge_commit_sha })}\n`)
  summary(`### Published ${state.tag}\n- ${release.html_url}\n- npm: \`${manifest.name}@${state.version}\` (${distTag})`)
}

async function pages(api) {
  let identity
  if (process.env.PAGES_TAG) identity = { tag: process.env.PAGES_TAG }
  else identity = JSON.parse(readFileSync('pages-release.json', 'utf8'))
  const version = identity.tag?.replace(/^v/, '')
  if (semver.valid(version) !== version || semver.prerelease(version)) throw new Error('Pages requires a published stable tag.')
  const release = await releaseState(api, identity.tag)
  if (!release || release.draft || release.prerelease) throw new Error('Pages requires a published stable GitHub release.')
  const sha = await tagCommit(api, identity.tag)
  if (!sha || (identity.sha && identity.sha !== sha)) throw new Error('Pages release commit does not match its tag.')
  const manifest = metadata()
  if (npmView(manifest.name, 'dist-tags')?.latest !== version) {
    output('deploy', 'false')
    summary(`Skipping ${identity.tag}: it is no longer npm latest.`)
    return
  }
  const checksumAsset = (await api.list(`/releases/${release.id}/assets`)).find(asset => asset.name === 'SHA256SUMS')
  if (!checksumAsset) throw new Error('Published release has no checksums.')
  const checksum = (await assetBytes(api, checksumAsset)).toString('utf8')
  const tarballName = `${manifest.name.slice(1).replace('/', '-')}-${version}.tgz`
  const tarball = (await api.list(`/releases/${release.id}/assets`)).find(asset => asset.name === tarballName)
  if (!tarball) throw new Error('Published release has no npm tarball.')
  const bytes = await assetBytes(api, tarball)
  if (!checksum.split('\n').includes(`${createHash('sha256').update(bytes).digest('hex')}  ${tarballName}`)) throw new Error('Published package checksum mismatch.')
  verifyIntegrity(bytes, npmView(`${manifest.name}@${version}`))
  writeFileSync('pages-release.json', `${JSON.stringify({ tag: identity.tag, sha })}\n`)
  output('deploy', 'true')
  output('tag', identity.tag)
  output('sha', sha)
}

async function recoveryState(api, allowCancelled = false) {
  const version = process.env.RELEASE_VERSION
  if (!releaseBranch.test(`release/v${version}`)) throw new Error('Enter the exact prepared release version.')
  const repo = await api.request('')
  if (process.env.GITHUB_REF !== `refs/heads/${repo.default_branch}`) throw new Error('Run recovery from the default branch.')
  const branch = `release/v${version}`
  const prs = await api.list(`/pulls?state=all&head=${api.repository.split('/')[0]}:${branch}`)
  const reference = await api.request(`/git/ref/heads/${branch}`)
  const candidate = prs.find(pr => pr.merged_at || !reference || pr.head.sha === reference.object.sha)
  const pr = candidate ? await pullRequest(api, candidate.number) : {
    head: { ref: branch, sha: reference?.object.sha, repo: { full_name: api.repository } },
    base: { ref: repo.default_branch }, merged: false, state: 'closed', labels: [],
  }
  const checked = allowCancelled ? { ...pr, labels: pr.labels.filter(label => ![cancelledLabel, cancellationRequestedLabel].includes(label.name)) } : pr
  validatePullRequest(checked, api.repository, repo.default_branch, version, false)
  if (version !== process.env.RELEASE_CONFIRMATION) throw new Error('Retype the exact prepared version to confirm recovery.')
  if (!['discard', 'cancel'].includes(process.env.RECOVERY_MODE)) throw new Error('Choose discard or cancel.')
  if (Boolean(pr.merged) !== (process.env.RECOVERY_MODE === 'cancel')) throw new Error('Use discard before merge, or cancel after merge.')
  return { pr, version }
}

async function stop(api) {
  const { pr, version } = await recoveryState(api, true)
  if (!pr.labels.some(label => label.name === cancellationRequestedLabel)) await assertUnpublished(api, version, metadata().name)
  if (!pr.merged) return
  if (!await api.request(`/labels/${cancellationRequestedLabel}`)) await api.request('/labels', 'POST', { name: cancellationRequestedLabel, color: 'fbca04', description: 'Publication paused for cancellation' })
  await api.request(`/issues/${pr.number}/labels`, 'POST', { labels: [cancellationRequestedLabel] })
  const requested = new Set()
  for (let attempt = 0; attempt < 60; attempt++) {
    const active = []
    for (let page = 1; ; page++) {
      const runs = await api.request(`/actions/workflows/release.yml/runs?per_page=100&page=${page}`)
      active.push(...runs.workflow_runs.filter(run => run.display_title === `Publish release PR #${pr.number}` && run.status !== 'completed'))
      if (runs.workflow_runs.length < 100) break
    }
    if (!active.length) return
    for (const run of active) {
      if (requested.has(run.id)) continue
      try {
        await api.request(`/actions/runs/${run.id}/cancel`, 'POST')
      } catch (error) {
        if (!(error instanceof GitHubRequestError) || error.status !== 409 ||
            (await api.request(`/actions/runs/${run.id}`))?.status !== 'completed') throw error
      }
      requested.add(run.id)
    }
    await new Promise(resolve => setTimeout(resolve, 5000))
  }
  throw new Error('Publication did not stop in time. Cancellation marker remains; retry recovery.')
}

async function recover(api) {
  const { pr, version } = await recoveryState(api, true)
  let release
  try {
    release = await assertUnpublished(api, version, metadata().name)
  } catch (error) {
    if (error instanceof PublishedReleaseError && pr.labels.some(label => label.name === cancellationRequestedLabel)) {
      await api.request(`/issues/${pr.number}/labels/${cancellationRequestedLabel}`, 'DELETE')
    }
    throw error
  }
  if (pr.merged) {
    if (!pr.labels.some(label => [cancelledLabel, cancellationRequestedLabel].includes(label.name))) throw new Error('Publication must be blocked before recovery.')
    if (!await api.request(`/labels/${cancelledLabel}`)) await api.request('/labels', 'POST', { name: cancelledLabel, color: 'b60205', description: 'Publication permanently blocked' })
    await api.request(`/issues/${pr.number}/labels`, 'POST', { labels: [cancelledLabel] })
    if (pr.labels.some(label => label.name === cancellationRequestedLabel)) await api.request(`/issues/${pr.number}/labels/${cancellationRequestedLabel}`, 'DELETE')
  }
  if (!pr.merged && pr.number && pr.state !== 'closed') await api.request(`/pulls/${pr.number}`, 'PATCH', { state: 'closed' })
  const refreshed = pr.number ? await pullRequest(api, pr.number) : pr
  if (!pr.merged && refreshed.merged) throw new Error('Release was merged during discard; use cancel instead.')
  if (release) await api.request(`/releases/${release.id}`, 'DELETE')
  if (!pr.merged) {
    const ref = await api.request(`/git/ref/heads/${pr.head.ref}`)
    if (ref) {
      if (ref.object.sha !== pr.head.sha) throw new Error('Release branch changed; refusing to delete it.')
      command('git', ['fetch', 'origin', pr.head.ref])
      const manifest = JSON.parse(command('git', ['show', `${ref.object.sha}:package.json`]))
      const changed = command('git', ['diff', '--name-only', `origin/${pr.base.ref}...${ref.object.sha}`]).split('\n').filter(Boolean)
      if (manifest.version !== version || changed.some(file => !versionFiles.includes(file))) {
        throw new Error('Release branch contains unexpected changes; refusing to delete it.')
      }
      await api.request(`/git/refs/heads/${pr.head.ref}`, 'DELETE')
    }
    summary(`Discarded release v${version}.`)
    return
  }
  const branch = `revert-release/${pr.number}`
  const existing = await api.list(`/pulls?state=all&head=${api.repository.split('/')[0]}:${branch}`)
  if (existing.length) {
    const recovery = existing[0]
    if (recovery.head.repo?.full_name !== api.repository || recovery.head.ref !== branch || recovery.base.ref !== pr.base.ref) {
      throw new Error('Existing recovery PR does not match the cancelled release repository and base.')
    }
    if (!recovery.merged_at && recovery.state === 'closed') await api.request(`/pulls/${recovery.number}`, 'PATCH', { state: 'open' })
    if (!recovery.merged_at) await api.request('/actions/workflows/check.yml/dispatches', 'POST', { ref: branch })
    summary(`Publication is blocked. Review recovery: ${recovery.html_url}`)
    return
  }
  const remoteBranch = await api.request(`/git/ref/heads/${branch}`)
  if (!remoteBranch) {
    command('git', ['switch', '--create', branch, `origin/${pr.base.ref}`])
    command('git', ['merge-base', '--is-ancestor', pr.merge_commit_sha, 'HEAD'])
    const parents = command('git', ['rev-list', '--parents', '-n', '1', pr.merge_commit_sha]).split(' ').length - 1
    command('git', ['revert', '--no-commit', ...(parents > 1 ? ['-m', '1'] : []), pr.merge_commit_sha])
    const files = command('git', ['diff', '--cached', '--name-only']).split('\n').filter(Boolean)
    if (files.some(file => !versionFiles.includes(file))) throw new Error('Recovery would change more than release metadata; review and revert manually.')
    command('git', ['config', 'user.name', 'github-actions[bot]'])
    command('git', ['config', 'user.email', '41898282+github-actions[bot]@users.noreply.github.com'])
    command('git', ['commit', '--allow-empty', '-m', `release: revert cancelled v${version}`, '-m', `Reverts release PR #${pr.number} (${pr.merge_commit_sha})`])
    command('git', ['push', 'origin', branch])
  } else {
    command('git', ['fetch', 'origin', branch])
    const message = command('git', ['show', '-s', '--format=%B', remoteBranch.object.sha])
    if (!message.includes(`Reverts release PR #${pr.number} (${pr.merge_commit_sha})`)) throw new Error('Existing recovery branch does not match the cancelled release.')
  }
  const recovery = await api.request('/pulls', 'POST', {
    title: `Revert cancelled release v${version}`, head: branch, base: pr.base.ref,
    body: `Publication of #${pr.number} is permanently blocked by the ${cancelledLabel} label. This reverts only release preparation. Review and merge before preparing a newer release. Do not remove the cancellation label or rerun the old publication workflow.`,
  })
  await api.request('/actions/workflows/check.yml/dispatches', 'POST', { ref: branch })
  summary(`Publication is blocked. Review and merge: ${recovery.html_url}`)
}

async function main() {
  const api = github()
  const commands = { prepare, validate, 'check-pr': checkPr, publish, pages, stop, recover }
  const action = commands[process.argv[2]]
  if (!action) throw new Error('Usage: release.mjs prepare | validate | publish | pages | stop | recover')
  await action(api)
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => {
    console.error(error.message)
    process.exitCode = 1
  })
}
