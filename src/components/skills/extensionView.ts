import type { Extension, ManagedSkill } from './types'

export type ExtensionSkillItem = {
  componentId: string
  skill: ManagedSkill
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
