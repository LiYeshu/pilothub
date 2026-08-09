import { memo } from 'react'
import {
  Boxes,
  ChevronRight,
  CircleAlert,
  CircleCheck,
  Eye,
  PackageOpen,
  Plus,
  Stethoscope,
  Trash2,
  UsersRound,
  Wrench,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import type {
  Extension,
  InstalledCodexPlugin,
  ManagedSkill,
  ToolOption,
} from './types'
import {
  resolveExtensionSkills,
  summarizeExtensionStatus,
} from './extensionView'
import ToolIcon from './ToolIcon'
import {
  formatSkillDisplayName,
  formatSkillPurpose,
} from './skillPresentation'

type ExtensionsPageProps = {
  extensions: Extension[]
  codexPlugins: InstalledCodexPlugin[]
  managedSkills: ManagedSkill[]
  tools: ToolOption[]
  onOpenSkill: (skill: ManagedSkill) => void
  onOpenPlugin: (plugin: InstalledCodexPlugin) => void
  onAddPlugin: () => void
  onDoctorPlugin: (pluginName: string) => void
  onRepairPlugin: (pluginName: string) => void
  onUninstallPlugin: (plugin: InstalledCodexPlugin) => void
  t: TFunction
}

const ExtensionsPage = ({
  extensions,
  codexPlugins,
  managedSkills,
  tools,
  onOpenSkill,
  onOpenPlugin,
  onAddPlugin,
  onDoctorPlugin,
  onRepairPlugin,
  onUninstallPlugin,
  t,
}: ExtensionsPageProps) => (
  <div className="extensions-page">
    <header className="extensions-header">
      <div>
        <h1>{t('extensions.title')}</h1>
        <p>{t('extensions.subtitle')}</p>
      </div>
      <div className="extensions-header-actions">
        <div className="extensions-summary" aria-label={t('extensions.summary')}>
          <Boxes size={18} />
          <strong>{extensions.length + codexPlugins.length}</strong>
          <span>{t('extensions.collections')}</span>
        </div>
        <button className="btn btn-primary" type="button" onClick={onAddPlugin}>
          <Plus size={16} />
          {t('plugins.add')}
        </button>
      </div>
    </header>

    {codexPlugins.length > 0 ? (
      <section className="codex-plugin-section" aria-labelledby="codex-plugins-title">
        <div className="extension-section-heading">
          <div>
            <h2 id="codex-plugins-title">{t('plugins.sectionTitle')}</h2>
            <p>{t('plugins.sectionSubtitle')}</p>
          </div>
          <span>{t('plugins.pluginCount', { count: codexPlugins.length })}</span>
        </div>
        <div className="codex-plugin-list">
          {codexPlugins.map((plugin) => {
            const isExpertTeam = plugin.descriptor.skills.length > 1
            const visibleSkills = plugin.descriptor.skills.slice(0, 4)
            const remainingSkillCount =
              plugin.descriptor.skills.length - visibleSkills.length
            return (
              <article className="codex-plugin-card" key={plugin.descriptor.name}>
                <div className="codex-plugin-header">
                  <div className="extension-identity">
                    <span className="extension-icon" aria-hidden="true">
                      {isExpertTeam ? (
                        <UsersRound size={20} />
                      ) : (
                        <PackageOpen size={20} />
                      )}
                    </span>
                    <div>
                      <span className="plugin-product-kind">
                        {isExpertTeam
                          ? t('plugins.expertTeam')
                          : t('plugins.aiCapability')}
                      </span>
                      <h3>{plugin.descriptor.display_name}</h3>
                      <p>
                        <code>{plugin.descriptor.name}</code>
                        <span>v{plugin.descriptor.version}</span>
                      </p>
                    </div>
                  </div>
                  <span
                    className={`extension-health ${
                      plugin.status.health === 'healthy'
                        ? 'healthy'
                        : 'attention'
                    }`}
                  >
                    {plugin.status.health === 'healthy' ? (
                      <CircleCheck size={15} aria-hidden="true" />
                    ) : (
                      <CircleAlert size={15} aria-hidden="true" />
                    )}
                    {plugin.status.health === 'healthy'
                      ? t('extensions.healthy')
                      : t('extensions.attention')}
                  </span>
                </div>
                <p className="codex-plugin-description">
                  {plugin.descriptor.description}
                </p>
                <div className="codex-plugin-meta">
                  <span>{t('plugins.runtime')}: Codex</span>
                  <span>
                    {isExpertTeam
                      ? t('plugins.specialistCount', {
                          count: plugin.descriptor.skills.length,
                        })
                      : t('plugins.capabilityCount', {
                          count: plugin.descriptor.skills.length,
                        })}
                  </span>
                  <span>{t('plugins.managedByPilotHub')}</span>
                  <span>
                    {plugin.status.invocation.mode === 'native'
                      ? t('plugins.nativeMode')
                      : plugin.status.invocation.mode === 'compatibility'
                        ? t('plugins.compatibilityMode')
                        : t('plugins.unavailableMode')}
                  </span>
                </div>
                <div
                  className="codex-plugin-skills"
                  aria-label={t('plugins.professionalCapabilities')}
                >
                  {visibleSkills.map((skill) => (
                    <span key={skill.relative_path}>
                      {formatSkillDisplayName(skill.name)}
                    </span>
                  ))}
                  {remainingSkillCount > 0 ? (
                    <span>
                      {t('plugins.moreCapabilities', {
                        count: remainingSkillCount,
                      })}
                    </span>
                  ) : null}
                </div>
                <div className="codex-plugin-actions">
                  <button
                    className="btn btn-primary"
                    type="button"
                    onClick={() => onOpenPlugin(plugin)}
                  >
                    <Eye size={15} />
                    {isExpertTeam
                      ? t('plugins.viewExpertTeam')
                      : t('plugins.viewCapability')}
                  </button>
                  <button
                    className="btn btn-secondary"
                    type="button"
                    onClick={() => onDoctorPlugin(plugin.descriptor.name)}
                  >
                    <Stethoscope size={15} />
                    {t('plugins.doctor')}
                  </button>
                  <button
                    className="btn btn-secondary"
                    type="button"
                    onClick={() => onRepairPlugin(plugin.descriptor.name)}
                  >
                    <Wrench size={15} />
                    {t('plugins.repair')}
                  </button>
                  <button
                    className="btn btn-secondary danger"
                    type="button"
                    onClick={() => onUninstallPlugin(plugin)}
                  >
                    <Trash2 size={15} />
                    {t('plugins.uninstall')}
                  </button>
                </div>
              </article>
            )
          })}
        </div>
      </section>
    ) : null}

    {extensions.length === 0 && codexPlugins.length === 0 ? (
      <div className="extensions-empty">
        <PackageOpen size={28} />
        <strong>{t('extensions.emptyTitle')}</strong>
        <span>{t('extensions.emptyBody')}</span>
      </div>
    ) : extensions.length > 0 ? (
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
                  const displayName = formatSkillDisplayName(skill.name)
                  const purpose = formatSkillPurpose(
                    skill.description,
                    t('skillPresentation.fallbackPurpose', { name: displayName }),
                  )
                  return (
                    <button
                      className="extension-component"
                      type="button"
                      key={componentId}
                      onClick={() => onOpenSkill(skill)}
                    >
                      <span className="extension-component-copy">
                        <strong>{displayName}</strong>
                        {displayName !== skill.name ? <code>{skill.name}</code> : null}
                        <small>{purpose}</small>
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
    ) : null}
  </div>
)

export default memo(ExtensionsPage)
