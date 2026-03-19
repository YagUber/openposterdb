import { readdirSync, realpathSync } from 'node:fs'
import { join, extname, relative } from 'node:path'

export const IMAGE_EXTS = new Set(['.jpg', '.jpeg', '.png'])
export const IMAGE_DIRS = ['examples', 'icons'] as const

/** Recursively list all image files in a directory, returning paths relative to `dir`. */
export function listImages(dir: string): string[] {
  const results: string[] = []
  const seen = new Set<string>()
  function walk(current: string) {
    const real = realpathSync(current)
    if (seen.has(real)) return
    seen.add(real)
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      const full = join(current, entry.name)
      if (entry.isDirectory() || entry.isSymbolicLink()) {
        walk(full)
      } else if (IMAGE_EXTS.has(extname(entry.name).toLowerCase())) {
        results.push(relative(dir, full))
      }
    }
  }
  walk(dir)
  return results
}
