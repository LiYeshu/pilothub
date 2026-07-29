const ACRONYMS = new Map([
  ['ai', 'AI'],
  ['api', 'API'],
  ['cli', 'CLI'],
  ['css', 'CSS'],
  ['csv', 'CSV'],
  ['docx', 'DOCX'],
  ['html', 'HTML'],
  ['json', 'JSON'],
  ['mcp', 'MCP'],
  ['pdf', 'PDF'],
  ['pptx', 'PPTX'],
  ['sdk', 'SDK'],
  ['sql', 'SQL'],
  ['ui', 'UI'],
  ['url', 'URL'],
  ['ux', 'UX'],
  ['xml', 'XML'],
])

const MAX_PURPOSE_LENGTH = 160

export function formatSkillDisplayName(name: string): string {
  const trimmed = name.trim()
  if (!trimmed || !/[-_.]/.test(trimmed)) return trimmed

  return trimmed
    .split(/[-_.]+/)
    .filter(Boolean)
    .map((word) => {
      const acronym = ACRONYMS.get(word.toLowerCase())
      if (acronym) return acronym
      return `${word.charAt(0).toUpperCase()}${word.slice(1)}`
    })
    .join(' ')
}

export function formatSkillPurpose(
  description: string | null | undefined,
  fallback: string,
): string {
  const normalized = description
    ?.replace(/^\s*[>|][-+]?\s*/, '')
    .replace(/\s+/g, ' ')
    .trim()

  if (!normalized) return fallback
  if (normalized.length <= MAX_PURPOSE_LENGTH) return normalized

  const sentenceEnd = normalized.slice(0, MAX_PURPOSE_LENGTH + 1).search(/[.!?。！？](?:\s|$)/)
  if (sentenceEnd >= 0) return normalized.slice(0, sentenceEnd + 1)

  const shortened = normalized.slice(0, MAX_PURPOSE_LENGTH + 1)
  const lastSpace = shortened.lastIndexOf(' ')
  const end = lastSpace >= MAX_PURPOSE_LENGTH * 0.6 ? lastSpace : MAX_PURPOSE_LENGTH
  return `${normalized.slice(0, end).trimEnd()}…`
}

export function getActiveToolLabels(
  targets: { tool: string; status: string }[],
  tools: { id: string; label: string }[],
): string[] {
  const labelsById = new Map(tools.map((tool) => [tool.id, tool.label]))
  return [
    ...new Set(
      targets
        .filter((target) => target.status !== 'disabled')
        .map((target) => labelsById.get(target.tool) ?? target.tool),
    ),
  ]
}
