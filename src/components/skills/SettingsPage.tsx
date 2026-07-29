import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import {
  ArrowLeft,
  Boxes,
  Database,
  ExternalLink,
  Github,
  Palette,
  RefreshCw,
  ShieldCheck,
} from 'lucide-react'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { TFunction } from 'i18next'
import { toast } from 'sonner'
import type {
  GithubProxyConfigDto,
  PackageManagerStatusDto,
  ProductFeedbackStatusDto,
} from './types'

const PROJECT_REPOSITORY_URL = 'https://github.com/LiYeshu/pilothub'

type SettingsPageProps = {
  isTauri: boolean
  language: string
  storagePath: string
  gitCacheCleanupDays: number
  gitCacheTtlSecs: number
  themePreference: 'system' | 'light' | 'dark'
  githubToken: string
  githubProxyConfig: GithubProxyConfigDto
  invokeTauri: <T>(command: string, args?: Record<string, unknown>) => Promise<T>
  onPickStoragePath: () => void
  onToggleLanguage: () => void
  onThemeChange: (nextTheme: 'system' | 'light' | 'dark') => void
  onGitCacheCleanupDaysChange: (nextDays: number) => void
  onGitCacheTtlSecsChange: (nextSecs: number) => void
  onClearGitCacheNow: () => void
  onGithubTokenChange: (token: string) => void
  onGithubProxyConfigChange: (enabled: boolean, port: number) => void
  onBack: () => void
  t: TFunction
}

const SettingsPage = ({
  isTauri,
  language,
  storagePath,
  gitCacheCleanupDays,
  gitCacheTtlSecs,
  themePreference,
  invokeTauri,
  onPickStoragePath,
  onToggleLanguage,
  onThemeChange,
  onGitCacheCleanupDaysChange,
  onGitCacheTtlSecsChange,
  onClearGitCacheNow,
  githubToken,
  onGithubTokenChange,
  githubProxyConfig,
  onGithubProxyConfigChange,
  onBack,
  t,
}: SettingsPageProps) => {
  const [localToken, setLocalToken] = useState(githubToken)
  useEffect(() => {
    setLocalToken(githubToken)
  }, [githubToken])
  const [localGithubProxyPort, setLocalGithubProxyPort] = useState(
    String(githubProxyConfig.port),
  )
  useEffect(() => {
    setLocalGithubProxyPort(String(githubProxyConfig.port))
  }, [githubProxyConfig.port])

  const [appVersion, setAppVersion] = useState<string | null>(null)
  const [packageManagers, setPackageManagers] = useState<PackageManagerStatusDto[]>([])
  const [packageManagersLoading, setPackageManagersLoading] = useState(false)
  const [packageManagerInstalling, setPackageManagerInstalling] = useState(false)
  const [productFeedback, setProductFeedback] =
    useState<ProductFeedbackStatusDto | null>(null)
  const [productFeedbackSaving, setProductFeedbackSaving] = useState(false)
  const versionText = useMemo(() => {
    if (!isTauri) return t('notAvailable')
    if (!appVersion) return t('unknown')
    return `v${appVersion}`
  }, [appVersion, isTauri, t])

  const loadAppVersion = useCallback(async () => {
    if (!isTauri) {
      setAppVersion(null)
      return
    }
    try {
      const { getVersion } = await import('@tauri-apps/api/app')
      const v = await getVersion()
      setAppVersion(v)
    } catch {
      setAppVersion(null)
    }
  }, [isTauri])

  useEffect(() => {
    void loadAppVersion()
  }, [loadAppVersion])

  const loadPackageManagers = useCallback(async () => {
    if (!isTauri) {
      setPackageManagers([])
      return
    }
    setPackageManagersLoading(true)
    try {
      setPackageManagers(
        await invokeTauri<PackageManagerStatusDto[]>('get_package_manager_status'),
      )
    } catch {
      setPackageManagers([])
    } finally {
      setPackageManagersLoading(false)
    }
  }, [invokeTauri, isTauri])

  useEffect(() => {
    void loadPackageManagers()
  }, [loadPackageManagers])

  const loadProductFeedback = useCallback(async () => {
    if (!isTauri) {
      setProductFeedback({ enabled: false, event_count: 0 })
      return
    }
    try {
      setProductFeedback(
        await invokeTauri<ProductFeedbackStatusDto>(
          'get_product_feedback_status',
        ),
      )
    } catch {
      setProductFeedback(null)
    }
  }, [invokeTauri, isTauri])

  useEffect(() => {
    void loadProductFeedback()
  }, [loadProductFeedback])

  const handleProductFeedbackToggle = useCallback(async () => {
    if (!productFeedback) return
    setProductFeedbackSaving(true)
    try {
      const status = await invokeTauri<ProductFeedbackStatusDto>(
        'set_product_feedback_enabled',
        { enabled: !productFeedback.enabled },
      )
      setProductFeedback(status)
      toast.success(
        t(
          status.enabled
            ? 'productFeedback.enabledToast'
            : 'productFeedback.disabledToast',
        ),
      )
    } catch (error) {
      toast.error(t('productFeedback.saveFailed'), {
        description: error instanceof Error ? error.message : String(error),
      })
    } finally {
      setProductFeedbackSaving(false)
    }
  }, [invokeTauri, productFeedback, t])

  const handleClearProductFeedback = useCallback(async () => {
    setProductFeedbackSaving(true)
    try {
      setProductFeedback(
        await invokeTauri<ProductFeedbackStatusDto>(
          'clear_product_feedback',
        ),
      )
      toast.success(t('productFeedback.clearedToast'))
    } catch (error) {
      toast.error(t('productFeedback.clearFailed'), {
        description: error instanceof Error ? error.message : String(error),
      })
    } finally {
      setProductFeedbackSaving(false)
    }
  }, [invokeTauri, t])

  const handleInstallPackageManager = useCallback(async () => {
    setPackageManagerInstalling(true)
    try {
      const installed = await invokeTauri<PackageManagerStatusDto>(
        'install_managed_apm_runtime',
      )
      setPackageManagers([installed])
      toast.success(t('packageManagers.installSuccess'))
    } catch (error) {
      toast.error(t('packageManagers.installFailed'), {
        description: error instanceof Error ? error.message : String(error),
      })
    } finally {
      setPackageManagerInstalling(false)
    }
  }, [invokeTauri, t])

  const handleOpenProject = useCallback(async () => {
    try {
      if (isTauri) {
        await openUrl(PROJECT_REPOSITORY_URL)
      } else {
        window.open(PROJECT_REPOSITORY_URL, '_blank', 'noopener,noreferrer')
      }
    } catch {
      toast.error(t('projectLink.openFailed'))
    }
  }, [isTauri, t])

  return (
    <div className="settings-page">
      <div className="settings-shell">
        <div className="settings-hero">
          <div className="settings-hero-main">
            <div className="settings-title-row">
              <button className="detail-back-btn settings-back" type="button" onClick={onBack}>
                <ArrowLeft size={16} />
                {t('detail.back')}
              </button>
              <div className="settings-title-copy">
                <h1>{t('settings')}</h1>
                <p>{t('settingsPageSubtitle')}</p>
              </div>
            </div>
          </div>
          <div className="settings-hero-summary" aria-label={t('settingsSummary')}>
            <div className="settings-summary-item">
              <span>{t('interfaceLanguage')}</span>
              <strong>{t(`languageOptions.${language}`)}</strong>
            </div>
            <div className="settings-summary-item">
              <span>{t('themeMode')}</span>
              <strong>{t(`themeOptions.${themePreference}`)}</strong>
            </div>
            <div className="settings-summary-item">
              <span>{t('appVersion')}</span>
              <strong>{versionText}</strong>
            </div>
          </div>
        </div>

        <div className="settings-grid">
          <div className="settings-column">
            <section className="settings-card">
              <div className="settings-card-head">
                <span className="settings-card-icon">
                  <Palette size={18} />
                </span>
                <div>
                  <h2>{t('settingsSectionAppearance')}</h2>
                  <p>{t('settingsSectionAppearanceDesc')}</p>
                </div>
              </div>
              <div className="settings-card-body">
                <div className="settings-field">
                  <label className="settings-label" htmlFor="settings-language">
                    {t('interfaceLanguage')}
                  </label>
                  <div className="settings-select-wrap">
                    <select
                      id="settings-language"
                      className="settings-select"
                      value={language}
                      onChange={(event) => {
                        if (event.target.value !== language) {
                          onToggleLanguage()
                        }
                      }}
                    >
                      <option value="en">{t('languageOptions.en')}</option>
                      <option value="zh">{t('languageOptions.zh')}</option>
                    </select>
                    <svg
                      className="settings-select-caret"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      aria-hidden="true"
                    >
                      <path d="M6 9l6 6 6-6" />
                    </svg>
                  </div>
                </div>

                <div className="settings-field">
                  <label className="settings-label" id="settings-theme-label">
                    {t('themeMode')}
                  </label>
                  <div className="settings-theme-options" role="group" aria-labelledby="settings-theme-label">
                    <button
                      type="button"
                      className={`settings-theme-btn ${
                        themePreference === 'system' ? 'active' : ''
                      }`}
                      aria-pressed={themePreference === 'system'}
                      onClick={() => onThemeChange('system')}
                    >
                      {t('themeOptions.system')}
                    </button>
                    <button
                      type="button"
                      className={`settings-theme-btn ${
                        themePreference === 'light' ? 'active' : ''
                      }`}
                      aria-pressed={themePreference === 'light'}
                      onClick={() => onThemeChange('light')}
                    >
                      {t('themeOptions.light')}
                    </button>
                    <button
                      type="button"
                      className={`settings-theme-btn ${
                        themePreference === 'dark' ? 'active' : ''
                      }`}
                      aria-pressed={themePreference === 'dark'}
                      onClick={() => onThemeChange('dark')}
                    >
                      {t('themeOptions.dark')}
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <section className="settings-card">
            <div className="settings-card-head">
              <span className="settings-card-icon">
                <Database size={18} />
              </span>
              <div>
                <h2>{t('settingsSectionStorage')}</h2>
                <p>{t('settingsSectionStorageDesc')}</p>
              </div>
            </div>
            <div className="settings-card-body">
              <div className="settings-field">
                <label className="settings-label" htmlFor="settings-storage">
                  {t('skillsStoragePath')}
                </label>
                <div className="settings-input-row">
                  <input
                    id="settings-storage"
                    className="settings-input mono"
                    value={storagePath}
                    readOnly
                  />
                  <button
                    className="btn btn-secondary settings-browse"
                    type="button"
                    onClick={onPickStoragePath}
                  >
                    {t('browse')}
                  </button>
                </div>
                <div className="settings-helper">{t('skillsStorageHint')}</div>
              </div>

              <div className="settings-field">
                <label className="settings-label" htmlFor="settings-git-cache-days">
                  {t('gitCacheCleanupDays')}
                </label>
                <div className="settings-input-row">
                  <input
                    id="settings-git-cache-days"
                    className="settings-input"
                    type="number"
                    min={0}
                    max={3650}
                    step={1}
                    value={gitCacheCleanupDays}
                    onChange={(event) => {
                      const next = Number(event.target.value)
                      if (!Number.isNaN(next)) {
                        onGitCacheCleanupDaysChange(next)
                      }
                    }}
                  />
                  <button
                    className="btn btn-secondary settings-browse"
                    type="button"
                    onClick={onClearGitCacheNow}
                  >
                    {t('cleanNow')}
                  </button>
                </div>
                <div className="settings-helper">{t('gitCacheCleanupHint')}</div>
              </div>

              <div className="settings-field">
                <label className="settings-label" htmlFor="settings-git-cache-ttl">
                  {t('gitCacheTtlSecs')}
                </label>
                <div className="settings-input-row">
                  <input
                    id="settings-git-cache-ttl"
                    className="settings-input"
                    type="number"
                    min={0}
                    max={3600}
                    step={1}
                    value={gitCacheTtlSecs}
                    onChange={(event) => {
                      const next = Number(event.target.value)
                      if (!Number.isNaN(next)) {
                        onGitCacheTtlSecsChange(next)
                      }
                    }}
                  />
                </div>
                <div className="settings-helper">{t('gitCacheTtlHint')}</div>
              </div>
            </div>
            </section>

            <section className="settings-card">
              <div className="settings-card-head">
                <span className="settings-card-icon">
                  <ShieldCheck size={18} />
                </span>
                <div>
                  <h2>{t('settingsSectionPrivacy')}</h2>
                  <p>{t('settingsSectionPrivacyDesc')}</p>
                </div>
              </div>
              <div className="settings-card-body">
                <div className="settings-item">
                  <div className="settings-item-info">
                    <div className="settings-item-title">
                      {t('productFeedback.title')}
                    </div>
                    <div className="settings-item-desc">
                      {t('productFeedback.description')}
                    </div>
                  </div>
                  <button
                    type="button"
                    className={`settings-toggle${
                      productFeedback?.enabled ? ' checked' : ''
                    }`}
                    aria-pressed={Boolean(productFeedback?.enabled)}
                    aria-label={t('productFeedback.title')}
                    disabled={!productFeedback || productFeedbackSaving}
                    onClick={() => void handleProductFeedbackToggle()}
                  >
                    <span className="settings-toggle-knob" />
                  </button>
                </div>
                <div className="product-feedback-boundary">
                  <strong>{t('productFeedback.boundaryTitle')}</strong>
                  <span>{t('productFeedback.boundaryBody')}</span>
                  <small>{t('productFeedback.localOnly')}</small>
                </div>
                <div className="settings-project-row product-feedback-actions">
                  <div className="settings-item-info">
                    <div className="settings-item-title">
                      {t('productFeedback.localEvents', {
                        count: productFeedback?.event_count ?? 0,
                      })}
                    </div>
                    <div className="settings-item-desc">
                      {t('productFeedback.clearDescription')}
                    </div>
                  </div>
                  <button
                    className="btn btn-secondary btn-sm"
                    type="button"
                    disabled={
                      productFeedbackSaving ||
                      !productFeedback ||
                      productFeedback.event_count === 0
                    }
                    onClick={() => void handleClearProductFeedback()}
                  >
                    {t('productFeedback.clear')}
                  </button>
                </div>
              </div>
            </section>
          </div>

          <div className="settings-column">
            <section className="settings-card">
            <div className="settings-card-head">
              <span className="settings-card-icon">
                <Github size={18} />
              </span>
              <div>
                <h2>{t('settingsSectionNetwork')}</h2>
                <p>{t('settingsSectionNetworkDesc')}</p>
              </div>
            </div>
            <div className="settings-card-body">
              <div className="settings-project-row">
                <div className="settings-item-info">
                  <div className="settings-item-title">{t('projectLink.title')}</div>
                  <div className="settings-item-desc">{t('projectLink.description')}</div>
                </div>
                <button
                  className="btn btn-secondary btn-sm settings-project-link"
                  type="button"
                  onClick={() => void handleOpenProject()}
                  aria-label={t('projectLink.open')}
                >
                  {t('projectLink.view')}
                  <ExternalLink size={14} />
                </button>
              </div>
              <div className="settings-field">
                <label className="settings-label" htmlFor="settings-github-token">
                  {t('githubToken')}
                </label>
                <div className="settings-input-row">
                  <input
                    id="settings-github-token"
                    className="settings-input mono"
                    type="password"
                    placeholder={t('githubTokenPlaceholder')}
                    value={localToken}
                    onChange={(e) => setLocalToken(e.target.value)}
                    onBlur={() => {
                      if (localToken !== githubToken) {
                        onGithubTokenChange(localToken)
                      }
                    }}
                  />
                </div>
                <div className="settings-helper">{t('githubTokenHint')}</div>
              </div>

              <div className="settings-field">
                <div className="settings-item">
                  <div className="settings-item-info">
                    <div className="settings-item-title">{t('networkProxy')}</div>
                    <div className="settings-item-desc">{t('networkProxyHint')}</div>
                  </div>
                  <button
                    type="button"
                    className={`settings-toggle${githubProxyConfig.enabled ? ' checked' : ''}`}
                    aria-pressed={githubProxyConfig.enabled}
                    onClick={() => {
                      const nextPort = Number(localGithubProxyPort)
                      onGithubProxyConfigChange(
                        !githubProxyConfig.enabled,
                        Number.isNaN(nextPort) ? githubProxyConfig.port : nextPort,
                      )
                    }}
                  >
                    <span className="settings-toggle-knob" />
                  </button>
                </div>
                <label className="settings-label" htmlFor="settings-github-proxy-port">
                  {t('networkProxyPort')}
                </label>
                <div className="settings-input-row">
                  <input
                    id="settings-github-proxy-port"
                    className="settings-input mono"
                    type="number"
                    min={1}
                    max={65535}
                    step={1}
                    value={localGithubProxyPort}
                    onChange={(e) => setLocalGithubProxyPort(e.target.value)}
                    onBlur={() => {
                      const nextPort = Number(localGithubProxyPort)
                      if (
                        githubProxyConfig.enabled &&
                        !Number.isNaN(nextPort) &&
                        nextPort !== githubProxyConfig.port
                      ) {
                        onGithubProxyConfigChange(true, nextPort)
                      }
                    }}
                  />
                </div>
                <div className="settings-helper">
                  {githubProxyConfig.auto_detected
                    ? t('networkProxyAutoDetected')
                    : t('networkProxyPortHint')}
                </div>
              </div>
            </div>
            </section>

            <section className="settings-card">
            <div className="settings-card-head">
              <span className="settings-card-icon">
                <RefreshCw size={18} />
              </span>
              <div>
                <h2>{t('settingsSectionUpdates')}</h2>
                <p>{t('settingsSectionUpdatesDesc')}</p>
              </div>
            </div>
            <div className="settings-card-body">
              <div className="settings-version-row">
                <div>
                  <span className="settings-version-label">{t('appVersion')}</span>
                  <span className="settings-version-text">{versionText}</span>
                </div>
              </div>
            <div className="settings-helper">{t('updaterDisabled')}</div>
            </div>
            </section>

            <section className="settings-card">
            <div className="settings-card-head">
              <span className="settings-card-icon">
                <Boxes size={18} />
              </span>
              <div>
                <h2>{t('settingsSectionPackageManagers')}</h2>
                <p>{t('settingsSectionPackageManagersDesc')}</p>
              </div>
            </div>
            <div className="settings-card-body">
              {packageManagersLoading ? (
                <div className="settings-helper">{t('packageManagers.detecting')}</div>
              ) : packageManagers.length === 0 ? (
                <div className="settings-helper">{t('packageManagers.none')}</div>
              ) : (
                packageManagers.map((manager) => (
                  <div className="settings-project-row" key={manager.id}>
                    <div className="settings-item-info">
                      <div className="settings-item-title">{manager.label}</div>
                      <div className="settings-item-desc mono">
                        {manager.version ?? t('packageManagers.versionUnknown')}
                        {manager.source
                          ? ` · ${t(`packageManagers.source.${manager.source}`)}`
                          : ''}
                      </div>
                    </div>
                    {manager.available ? (
                      <span className="badge">{t('packageManagers.available')}</span>
                    ) : (
                      <button
                        className="btn btn-primary"
                        type="button"
                        disabled={packageManagerInstalling}
                        onClick={() => void handleInstallPackageManager()}
                      >
                        {packageManagerInstalling
                          ? t('packageManagers.installing')
                          : t('packageManagers.install')}
                      </button>
                    )}
                  </div>
                ))
              )}
            </div>
            </section>
          </div>
        </div>
      </div>
    </div>
  )
}

export default memo(SettingsPage)
