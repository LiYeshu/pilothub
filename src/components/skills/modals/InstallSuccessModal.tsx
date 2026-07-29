import { memo, useEffect, useRef } from 'react'
import type { TFunction } from 'i18next'
import { Bot, CheckCircle2 } from 'lucide-react'
import type { InstallSuccessState } from '../quickInstall'

type InstallSuccessModalProps = {
  result: InstallSuccessState | null
  onClose: () => void
  onViewSkill: () => void
  t: TFunction
}

const InstallSuccessModal = ({
  result,
  onClose,
  onViewSkill,
  t,
}: InstallSuccessModalProps) => {
  const viewButtonRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (!result) return

    viewButtonRef.current?.focus()
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onClose, result])

  if (!result) return null

  return (
    <div className="modal-backdrop install-success-backdrop">
      <div
        className="modal install-success-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="install-success-title"
        aria-describedby="install-success-description"
      >
        <div className="install-success-content">
          <CheckCircle2
            className="install-success-icon"
            size={42}
            aria-hidden="true"
          />
          <div>
            <h1 id="install-success-title">{t('quickInstall.successTitle')}</h1>
            <p id="install-success-description">
              {t('quickInstall.successDescription', {
                name: result.skillName,
              })}
            </p>
          </div>
          <section className="install-success-targets">
            <span>{t('quickInstall.installedTo')}</span>
            <ul>
              {result.targetLabels.map((label) => (
                <li key={label}>
                  <Bot size={16} aria-hidden="true" />
                  <strong>{label}</strong>
                </li>
              ))}
            </ul>
          </section>
          <section className="install-success-next">
            <span>{t('quickInstall.nextStep')}</span>
            <p>
              {t('quickInstall.usageHint', {
                name: result.skillName,
              })}
            </p>
          </section>
        </div>
        <div className="modal-footer install-success-actions">
          <button className="btn btn-secondary" type="button" onClick={onClose}>
            {t('quickInstall.done')}
          </button>
          <button
            ref={viewButtonRef}
            className="btn btn-primary"
            type="button"
            onClick={onViewSkill}
          >
            {t('quickInstall.viewSkill')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(InstallSuccessModal)
