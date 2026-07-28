import { memo } from 'react'
import {
  Boxes,
  ChevronRight,
  CircleAlert,
  CircleCheck,
  PackageOpen,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import type { Extension, ManagedSkill, ToolOption } from './types'
import {
  resolveExtensionSkills,
  summarizeExtensionStatus,
} from './extensionView'
import ToolIcon from './ToolIcon'

type ExtensionsPageProps = {
  extensions: Extension[]
  managedSkills: ManagedSkill[]
  tools: ToolOption[]
  onOpenSkill: (skill: ManagedSkill) => void
  t: TFunction
}

const ExtensionsPage = ({
  extensions,
  managedSkills,
  tools,
  onOpenSkill,
  t,
}: ExtensionsPageProps) => (
  <div className="extensions-page">
    <header className="extensions-header">
      <div>
        <h1>{t('extensions.title')}</h1>
        <p>{t('extensions.subtitle')}</p>
      </div>
      <div className="extensions-summary" aria-label={t('extensions.summary')}>
        <Boxes size={18} />
        <strong>{extensions.length}</strong>
        <span>{t('extensions.collections')}</span>
      </div>
    </header>

    {extensions.length === 0 ? (
      <div className="extensions-empty">
        <PackageOpen size={28} />
        <strong>{t('extensions.emptyTitle')}</strong>
        <span>{t('extensions.emptyBody')}</span>
      </div>
    ) : (
      <div className="extension-collection-list">
        {extensions.map((extension) => {
          const skills = resolveExtensionSkills(extension, managedSkills)
          const summary = summarizeExtensionStatus(extension, managedSkills)
          const toolById = new Map(tools.map((tool) => [tool.id, tool]))
          return (
            <section className="extension-collection" key={extension.id}>
              <div className="extension-collection-header">
                <div className="extension-identity">
                  <span className="extension-icon" aria-hidden="true">
                    <PackageOpen size={20} />
                  </span>
                  <div>
                    <h2>{extension.name}</h2>
                    <p>{extension.source.source_ref ?? extension.source.source_type}</p>
                  </div>
                </div>
                <div className="extension-header-status">
                  <span
                    className={`extension-health ${summary.healthy ? 'healthy' : 'attention'}`}
                  >
                    {summary.healthy ? (
                      <CircleCheck size={15} aria-hidden="true" />
                    ) : (
                      <CircleAlert size={15} aria-hidden="true" />
                    )}
                    {summary.healthy
                      ? t('extensions.healthy')
                      : t('extensions.attention')}
                  </span>
                  <span className="extension-count">
                    {t('extensions.skillCount', { count: skills.length })}
                  </span>
                </div>
              </div>
              <div className="extension-target-summary">
                <span className="extension-target-label">
                  {t('extensions.targets')}
                </span>
                <div className="extension-tool-list">
                  {summary.toolIds.length > 0 ? (
                    summary.toolIds.map((toolId) => {
                      const tool = toolById.get(toolId)
                      const label = tool?.label ?? toolId
                      return (
                        <span className="extension-tool-chip" key={toolId}>
                          <ToolIcon
                            toolKey={toolId}
                            label={label}
                            avatar={tool?.avatar}
                            className="extension-tool-icon"
                          />
                          {label}
                        </span>
                      )
                    })
                  ) : (
                    <span className="extension-no-targets">
                      {t('extensions.noTargets')}
                    </span>
                  )}
                </div>
                <span className="extension-scope-summary">
                  {summary.globalTargets > 0
                    ? t('extensions.globalTargets', {
                        count: summary.globalTargets,
                      })
                    : null}
                  {summary.globalTargets > 0 && summary.projectTargets > 0
                    ? ' · '
                    : null}
                  {summary.projectTargets > 0
                    ? t('extensions.projectTargets', {
                        count: summary.projectTargets,
                      })
                    : null}
                </span>
              </div>
              <div className="extension-components">
                {skills.map(({ componentId, skill }) => {
                  const activeTargets = skill.targets.filter(
                    (target) => target.status !== 'disabled',
                  )
                  const scopes = new Set(activeTargets.map((target) => target.scope))
                  return (
                    <button
                      className="extension-component"
                      type="button"
                      key={componentId}
                      onClick={() => onOpenSkill(skill)}
                    >
                      <span className="extension-component-copy">
                        <strong>{skill.name}</strong>
                        <small>
                          {skill.description || t('skillDescriptionEmpty')}
                        </small>
                      </span>
                      <span className="extension-component-meta">
                        {scopes.has('global') ? (
                          <i>{t('scope.globalBadge')}</i>
                        ) : null}
                        {scopes.has('project') ? (
                          <i>
                            {t('extensions.projectScope')}
                          </i>
                        ) : null}
                        <small>
                          {t('extensions.targetCount', {
                            count: activeTargets.length,
                          })}
                        </small>
                        <ChevronRight size={17} aria-hidden="true" />
                      </span>
                    </button>
                  )
                })}
              </div>
            </section>
          )
        })}
      </div>
    )}
  </div>
)

export default memo(ExtensionsPage)
