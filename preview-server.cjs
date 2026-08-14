// Minimal static file server for previewing the built dist/.
const http = require("node:http")
const fs = require("node:fs")
const path = require("node:path")

const ROOT = path.resolve(__dirname, "dist")
const PORT = Number(process.env.PORT || 4173)

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".mjs": "application/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".ico": "image/x-icon",
  ".woff": "font/woff",
  ".woff2": "font/woff2",
  ".ttf": "font/ttf",
  ".txt": "text/plain; charset=utf-8",
}

const server = http.createServer((req, res) => {
  try {
    const url = decodeURIComponent((req.url || "/").split("?")[0])
    let file = path.join(ROOT, url === "/" ? "index.html" : url)
    if (!file.startsWith(ROOT)) {
      res.statusCode = 403
      return res.end("forbidden")
    }
    if (fs.existsSync(file) && fs.statSync(file).isDirectory())
      file = path.join(file, "index.html")
    if (!fs.existsSync(file)) {
      // SPA fallback
      file = path.join(ROOT, "index.html")
    }
    const ext = path.extname(file).toLowerCase()
    res.setHeader("content-type", MIME[ext] || "application/octet-stream")
    res.end(fs.readFileSync(file))
  } catch (err) {
    res.statusCode = 500
    res.end(String(err))
  }
})

server.listen(PORT, "127.0.0.1", () => {
  console.log(`preview ready on http://127.0.0.1:${PORT}/`)
})
