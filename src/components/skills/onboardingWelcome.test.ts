import { describe, expect, it } from 'vitest'
import { shouldShowWelcome } from './onboardingWelcome'

describe('shouldShowWelcome', () => {
  it('shows the welcome experience before it has been completed', () => {
    expect(shouldShowWelcome(null)).toBe(true)
    expect(shouldShowWelcome('false')).toBe(true)
  })

  it('keeps the welcome experience hidden after completion', () => {
    expect(shouldShowWelcome('true')).toBe(false)
  })
})
