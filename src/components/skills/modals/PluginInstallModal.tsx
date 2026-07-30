import { memo } from 'react'
import {
  AlertTriangle,
  CheckCircle2,
  FolderOpen,
  GitBranch,
  PackageOpen,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import type { PluginPreview } from '../types'

type PluginInstallModalProps = {
  open: boolean
  loading: boolean
  sourceType: 'local' | 'git'
  sourceRef: string
  preview: PluginPreview | null
  onRequestClose: () => void
  onSourceTypeChange: (sourceType: 'local' | 'git') => void
  onSourceRefChange: (sourceRef: string) => void
  onPickLocalPath: () => void
  onPreview: () => void
  onInstall: () => void
  t: TFunction
}

const PluginInstallModal = ({
  open,
  loading,
  sourceType,
  sourceRef,
  preview,
  onRequestClose,
  onSourceTypeChange,
  onSourceRefChange,
  onPickLocalPath,
  onPreview,
  onInstall,
  t,
}: PluginInstallModalProps) => {
  if (!open) return null

  const canPreview = sourceRef.trim().length > 0 && !loading

  return (
    <div
      className="modal-backdrop"
      onClick={() => (loading ? null : onRequestClose())}
    >
      <div
        className="modal plugin-install-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="plugin-install-title"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal-header">
          <div className="modal-title" id="plugin-install-title">
            {t('plugins.installTitle')}
          </div>
          <button
            className="modal-close"
            type="button"
            onClick={onRequestClose}
            aria-label={t('close')}
            disabled={loading}
          >
            ✕
          </button>
        </header>

        <div className="modal-body plugin-install-body">
          <div className="tabs" role="tablist">
            <button
              className={`tab-item${sourceType === 'git' ? ' active' : ''}`}
              type="button"
              role="tab"
              aria-selected={sourceType === 'git'}
              onClick={() => onSourceTypeChange('git')}
              disabled={loading}
            >
              <GitBranch size={16} />
              {t('plugins.gitSource')}
            </button>
            <button
              className={`tab-item${sourceType === 'local' ? ' active' : ''}`}
              type="button"
              role="tab"
              aria-selected={sourceType === 'local'}
              onClick={() => onSourceTypeChange('local')}
              disabled={loading}
            >
              <FolderOpen size={16} />
              {t('plugins.localSource')}
            </button>
          </div>

          <section className="plugin-source-panel">
            <label className="label" htmlFor="plugin-source">
              {sourceType === 'git'
                ? t('plugins.repositoryUrl')
                : t('plugins.pluginFolder')}
            </label>
            <div className="input-row">
              <input
                className="input"
                id="plugin-source"
                value={sourceRef}
                placeholder={
                  sourceType === 'git'
                    ? t('plugins.gitPlaceholder')
                    : t('plugins.localPlaceholder')
                }
                onChange={(event) => onSourceRefChange(event.target.value)}
                disabled={loading}
              />
              {sourceType === 'local' ? (
                <button
                  className="btn btn-secondary input-action"
                  type="button"
                  onClick={onPickLocalPath}
                  disabled={loading}
                >
                  {t('browse')}
                </button>
              ) : null}
            </div>
            <p>{t('plugins.sourceHint')}</p>
          </section>

          {preview ? (
            <section className="plugin-preview-panel">
              <div className="plugin-preview-heading">
                <span className="extension-icon" aria-hidden="true">
                  <PackageOpen size={20} />
                </span>
                <div>
                  <h3>{preview.descriptor.display_name}</h3>
                  <code>{preview.descriptor.name}</code>
                </div>
                <span className="plugin-version">
                  v{preview.descriptor.version}
                </span>
              </div>
              <p>{preview.descriptor.description}</p>
              <dl className="plugin-preview-meta">
                <div>
                  <dt>{t('plugins.author')}</dt>
                  <dd>{preview.descriptor.author ?? t('unknown')}</dd>
                </div>
                <div>
                  <dt>{t('plugins.license')}</dt>
                  <dd>{preview.descriptor.license ?? t('unknown')}</dd>
                </div>
                <div>
                  <dt>{t('plugins.target')}</dt>
                  <dd>Codex</dd>
                </div>
                <div>
                  <dt>{t('plugins.skillCount')}</dt>
                  <dd>{preview.descriptor.skills.length}</dd>
                </div>
              </dl>
              <div className="plugin-skill-preview">
                {preview.descriptor.skills.map((skill) => (
                  <div key={skill.relative_path}>
                    <strong>{skill.name}</strong>
                    <span>{skill.description ?? t('skillDescriptionEmpty')}</span>
                  </div>
                ))}
              </div>
              {preview.validation.errors.map((item) => (
                <div className="plugin-validation error" key={item.code}>
                  <AlertTriangle size={16} />
                  <span>
                    {t(`plugins.validation.${item.code}`, {
                      defaultValue: item.message,
                    })}
                  </span>
                </div>
              ))}
              {preview.validation.warnings.map((item) => (
                <div className="plugin-validation warning" key={item.code}>
                  <AlertTriangle size={16} />
                  <span>
                    {t(`plugins.validation.${item.code}`, {
                      defaultValue: item.message,
                    })}
                  </span>
                </div>
              ))}
              {preview.validation.valid ? (
                <div className="plugin-validation success">
                  <CheckCircle2 size={16} />
                  <span>{t('plugins.validationPassed')}</span>
                </div>
              ) : null}
            </section>
          ) : null}
        </div>

        <footer className="modal-footer">
          <button
            className="btn btn-secondary"
            type="button"
            onClick={onRequestClose}
            disabled={loading}
          >
            {t('cancel')}
          </button>
          <button
            className="btn btn-secondary"
            type="button"
            onClick={onPreview}
            disabled={!canPreview}
          >
            {loading ? t('plugins.checking') : t('plugins.preview')}
          </button>
          <button
            className="btn btn-primary"
            type="button"
            onClick={onInstall}
            disabled={loading || !preview?.validation.valid}
          >
            {loading ? t('plugins.installing') : t('plugins.install')}
          </button>
        </footer>
      </div>
    </div>
  )
}

export default memo(PluginInstallModal)
