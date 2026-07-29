import type { ToolOption } from './types'

export type InstallSuccessState = {
  skillId: string
  skillName: string
  targetLabels: string[]
}

const PREFERRED_AGENT_IDS = ['claude_code', 'codex', 'antigravity']

export const selectQuickInstallTools = (tools: ToolOption[]) => {
  const preferred = PREFERRED_AGENT_IDS.flatMap((id) => {
    const tool = tools.find((candidate) => candidate.id === id)
    return tool ? [tool] : []
  })
  return preferred.length > 0 ? preferred : tools.slice(0, 1)
}

export const getQuickInstallTargetLabels = (tools: ToolOption[]) =>
  tools.map((tool) => tool.label)
