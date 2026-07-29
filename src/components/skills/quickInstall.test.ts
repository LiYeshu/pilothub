import { describe, expect, it } from 'vitest'
import {
  getQuickInstallTargetLabels,
  selectQuickInstallTools,
} from './quickInstall'

describe('getQuickInstallTargetLabels', () => {
  it('preserves detected Agent order for the install preview', () => {
    expect(
      getQuickInstallTargetLabels([
        { id: 'codex', label: 'Codex' },
        { id: 'antigravity', label: 'Antigravity' },
      ]),
    ).toEqual(['Codex', 'Antigravity'])
  })

  it('returns an empty preview when no Agent is detected', () => {
    expect(getQuickInstallTargetLabels([])).toEqual([])
  })
})

describe('selectQuickInstallTools', () => {
  it('selects supported primary Agents in a stable order', () => {
    expect(
      selectQuickInstallTools([
        { id: 'antigravity', label: 'Antigravity' },
        { id: 'cline', label: 'Cline' },
        { id: 'codex', label: 'Codex' },
      ]).map((tool) => tool.id),
    ).toEqual(['codex', 'antigravity'])
  })

  it('uses one detected fallback when no primary Agent is available', () => {
    expect(
      selectQuickInstallTools([
        { id: 'opencode', label: 'OpenCode' },
        { id: 'cline', label: 'Cline' },
      ]).map((tool) => tool.id),
    ).toEqual(['opencode'])
  })
})
