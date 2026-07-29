import { memo, useEffect, useRef } from 'react'
import type { TFunction } from 'i18next'
import { Download, RefreshCw, Search, Sparkles } from 'lucide-react'

type WelcomeModalProps = {
  open: boolean
  onSkip: () => void
  onStart: () => void
  t: TFunction
}

const WelcomeModal = ({ open, onSkip, onStart, t }: WelcomeModalProps) => {
  const startButtonRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (!open) return

    startButtonRef.current?.focus()
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onSkip()
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onSkip, open])

  if (!open) return null

  const steps = [
    { icon: Search, title: t('welcome.steps.discover') },
    { icon: Download, title: t('welcome.steps.install') },
    { icon: RefreshCw, title: t('welcome.steps.sync') },
  ]

  return (
    <div className="modal-backdrop welcome-backdrop">
      <div
        className="modal welcome-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="welcome-title"
        aria-describedby="welcome-description"
      >
        <div className="welcome-content">
          <span className="welcome-mark" aria-hidden="true">
            <Sparkles size={24} />
          </span>
          <div className="welcome-heading">
            <p>{t('welcome.eyebrow')}</p>
            <h1 id="welcome-title">{t('welcome.title')}</h1>
            <span id="welcome-description">{t('welcome.description')}</span>
          </div>
          <ol className="welcome-steps">
            {steps.map(({ icon: Icon, title }, index) => (
              <li key={title}>
                <span className="welcome-step-icon" aria-hidden="true">
                  <Icon size={18} />
                </span>
                <span className="welcome-step-index">{index + 1}</span>
                <strong>{title}</strong>
              </li>
            ))}
          </ol>
        </div>
        <div className="modal-footer welcome-actions">
          <button className="btn btn-secondary" type="button" onClick={onSkip}>
            {t('welcome.skip')}
          </button>
          <button
            ref={startButtonRef}
            className="btn btn-primary"
            type="button"
            onClick={onStart}
          >
            {t('welcome.start')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(WelcomeModal)
