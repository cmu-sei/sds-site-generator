import { join } from 'node:path'

export const platforms = [
  { platform: 'darwin', arch: 'arm64', target: 'aarch64-apple-darwin', runner: 'macos-15', executable: 'sds-site-generator' },
  { platform: 'darwin', arch: 'x64', target: 'x86_64-apple-darwin', runner: 'macos-15-intel', executable: 'sds-site-generator' },
  { platform: 'linux', arch: 'arm64', target: 'aarch64-unknown-linux-musl', runner: 'ubuntu-24.04-arm', executable: 'sds-site-generator' },
  { platform: 'linux', arch: 'x64', target: 'x86_64-unknown-linux-musl', runner: 'ubuntu-24.04', executable: 'sds-site-generator' },
  { platform: 'win32', arch: 'arm64', target: 'aarch64-pc-windows-msvc', runner: 'windows-11-arm', executable: 'sds-site-generator.exe' },
  { platform: 'win32', arch: 'x64', target: 'x86_64-pc-windows-msvc', runner: 'windows-2022', executable: 'sds-site-generator.exe' },
]

export function selectPlatform(platform = process.platform, arch = process.arch) {
  const selected = platforms.find(candidate => candidate.platform === platform && candidate.arch === arch)
  if (!selected) throw new Error(`Unsupported platform: ${platform}/${arch}. Supported: macOS, Linux, and Windows on x64 or ARM64.`)
  return selected
}

export function executablePath(root, selected = selectPlatform()) {
  return join(root, 'native', `${selected.platform}-${selected.arch}`, selected.executable)
}