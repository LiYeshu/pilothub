import { describe, expect, it } from 'vitest'
import {
  formatSkillDisplayName,
  formatSkillPurpose,
  getActiveToolLabels,
} from './skillPresentation'

describe('formatSkillDisplayName', () => {
  it('turns a technical identifier into a readable title', () => {
    expect(formatSkillDisplayName('baoyu-cover-image')).toBe('Baoyu Cover Image')
  })

  it('preserves common acronyms', () => {
    expect(formatSkillDisplayName('pdf-to-ai-summary')).toBe('PDF To AI Summary')
  })

  it('keeps an existing human-readable name unchanged', () => {
    expect(formatSkillDisplayName('封面图片生成助手')).toBe('封面图片生成助手')
  })
})

describe('formatSkillPurpose', () => {
  it('normalizes multiline descriptions', () => {
    expect(formatSkillPurpose('Creates\n  useful images.', 'Fallback')).toBe(
      'Creates useful images.',
    )
  })

  it('uses the supplied fallback for missing descriptions', () => {
    expect(formatSkillPurpose(null, 'Fallback')).toBe('Fallback')
  })

  it('shortens long descriptions without cutting through a word', () => {
    const description = Array.from({ length: 40 }, () => 'useful').join(' ')
    const result = formatSkillPurpose(description, 'Fallback')
    expect(result.endsWith('…')).toBe(true)
    expect(result.length).toBeLessThanOrEqual(161)
  })
})

describe('getActiveToolLabels', () => {
  it('returns unique labels for active targets only', () => {
    expect(
      getActiveToolLabels(
        [
          { tool: 'codex', status: 'synced' },
          { tool: 'codex', status: 'synced' },
          { tool: 'antigravity', status: 'disabled' },
        ],
        [
          { id: 'codex', label: 'Codex' },
          { id: 'antigravity', label: 'Antigravity' },
        ],
      ),
    ).toEqual(['Codex'])
  })
})
