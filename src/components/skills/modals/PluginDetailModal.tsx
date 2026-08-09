import { memo } from 'react'
import {
  Bot,
  CircleAlert,
  CircleCheck,
  Code2,
  MessageSquareText,
  PackageOpen,
  UsersRound,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import {
  formatSkillDisplayName,
  formatSkillPurpose,
} from '../skillPresentation'
import type { InstalledCodexPlugin } from '../types'

type PluginDetailModalProps = {
  plugin: InstalledCodexPlugin | null
  onRequestClose: () => void
  t: TFunction
}

const PluginDetailModal = ({
  plugin,
  onRequestClose,
  t,
}: PluginDetailModalProps) => {
  if (!plugin) return null

  const { descriptor, status } = plugin
  const isExpertTeam = descriptor.skills.length > 1
  const healthy = status.health === 'healthy'
  const prompts =
    descriptor.default_prompts.length > 0
      ? descriptor.default_prompts
      : [t('plugins.usageFallback', { name: descriptor.display_name })]

  return (
    <div className="modal-backdrop" onClick={onRequestClose}>
      <div
        className="modal plugin-detail-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="plugin-detail-title"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal-header">
          <div className="modal-title" id="plugin-detail-title">
            {isExpertTeam
              ? t('plugins.expertTeamDetails')
              : t('plugins.capabilityDetails')}
          </div>
          <button
            className="modal-close"
            type="button"
            onClick={onRequestClose}
            aria-label={t('close')}
          >
            ✕
          </button>
        </header>

        <div className="modal-body plugin-detail-body">
          <section className="plugin-detail-hero">
            <span className="plugin-detail-icon" aria-hidden="true">
              {isExpertTeam ? <UsersRound size={24} /> : <PackageOpen size={24} />}
            </span>
            <div className="plugin-detail-identity">
              <span className="plugin-product-kind">
                {isExpertTeam
                  ? t('plugins.expertTeam')
                  : t('plugins.aiCapability')}
              </span>
              <h2>{descriptor.display_name}</h2>
              <p>{descriptor.description}</p>
            </div>
            <span
              className={`extension-health ${healthy ? 'healthy' : 'attention'}`}
            >
              {healthy ? (
                <CircleCheck size={15} aria-hidden="true" />
              ) : (
                <CircleAlert size={15} aria-hidden="true" />
              )}
              {healthy ? t('extensions.healthy') : t('extensions.attention')}
            </span>
          </section>

          <dl className="plugin-detail-summary">
            <div>
              <dt>
                <CircleCheck size={15} aria-hidden="true" />
                {t('plugins.invocationMode')}
              </dt>
              <dd>
                {status.invocation.mode === 'native'
                  ? t('plugins.nativeMode')
                  : status.invocation.mode === 'compatibility'
                    ? t('plugins.compatibilityMode')
                    : t('plugins.unavailableMode')}
              </dd>
            </div>
            <div>
              <dt>
                <Bot size={15} aria-hidden="true" />
                {t('plugins.runtime')}
              </dt>
              <dd>Codex</dd>
            </div>
            <div>
              <dt>
                <UsersRound size={15} aria-hidden="true" />
                {isExpertTeam
                  ? t('plugins.specialists')
                  : t('plugins.capabilities')}
              </dt>
              <dd>{descriptor.skills.length}</dd>
            </div>
            <div>
              <dt>
                <Code2 size={15} aria-hidden="true" />
                {t('plugins.version')}
              </dt>
              <dd>v{descriptor.version}</dd>
            </div>
            <div>
              <dt>{t('plugins.nativeRegistration')}</dt>
              <dd>
                {status.invocation.native_registration
                  ? t('plugins.available')
                  : t('plugins.unavailable')}
              </dd>
            </div>
            <div>
              <dt>{t('plugins.nativeDiscovery')}</dt>
              <dd>
                {status.invocation.native_discovery
                  ? t('plugins.available')
                  : t('plugins.unavailable')}
              </dd>
            </div>
            <div>
              <dt>{t('plugins.nativeInvocation')}</dt>
              <dd>
                {status.invocation.native_invocation
                  ? t('plugins.available')
                  : t('plugins.unavailable')}
              </dd>
            </div>
          </dl>

          <section className="plugin-detail-section">
            <div className="plugin-detail-section-heading">
              <div>
                <h3>{t('plugins.professionalCapabilities')}</h3>
                <p>
                  {isExpertTeam
                    ? t('plugins.professionalCapabilitiesHint')
                    : t('plugins.capabilityHint')}
                </p>
              </div>
              <span>{descriptor.skills.length}</span>
            </div>
            <div className="plugin-capability-list">
              {descriptor.skills.map((skill) => {
                const displayName = formatSkillDisplayName(skill.name)
                return (
                  <article key={skill.relative_path}>
                    <span aria-hidden="true">
                      <Bot size={16} />
                    </span>
                    <div>
                      <strong>{displayName}</strong>
                      {displayName !== skill.name ? <code>{skill.name}</code> : null}
                      <p>
                        {formatSkillPurpose(
                          skill.description,
                          t('skillPresentation.fallbackPurpose', {
                            name: displayName,
                          }),
                        )}
                      </p>
                    </div>
                  </article>
                )
              })}
            </div>
          </section>

          <section className="plugin-detail-section">
            <div className="plugin-detail-section-heading">
              <div>
                <h3>{t('plugins.tryInCodex')}</h3>
                <p>{t('plugins.tryInCodexHint')}</p>
              </div>
              <MessageSquareText size={18} aria-hidden="true" />
            </div>
            <div className="plugin-prompt-list">
              {prompts.map((prompt) => (
                <div key={prompt}>
                  <MessageSquareText size={15} aria-hidden="true" />
                  <span>{prompt}</span>
                </div>
              ))}
            </div>
          </section>

          <details className="plugin-technical-details">
            <summary>{t('plugins.technicalDetails')}</summary>
            <dl>
              <div>
                <dt>{t('plugins.pluginName')}</dt>
                <dd>
                  <code>{descriptor.name}</code>
                </dd>
              </div>
              <div>
                <dt>{t('plugins.source')}</dt>
                <dd>{descriptor.source.source_ref}</dd>
              </div>
              <div>
                <dt>{t('plugins.marketplace')}</dt>
                <dd>
                  <code>{status.marketplace_name}</code>
                </dd>
              </div>
              {status.catalog.visible ? (
                <>
                  <div>
                    <dt>{t('plugins.compatibilityLauncher')}</dt>
                    <dd>
                      <code>${status.catalog.skill_name}</code>
                    </dd>
                  </div>
                  <div>
                    <dt>{t('plugins.compatibilityLauncherPath')}</dt>
                    <dd>{status.catalog.path}</dd>
                  </div>
                </>
              ) : null}
              <div>
                <dt>{t('plugins.installPath')}</dt>
                <dd>{status.installed_path ?? t('unknown')}</dd>
              </div>
            </dl>
          </details>
        </div>

        <footer className="modal-footer">
          <button
            className="btn btn-primary"
            type="button"
            onClick={onRequestClose}
          >
            {t('close')}
          </button>
        </footer>
      </div>
    </div>
  )
}

export default memo(PluginDetailModal)
