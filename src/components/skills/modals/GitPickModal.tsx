import { memo, useMemo, useState } from 'react'
import { Search } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { GitSkillCandidate, SkillCollection } from '../types'

type GitPickModalProps = {
  open: boolean
  loading: boolean
  collection: SkillCollection | null
  gitCandidates: GitSkillCandidate[]
  gitCandidateSelected: Record<string, boolean>
  installScope: 'global' | 'project'
  targetLabels: string[]
  onRequestClose: () => void
  onCancel: () => void
  onToggleCandidate: (subpath: string, checked: boolean) => void
  onInstall: () => void
  t: TFunction
}

const GitPickModal = ({
  open,
  loading,
  collection,
  gitCandidates,
  gitCandidateSelected,
  installScope,
  targetLabels,
  onRequestClose,
  onCancel,
  onToggleCandidate,
  onInstall,
  t,
}: GitPickModalProps) => {
  const [query, setQuery] = useState('')
  const normalizedQuery = query.trim().toLowerCase()
  const filteredCandidates = useMemo(() => {
    if (!normalizedQuery) return gitCandidates
    return gitCandidates.filter((c) =>
      [c.name, c.description ?? '', c.subpath].some((value) =>
        value.toLowerCase().includes(normalizedQuery),
      ),
    )
  }, [gitCandidates, normalizedQuery])
  const selectedCount = filteredCandidates.filter(
    (c) => gitCandidateSelected[c.subpath],
  ).length
  const allVisibleSelected =
    filteredCandidates.length > 0 &&
    filteredCandidates.every((c) => gitCandidateSelected[c.subpath])

  const toggleVisibleCandidates = (checked: boolean) => {
    filteredCandidates.forEach((c) => onToggleCandidate(c.subpath, checked))
  }

  if (!open) return null

  return (
    <div className="modal-backdrop" onClick={onRequestClose}>
      <div className="modal pick-skill-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div className="modal-title">{t('gitPickTitle')}</div>
          <button
            className="modal-close"
            type="button"
            onClick={onRequestClose}
            aria-label={t('close')}
          >
            ✕
          </button>
        </div>
        <div className="modal-body">
          {collection ? (
            <section className="collection-preview" aria-label={t('collectionPreview.title')}>
              <div className="collection-preview-heading">
                <div>
                  <div className="collection-preview-name">{collection.name}</div>
                  <div className="collection-preview-source">{collection.source_url}</div>
                </div>
                <span className="collection-preview-count">
                  {t('collectionPreview.skillCount', {
                    count: collection.skills.length,
                  })}
                </span>
              </div>
              <dl className="collection-preview-meta">
                <div>
                  <dt>{t('collectionPreview.author')}</dt>
                  <dd>{collection.author ?? t('collectionPreview.unknown')}</dd>
                </div>
                <div>
                  <dt>{t('collectionPreview.license')}</dt>
                  <dd>{collection.license ?? t('collectionPreview.unknown')}</dd>
                </div>
                <div>
                  <dt>{t('collectionPreview.scope')}</dt>
                  <dd>{t(`scope.${installScope}`)}</dd>
                </div>
                <div>
                  <dt>{t('collectionPreview.targets')}</dt>
                  <dd>
                    {targetLabels.length > 0
                      ? targetLabels.join(', ')
                      : t('collectionPreview.noTargets')}
                  </dd>
                </div>
              </dl>
            </section>
          ) : null}
          <p className="label">{t('gitPickBody')}</p>
          <div className="pick-search">
            <Search size={16} className="search-icon-abs" />
            <input
              className="search-input"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t('pickSearchPlaceholder')}
            />
          </div>
          <div className="pick-toolbar">
            <label className="inline-checkbox">
              <input
                type="checkbox"
                checked={allVisibleSelected}
                onChange={(e) => toggleVisibleCandidates(e.target.checked)}
                disabled={filteredCandidates.length === 0}
              />
              {t('selectAll')}
            </label>
            <span className="pick-toolbar-count">
              {t('selectedCount', {
                selected: selectedCount,
                total: filteredCandidates.length,
              })}
            </span>
          </div>
          <div className="pick-list">
            {filteredCandidates.length === 0 ? (
              <div className="empty">{t('pickSearchEmpty')}</div>
            ) : null}
            {filteredCandidates.map((c) => (
              <div className="pick-item" key={c.subpath}>
                <label className="pick-item-checkbox">
                  <input
                    type="checkbox"
                    checked={Boolean(gitCandidateSelected[c.subpath])}
                    onChange={(e) => onToggleCandidate(c.subpath, e.target.checked)}
                  />
                </label>
                <div className="pick-item-main">
                  <div className="pick-item-title">{c.name}</div>
                  {c.description ? (
                    <div className="pick-item-desc">{c.description}</div>
                  ) : null}
                  <div className="pick-item-path">{c.subpath}</div>
                  <div className="pick-item-contents">
                    {c.contents.map((content) => (
                      <span key={content}>{content}</span>
                    ))}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={onCancel} disabled={loading}>
            {t('cancel')}
          </button>
          <button className="btn btn-primary" onClick={onInstall} disabled={loading}>
            {t('installSelected')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(GitPickModal)
