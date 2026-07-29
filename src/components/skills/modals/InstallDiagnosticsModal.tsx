import { memo } from 'react'
import {
  AlertTriangle,
  CheckCircle2,
  CircleX,
  RefreshCw,
  Stethoscope,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import type {
  InstallDiagnosticCheckDto,
  InstallDiagnosticsDto,
} from '../types'

type InstallDiagnosticsModalProps = {
  open: boolean
  loading: boolean
  result: InstallDiagnosticsDto | null
  error: string | null
  onClose: () => void
  onRetry: () => void
  t: TFunction
}

const statusIcon = (check: InstallDiagnosticCheckDto) => {
  if (check.status === 'pass') return <CheckCircle2 size={18} />
  if (check.status === 'warning') return <AlertTriangle size={18} />
  return <CircleX size={18} />
}

const InstallDiagnosticsModal = ({
  open,
  loading,
  result,
  error,
  onClose,
  onRetry,
  t,
}: InstallDiagnosticsModalProps) => {
  if (!open) return null

  return (
    <div className="modal-backdrop diagnostics-backdrop" onClick={loading ? undefined : onClose}>
      <div
        className="modal install-diagnostics-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="install-diagnostics-title"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="modal-header">
          <div>
            <div className="modal-title" id="install-diagnostics-title">
              {t('installDiagnostics.title')}
            </div>
            <div className="install-diagnostics-subtitle">
              {t('installDiagnostics.subtitle')}
            </div>
          </div>
          <button
            className="modal-close"
            type="button"
            onClick={onClose}
            disabled={loading}
            aria-label={t('close')}
          >
            ✕
          </button>
        </div>

        <div className="modal-body install-diagnostics-body">
          {loading ? (
            <div className="install-diagnostics-loading" role="status">
              <RefreshCw size={20} className="spinning" />
              <strong>{t('installDiagnostics.running')}</strong>
              <span>{t('installDiagnostics.runningHint')}</span>
            </div>
          ) : error ? (
            <div className="install-diagnostics-error" role="alert">
              <CircleX size={20} />
              <div>
                <strong>{t('installDiagnostics.unavailable')}</strong>
                <span>{error}</span>
              </div>
            </div>
          ) : (
            <div className="install-diagnostics-list">
              {(result?.checks ?? []).map((check) => (
                <section
                  className={`install-diagnostic-check ${check.status}`}
                  key={check.id}
                >
                  <span className="install-diagnostic-icon" aria-hidden="true">
                    {statusIcon(check)}
                  </span>
                  <div className="install-diagnostic-copy">
                    <strong>{t(`installDiagnostics.checks.${check.id}.title`)}</strong>
                    <span>
                      {t(
                        `installDiagnostics.checks.${check.id}.${check.status}`,
                        {
                          detail: check.detail ?? '',
                          count: check.paths.length,
                        },
                      )}
                    </span>
                    {check.status !== 'pass' ? (
                      <small>
                        {t(`installDiagnostics.checks.${check.id}.hint`)}
                      </small>
                    ) : null}
                    {check.paths.length > 0 ? (
                      <code title={check.paths.join('\n')}>
                        {check.paths.join(' · ')}
                      </code>
                    ) : null}
                  </div>
                </section>
              ))}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button
            className="btn btn-secondary"
            type="button"
            onClick={onClose}
            disabled={loading}
          >
            {t('close')}
          </button>
          <button
            className="btn btn-primary"
            type="button"
            onClick={onRetry}
            disabled={loading}
          >
            <Stethoscope size={15} />
            {t('installDiagnostics.retry')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(InstallDiagnosticsModal)
