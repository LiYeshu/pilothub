import type { Extension, ManagedSkill } from './types'

export type ExtensionSkillItem = {
  componentId: string
  skill: ManagedSkill
}

export type ExtensionStatusSummary = {
  healthy: boolean
  enabledSkills: number
  toolIds: string[]
  globalTargets: number
  projectTargets: number
}

export const resolveExtensionSkills = (
  extension: Extension,
  managedSkills: ManagedSkill[],
): ExtensionSkillItem[] => {
  const skillsById = new Map(managedSkills.map((skill) => [skill.id, skill]))
  return extension.components.flatMap((component) => {
    const skill = skillsById.get(component.skill_id)
    return skill ? [{ componentId: component.id, skill }] : []
  })
}

export const summarizeExtensionStatus = (
  extension: Extension,
  managedSkills: ManagedSkill[],
): ExtensionStatusSummary => {
  const items = resolveExtensionSkills(extension, managedSkills)
  const targets = items.flatMap(({ skill }) =>
    skill.targets.filter((target) => target.status !== 'disabled'),
  )
  return {
    healthy:
      items.length > 0 &&
      items.every(
        ({ skill }) =>
          skill.enabled &&
          skill.status === 'ok' &&
          skill.targets.every(
            (target) => target.status === 'ok' || target.status === 'disabled',
          ),
      ),
    enabledSkills: items.filter(({ skill }) => skill.enabled).length,
    toolIds: Array.from(new Set(targets.map((target) => target.tool))).sort(),
    globalTargets: targets.filter((target) => target.scope === 'global').length,
    projectTargets: targets.filter((target) => target.scope === 'project').length,
  }
}
