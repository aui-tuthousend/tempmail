import { join, normalize } from 'node:path'

import server from './dist/server/server.js'

const host = process.env.HOST ?? '0.0.0.0'
const port = Number(process.env.PORT ?? '3000')
const clientDir = join(import.meta.dir, 'dist/client')

function clientAsset(pathname) {
  const decodedPath = decodeURIComponent(pathname)
  const normalizedPath = normalize(decodedPath).replace(/^\.\.(\/|\\|$)/, '')
  return Bun.file(join(clientDir, normalizedPath))
}

Bun.serve({
  hostname: host,
  port,
  async fetch(request) {
    const url = new URL(request.url)

    if (url.pathname.startsWith('/assets/')) {
      const file = clientAsset(url.pathname)
      if (await file.exists()) {
        return new Response(file)
      }
    }

    return server.fetch(request)
  },
})

console.info(`frontend server listening on ${host}:${port}`)
