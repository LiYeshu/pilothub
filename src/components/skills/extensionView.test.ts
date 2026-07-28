import { describe, expect, it } from 'vitest'
import type { Extension, ManagedSkill } from './types'
import { resolveExtensionSkills, summarizeExtensionStatus } from './extensionView'

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

  it('summarizes health, unique tools, and target scopes', () => {
    const extension: Extension = {
      id: 'git:example',
      name: 'example-skills',
      source: { source_type: 'git', source_ref: null },
      components: [
        { id: 'a', type: 'skill', name: 'A', skill_id: 'a' },
        { id: 'b', type: 'skill', name: 'B', skill_id: 'b' },
      ],
    }
    const first = skill('a', 'A')
    first.targets = [
      {
        tool: 'codex',
        scope: 'global',
        mode: 'symlink',
        status: 'ok',
        target_path: '/tmp/codex/a',
      },
    ]
    const second = skill('b', 'B')
    second.targets = [
      {
        tool: 'codex',
        scope: 'project',
        project_path: '/tmp/project',
        mode: 'apm',
        status: 'ok',
        target_path: '/tmp/project/.agents/skills/b',
      },
      {
        tool: 'antigravity',
        scope: 'global',
        mode: 'copy',
        status: 'ok',
        target_path: '/tmp/antigravity/b',
      },
    ]

    expect(summarizeExtensionStatus(extension, [first, second])).toEqual({
      healthy: true,
      enabledSkills: 2,
      toolIds: ['antigravity', 'codex'],
      globalTargets: 2,
      projectTargets: 1,
    })
  })

  it('reports attention when a component or target is unhealthy', () => {
    const extension: Extension = {
      id: 'git:example',
      name: 'example-skills',
      source: { source_type: 'git', source_ref: null },
      components: [{ id: 'a', type: 'skill', name: 'A', skill_id: 'a' }],
    }
    const unhealthy = skill('a', 'A')
    unhealthy.targets = [
      {
        tool: 'codex',
        scope: 'global',
        mode: 'copy',
        status: 'error',
        target_path: '/tmp/codex/a',
      },
    ]

    expect(summarizeExtensionStatus(extension, [unhealthy]).healthy).toBe(false)
  })
})
