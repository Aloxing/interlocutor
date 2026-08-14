import {
  AlignmentType,
  BorderStyle,
  Document,
  HeadingLevel,
  Packer,
  Paragraph,
  ShadingType,
  Table,
  TableCell,
  TableRow,
  TextRun,
  WidthType,
} from "docx"
import MarkdownIt from "markdown-it"

const md = new MarkdownIt()

function collectInline(children: readonly any[], start: number, closeType: string) {
  let text = ""
  let i = start + 1
  while (i < children.length && children[i].type !== closeType) {
    if (children[i].type === "text") {
      text += children[i].content
    }
    i += 1
  }
  return { text, next: i + 1 }
}

function inlineRuns(children: readonly any[] | null | undefined): TextRun[] {
  const runs: TextRun[] = []
  if (!children) {
    return runs
  }

  let i = 0
  while (i < children.length) {
    const token = children[i]
    if (token.type === "text") {
      runs.push(new TextRun({ text: token.content }))
      i += 1
    } else if (token.type === "code_inline") {
      runs.push(new TextRun({ text: token.content, font: "Consolas" }))
      i += 1
    } else if (token.type === "strong_open") {
      const { text, next } = collectInline(children, i, "strong_close")
      runs.push(new TextRun({ text, bold: true }))
      i = next
    } else if (token.type === "em_open") {
      const { text, next } = collectInline(children, i, "em_close")
      runs.push(new TextRun({ text, italics: true }))
      i = next
    } else if (token.type === "s_open") {
      const { text, next } = collectInline(children, i, "s_close")
      runs.push(new TextRun({ text, strike: true }))
      i = next
    } else if (token.type === "link_open") {
      const { text, next } = collectInline(children, i, "link_close")
      runs.push(new TextRun({ text, color: "0969DA" }))
      i = next
    } else {
      i += 1
    }
  }
  return runs
}

function headingLevel(level: number) {
  const levels = [
    HeadingLevel.HEADING_1,
    HeadingLevel.HEADING_2,
    HeadingLevel.HEADING_3,
    HeadingLevel.HEADING_4,
    HeadingLevel.HEADING_5,
    HeadingLevel.HEADING_6,
  ]
  return levels[level - 1] ?? HeadingLevel.HEADING_3
}

export async function markdownToDocx(source: string): Promise<Uint8Array> {
  const tokens = md.parse(source, {})
  const children: any[] = []

  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i]

    if (token.type === "heading_open") {
      const inline = tokens[i + 1]
      children.push(
        new Paragraph({
          heading: headingLevel(Number(token.tag.slice(1))),
          spacing: { before: 240, after: 120 },
          children: inlineRuns(inline?.children),
        }),
      )
      i += 2
    } else if (token.type === "paragraph_open") {
      const inline = tokens[i + 1]
      children.push(
        new Paragraph({
          spacing: { after: 120 },
          children: inlineRuns(inline?.children),
        }),
      )
      i += 2
    } else if (token.type === "bullet_list_open") {
      i += 1
      while (i < tokens.length && tokens[i].type !== "bullet_list_close") {
        if (tokens[i].type === "list_item_open") {
          const inline = tokens[i + 2]
          children.push(
            new Paragraph({
              bullet: { level: 0 },
              spacing: { after: 60 },
              children: inlineRuns(inline?.children),
            }),
          )
          i += 5
        } else {
          i += 1
        }
      }
    } else if (token.type === "ordered_list_open") {
      i += 1
      while (i < tokens.length && tokens[i].type !== "ordered_list_close") {
        if (tokens[i].type === "list_item_open") {
          const inline = tokens[i + 2]
          children.push(
            new Paragraph({
              numbering: { reference: "ordered-list", level: 0 },
              spacing: { after: 60 },
              children: inlineRuns(inline?.children),
            }),
          )
          i += 5
        } else {
          i += 1
        }
      }
    } else if (token.type === "code_block") {
      children.push(
        new Paragraph({
          spacing: { after: 120 },
          shading: { type: ShadingType.CLEAR, fill: "F6F8FA" },
          border: {
            top: { style: BorderStyle.SINGLE, size: 4, color: "D0D7DE" },
            bottom: { style: BorderStyle.SINGLE, size: 4, color: "D0D7DE" },
            left: { style: BorderStyle.SINGLE, size: 4, color: "D0D7DE" },
            right: { style: BorderStyle.SINGLE, size: 4, color: "D0D7DE" },
          },
          children: [
            new TextRun({
              text: token.content,
              font: "Consolas",
              size: 20,
            }),
          ],
        }),
      )
    } else if (token.type === "blockquote_open") {
      i += 1
      while (i < tokens.length && tokens[i].type !== "blockquote_close") {
        if (tokens[i].type === "paragraph_open") {
          const inline = tokens[i + 1]
          children.push(
            new Paragraph({
              indent: { left: 480 },
              border: {
                left: {
                  style: BorderStyle.SINGLE,
                  size: 12,
                  color: "D0D7DE",
                },
              },
              spacing: { after: 120 },
              children: inlineRuns(inline?.children),
            }),
          )
          i += 3
        } else {
          i += 1
        }
      }
    } else if (token.type === "table_open") {
      const rows: TableRow[] = []
      i += 1
      let cells: TableCell[] = []
      while (i < tokens.length && tokens[i].type !== "table_close") {
        const current = tokens[i]
        if (current.type === "th_open" || current.type === "td_open") {
          const inline = tokens[i + 1]
          cells.push(
            new TableCell({
              children: [
                new Paragraph({
                  children: inlineRuns(inline?.children),
                }),
              ],
            }),
          )
          i += 3
        } else if (current.type === "tr_close") {
          rows.push(new TableRow({ children: cells }))
          cells = []
          i += 1
        } else {
          i += 1
        }
      }
      if (rows.length > 0) {
        children.push(
          new Table({
            rows,
            width: { size: 100, type: WidthType.PERCENTAGE },
          }),
        )
      }
    } else if (token.type === "hr") {
      children.push(
        new Paragraph({
          spacing: { after: 120 },
          border: {
            bottom: { style: BorderStyle.SINGLE, size: 6, color: "999999" },
          },
        }),
      )
    }
  }

  const doc = new Document({
    numbering: {
      config: [
        {
          reference: "ordered-list",
          levels: [
            {
              level: 0,
              format: "decimal",
              text: "%1.",
              alignment: AlignmentType.START,
              style: { paragraph: { indent: { left: 480 } } },
            },
          ],
        },
      ],
    },
    sections: [{ children }],
  })

  const blob = await Packer.toBlob(doc)
  return new Uint8Array(await blob.arrayBuffer())
}
