export type ProductFeedbackFailureCode =
  | 'network'
  | 'permission'
  | 'agent_unavailable'
  | 'target_exists'
  | 'invalid_skill'
  | 'duplicate'
  | 'cancelled'
  | 'unknown'

export function classifyInstallFailure(
  error: string,
): ProductFeedbackFailureCode {
  const normalized = error.toLowerCase()
  if (
    normalized.includes('cancelled|') ||
    normalized.includes('canceled')
  ) {
    return 'cancelled'
  }
  if (
    normalized.includes('tool_not_installed|') ||
    normalized.includes('tool not installed')
  ) {
    return 'agent_unavailable'
  }
  if (
    normalized.includes('permission denied') ||
    normalized.includes('access is denied') ||
    normalized.includes('tool_not_writable|')
  ) {
    return 'permission'
  }
  if (normalized.includes('target_exists|')) return 'target_exists'
  if (
    normalized.includes('skill already exists') ||
    normalized.includes('duplicate')
  ) {
    return 'duplicate'
  }
  if (
    normalized.includes('skill.md') ||
    normalized.includes('no skills found') ||
    normalized.includes('未在该仓库中发现')
  ) {
    return 'invalid_skill'
  }
  if (
    normalized.includes('github') ||
    normalized.includes('network') ||
    normalized.includes('timeout') ||
    normalized.includes('connection') ||
    normalized.includes('dns')
  ) {
    return 'network'
  }
  return 'unknown'
}
