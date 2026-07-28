import { describe, expect, it } from 'vitest'
import type { Extension, ManagedSkill } from './types'
import { resolveExtensionSkills } from './extensionView'

const skill = (id: string, name: string): ManagedSkill => ({
  id,
  name,
  description: null,
  source_type: 'git',
  source_ref: 'https://github.com/example/skills',
  central_path: `/tmp/${name}`,
  created_at: 1,
  updated_at: 1,
  last_sync_at: null,
  enabled: true,
  status: 'ok',
  tags: [],
  targets: [],
})

describe('resolveExtensionSkills', () => {
  it('keeps component order and skips stale skill references', () => {
    const extension: Extension = {
      id: 'git:example',
      name: 'example-skills',
      source: {
        source_type: 'git',
        source_ref: 'https://github.com/example/skills',
      },
      components: [
        {
          id: 'component-b',
          type: 'skill',
          name: 'B',
          skill_id: 'b',
        },
        {
          id: 'component-missing',
          type: 'skill',
          name: 'Missing',
          skill_id: 'missing',
        },
        {
          id: 'component-a',
          type: 'skill',
          name: 'A',
          skill_id: 'a',
        },
      ],
    }

    expect(resolveExtensionSkills(extension, [skill('a', 'A'), skill('b', 'B')])).toEqual([
      { componentId: 'component-b', skill: expect.objectContaining({ id: 'b' }) },
      { componentId: 'component-a', skill: expect.objectContaining({ id: 'a' }) },
    ])
  })
})
