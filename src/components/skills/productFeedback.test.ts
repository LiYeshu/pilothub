import { describe, expect, it } from 'vitest'
import { classifyInstallFailure } from './productFeedback'

describe('classifyInstallFailure', () => {
  it('maps known failures without retaining raw details', () => {
    expect(classifyInstallFailure('TOOL_NOT_WRITABLE|Codex|/private/path')).toBe(
      'permission',
    )
    expect(classifyInstallFailure('TARGET_EXISTS|/private/path')).toBe(
      'target_exists',
    )
    expect(classifyInstallFailure('GitHub connection timed out')).toBe(
      'network',
    )
    expect(classifyInstallFailure('No valid SKILL.md found')).toBe(
      'invalid_skill',
    )
  })

  it('uses a bounded fallback for unrecognized errors', () => {
    expect(classifyInstallFailure('secret internal detail')).toBe('unknown')
  })
})
