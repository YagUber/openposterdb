import { join, extname, basename } from 'node:path'
import { existsSync, statSync, writeFileSync, unlinkSync } from 'node:fs'
import type { Plugin } from 'vite'
import { IMAGE_EXTS, IMAGE_DIRS, listImages } from './vite-plugin-image-utils'
const WEBP_QUALITY = 80

/** Max widths at 2× retina for each image category on the landing page. */
const MAX_WIDTHS: Record<string, number> = {
  'size-backdrop-': 720, // displayed at 360px
  'size-poster-': 400,   // displayed at 200px
  'size-logo-': 400,     // displayed at 200px
  'backdrop-': 720,  // displayed at 360px
  'logo-': 400,      // displayed at 200px
  'style-': 400,     // displayed at 200px
  'label-': 400,     // displayed at 200px
  'pos-': 320,       // displayed at 160px
}
const DEFAULT_MAX_WIDTH = 400 // posters displayed at 160–200px

function maxWidthFor(filename: string): number {
  for (const [prefix, width] of Object.entries(MAX_WIDTHS)) {
    if (filename.startsWith(prefix)) return width
  }
  return DEFAULT_MAX_WIDTH
}

/** Resize and convert a source image to WebP. */
async function toWebP(sourcePath: string): Promise<Buffer> {
  const sharp = (await import('sharp')).default
  return sharp(sourcePath)
    .resize({ width: maxWidthFor(basename(sourcePath)), withoutEnlargement: true })
    .webp({ quality: WEBP_QUALITY })
    .toBuffer()
}

/** Find the source jpg/png file for a .webp request. */
function findSource(dir: string, webpName: string): string | null {
  const base = webpName.replace(/\.webp$/, '')
  for (const ext of ['.jpg', '.jpeg', '.png']) {
    const candidate = join(dir, base + ext)
    try {
      statSync(candidate)
      return candidate
    } catch { /* not found */ }
  }
  return null
}

/**
 * Vite plugin that converts example images to WebP.
 * - Build: resizes, converts to WebP, removes originals in output.
 * - Dev: serves on-the-fly WebP conversion via middleware.
 */
export default function optimizeExamplesPlugin(): Plugin {
  let outDir: string
  let publicDir: string

  return {
    name: 'optimize-examples',

    configResolved(config) {
      outDir = config.build.outDir
      publicDir = config.publicDir
    },

    // Dev: serve .webp requests by converting source jpg/png on the fly (with in-memory cache)
    configureServer(server) {
      const cache = new Map<string, { mtime: number; buf: Buffer }>()

      server.middlewares.use(async (req, res, next) => {
        if (!req.url?.endsWith('.webp')) return next()

        const dir = IMAGE_DIRS.find(d => req.url!.startsWith(`/${d}/`))
        if (!dir) return next()

        const webpName = req.url!.slice(`/${dir}/`.length)
        const source = findSource(join(publicDir, dir), webpName)
        if (!source) return next()

        try {
          const mtime = statSync(source).mtimeMs
          const cached = cache.get(webpName)

          let buf: Buffer
          if (cached && cached.mtime === mtime) {
            buf = cached.buf
          } else {
            buf = await toWebP(source)
            cache.set(webpName, { mtime, buf })
          }

          res.setHeader('Content-Type', 'image/webp')
          res.setHeader('Cache-Control', 'no-cache')
          res.end(buf)
        } catch {
          next()
        }
      })
    },

    // Build: convert all images to WebP, resize, and remove originals
    async closeBundle() {
      for (const dir of IMAGE_DIRS) {
        const dirOut = join(outDir, dir)
        if (!existsSync(dirOut)) continue
        const files = listImages(dirOut)
        if (files.length === 0) continue

        let totalBefore = 0
        let totalAfter = 0

        await Promise.all(files.map(async (file) => {
          const filePath = join(dirOut, file)
          const origSize = statSync(filePath).size

          const webpName = file.replace(/\.[^.]+$/, '.webp')
          const webpPath = join(dirOut, webpName)

          const buf = await toWebP(filePath)

          writeFileSync(webpPath, buf)
          unlinkSync(filePath)

          totalBefore += origSize
          totalAfter += buf.length
          const pct = ((1 - buf.length / origSize) * 100).toFixed(0)
          console.log(`  ${file} → ${webpName}: ${formatBytes(origSize)} → ${formatBytes(buf.length)} (−${pct}%)`)
        }))

        if (totalBefore > 0) {
          const pct = ((1 - totalAfter / totalBefore) * 100).toFixed(0)
          console.log(`  ${dir} total: ${formatBytes(totalBefore)} → ${formatBytes(totalAfter)} (−${pct}%)`)
        }
      }
    },
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  return `${(bytes / 1024).toFixed(0)}KB`
}
