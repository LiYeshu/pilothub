import { memo } from 'react'
import type { TFunction } from 'i18next'
import type { InstalledCodexPlugin } from '../types'

type PluginUninstallModalProps = {
  plugin: InstalledCodexPlugin | null
  loading: boolean
  onRequestClose: () => void
  onConfirm: () => void
  t: TFunction
}

const PluginUninstallModal = ({
  plugin,
  loading,
  onRequestClose,
  onConfirm,
  t,
}: PluginUninstallModalProps) => {
  if (!plugin) return null

  return (
    <div
      className="modal-backdrop"
      onClick={() => (loading ? null : onRequestClose())}
    >
      <div
        className="modal delete-modal"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="plugin-uninstall-title"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal-header">
          <div className="modal-title" id="plugin-uninstall-title">
            {t('plugins.uninstallTitle')}
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
        <div className="modal-body">
          <p>
            {t('plugins.uninstallConfirm', {
              name: plugin.descriptor.display_name,
            })}
          </p>
          <p className="helper-text">{t('plugins.uninstallHint')}</p>
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
            className="btn btn-danger"
            type="button"
            onClick={onConfirm}
            disabled={loading}
          >
            {loading ? t('plugins.uninstalling') : t('plugins.uninstall')}
          </button>
        </footer>
      </div>
    </div>
  )
}

export default memo(PluginUninstallModal)
