import { unified } from "unified"
import remarkParse from "remark-parse"
import remarkGfm from "remark-gfm"
import remarkMath from "remark-math"
import remarkRehype from "remark-rehype"
import rehypeRaw from "rehype-raw"
import rehypeHighlight from "rehype-highlight"
import rehypeKatex from "rehype-katex"
import rehypeStringify from "rehype-stringify"
import DOMPurify from "dompurify"
import hljsCss from "../styles/hljs-chatgpt.css?raw"
import hljsCssDark from "../styles/hljs-chatgpt-dark.css?raw"
import katexCss from "katex/dist/katex.min.css?raw"

const processor = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(remarkMath)
  .use(remarkRehype, { allowDangerousHtml: true })
  .use(rehypeRaw)
  .use(rehypeHighlight, { detect: false, ignoreMissing: true })
  .use(rehypeKatex)
  .use(rehypeStringify)

export function renderMarkdown(source: string): string {
  const file = processor.processSync(source)
  return DOMPurify.sanitize(String(file))
}

const lightVars = `
:root {
  --bg: #FAFBFD;
  --surface: #FFFFFF;
  --ink: #1F2937;
  --ink-soft: #1F2937;
  --muted: #64748B;
  --line: #E6EBF1;
  --line-strong: #D1D9E2;
  --md-h1: #2563EB;
  --md-h2: #4F46E5;
  --md-h3: #7C3AED;
  --md-link: #2563EB;
  --md-link-hover: #1D4ED8;
  --md-quote-border: #2563EB;
  --md-quote-bg: #F1F5F9;
  --md-quote-text: #475569;
  --md-th-bg: #F1F5F9;
  --code-bg: #F6F8FA;
  --code-border: #E6EBF1;
  --inline-code-bg: rgba(15, 23, 42, 0.06);
  --inline-code-color: #1F2937;
  --selection: rgba(37, 99, 235, 0.18);
  color-scheme: light;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #0B1020;
    --surface: #111827;
    --ink: #E5E7EB;
    --ink-soft: #F1F5F9;
    --muted: #94A3B8;
    --line: #263244;
    --line-strong: #334155;
    --md-h1: #60A5FA;
    --md-h2: #818CF8;
    --md-h3: #A78BFA;
    --md-link: #60A5FA;
    --md-link-hover: #93C5FD;
    --md-quote-border: #60A5FA;
    --md-quote-bg: #1E293B;
    --md-quote-text: #CBD5E1;
    --md-th-bg: #1E293B;
    --code-bg: #0F172A;
    --code-border: #263244;
    --inline-code-bg: rgba(255, 255, 255, 0.08);
    --inline-code-color: #E5E7EB;
    --selection: rgba(96, 165, 250, 0.30);
    color-scheme: dark;
  }
}
`

const markdownCss = `
* { box-sizing: border-box; }
body { margin: 0; padding: 0; background: var(--bg); }
::selection { background: var(--selection); color: inherit; }
.md-body { color: var(--ink); font-size: 0.95rem; line-height: 1.75; word-break: break-word; max-width: 820px; margin: 0 auto; padding: 32px 28px; }
.md-body > :first-child { margin-top: 0; }
.md-body h1, .md-body h2, .md-body h3, .md-body h4, .md-body h5, .md-body h6 { font-weight: 600; line-height: 1.35; margin: 1.5em 0 0.6em; letter-spacing: -0.01em; }
.md-body h1 { font-size: 1.85em; color: var(--md-h1); }
.md-body h2 { font-size: 1.45em; color: var(--md-h2); }
.md-body h3 { font-size: 1.2em; color: var(--md-h3); }
.md-body h4, .md-body h5, .md-body h6 { font-size: 1.05em; color: var(--ink-soft); }
.md-body p, .md-body ul, .md-body ol { margin: 0.7em 0; }
.md-body ul, .md-body ol { padding-left: 1.5em; }
.md-body li { margin: 0.25em 0; }
.md-body li::marker { color: var(--muted); }
.md-body a { color: var(--md-link); text-decoration: underline; text-underline-offset: 3px; text-decoration-thickness: 1px; text-decoration-color: color-mix(in srgb, var(--md-link) 50%, transparent); transition: color 0.15s ease, text-decoration-color 0.15s ease; }
.md-body a:hover { color: var(--md-link-hover); text-decoration-color: var(--md-link-hover); }
.md-body blockquote { border-left: 3px solid var(--md-quote-border); background: var(--md-quote-bg); color: var(--md-quote-text); margin: 1em 0; padding: 0.6em 1em; border-radius: 0 6px 6px 0; }
.md-body code { background: var(--inline-code-bg); color: var(--inline-code-color); border-radius: 5px; font-family: ui-monospace, SFMono-Regular, "JetBrains Mono", Consolas, monospace; font-size: 0.85em; padding: 0.15em 0.4em; font-weight: 500; }
.md-body pre { background: var(--code-bg); border: 1px solid var(--code-border); border-radius: 12px; margin: 1em 0; overflow: hidden; padding: 0; }
.md-body pre code { background: transparent; border-radius: 0; display: block; font-size: 0.85rem; line-height: 1.65; padding: 1em 1.1em; font-weight: 400; color: var(--ink); overflow-x: auto; }
.md-body table { border-collapse: collapse; display: block; margin: 1em 0; max-width: 100%; overflow-x: auto; width: max-content; border: 1px solid var(--line); border-radius: 8px; }
.md-body th, .md-body td { border-bottom: 1px solid var(--line); padding: 0.5em 0.9em; }
.md-body tr:last-child td { border-bottom: 0; }
.md-body th { background: var(--md-th-bg); color: var(--ink-soft); font-weight: 600; }
.md-body hr { border: 0; border-top: 1px solid var(--line); margin: 1.8em 0; }
.md-body img { max-width: 100%; border-radius: 8px; }
.hljs { background: transparent !important; }
`

export function buildHtmlDocument(source: string): string {
  const body = DOMPurify.sanitize(String(processor.processSync(source)))
  return `<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Markdown Export</title>
  <style>${lightVars}\n${markdownCss}\n${hljsCss}\n${hljsCssDark}\n${katexCss}</style>
</head>
<body>
  <article class="md-body">${body}</article>
</body>
</html>`
}
