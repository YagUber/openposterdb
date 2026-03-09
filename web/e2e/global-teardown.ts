import { execSync } from 'node:child_process'

const CONTAINER_NAME = 'openposterdb-test'

function containerCmd(): string {
  try {
    execSync('podman --version', { stdio: 'ignore' })
    return 'podman'
  } catch {
    return 'docker'
  }
}

export default async function globalTeardown() {
  const cmd = containerCmd()

  console.log('[e2e] Stopping and removing container...')
  try {
    execSync(`${cmd} rm -f ${CONTAINER_NAME}`, { stdio: 'inherit' })
  } catch {
    // may already be removed
  }
  console.log('[e2e] Cleanup complete')
}
