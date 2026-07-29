import { memo, useMemo } from 'react'
import { Bot, Plus, Search, Settings2, Star } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { FeaturedSkillDto, ManagedSkill, OnlineSkillDto } from './types'
import {
  formatSkillDisplayName,
  formatSkillPurpose,
} from './skillPresentation'

type ExplorePageProps = {
  featuredSkills: FeaturedSkillDto[]
  featuredLoading: boolean
  exploreFilter: string
  searchResults: OnlineSkillDto[]
  searchLoading: boolean
  managedSkills: ManagedSkill[]
  loading: boolean
  detectedAgentLabels: string[]
  onExploreFilterChange: (value: string) => void
  onInstallSkill: (sourceUrl: string, skillName?: string) => void
  onManageAgents: () => void
  onOpenManualAdd: (tab?: 'git' | 'local') => void
  t: TFunction
}

function formatCount(n: number): string {
  if (n >= 1000000) return `${(n / 1000000).toFixed(1)}M`
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`
  return String(n)
}

const ExplorePage = ({
  featuredSkills,
  featuredLoading,
  exploreFilter,
  searchResults,
  searchLoading,
  managedSkills,
  loading,
  detectedAgentLabels,
  onExploreFilterChange,
  onInstallSkill,
  onManageAgents,
  onOpenManualAdd,
  t,
}: ExplorePageProps) => {
  const filteredSkills = useMemo(() => {
    if (!exploreFilter.trim()) return featuredSkills
    const lower = exploreFilter.toLowerCase()
    return featuredSkills.filter(
      (s) =>
        s.name.toLowerCase().includes(lower) ||
        s.summary.toLowerCase().includes(lower),
    )
  }, [featuredSkills, exploreFilter])

  const deduplicatedResults = useMemo(() => {
    const featuredNames = new Set(filteredSkills.map((s) => s.name.toLowerCase()))
    return searchResults.filter((s) => !featuredNames.has(s.name.toLowerCase()))
  }, [searchResults, filteredSkills])

  const isSearchActive = exploreFilter.trim().length >= 2

  // Check if a skill is already installed by matching name + source (case-insensitive)
  const installedSkillKeys = useMemo(() => {
    const keys = new Set<string>()
    for (const skill of managedSkills) {
      const source = (skill.source_ref ?? '')
        .replace('https://github.com/', '')
        .replace(/\.git$/, '')
        .split('/tree/')[0]
        .toLowerCase()
      keys.add(`${skill.name.toLowerCase()}|${source}`)
    }
    return keys
  }, [managedSkills])

  const isInstalled = (skillName: string, source: string) => {
    const normalizedSource = source
      .replace('https://github.com/', '')
      .replace(/\.git$/, '')
      .split('/tree/')[0]
      .toLowerCase()
    return installedSkillKeys.has(`${skillName.toLowerCase()}|${normalizedSource}`)
  }

  return (
    <div className="explore-page">
      <div className="explore-tabs" role="tablist" aria-label={t('addSkills')}>
        <button className="active" type="button" role="tab" aria-selected="true">
          {t('exploreTabs.online')}
        </button>
        <button type="button" role="tab" aria-selected="false" onClick={() => onOpenManualAdd('git')}>
          {t('exploreTabs.git')}
        </button>
        <button type="button" role="tab" aria-selected="false" onClick={() => onOpenManualAdd('local')}>
          {t('exploreTabs.local')}
        </button>
      </div>
      <div className="explore-hero">
        <div className="explore-search-row">
          <div className="explore-search-wrap">
            <Search size={16} className="explore-search-icon" />
            <input
              className="explore-search-input"
              placeholder={t('exploreFilterPlaceholder')}
              value={exploreFilter}
              onChange={(e) => onExploreFilterChange(e.target.value)}
            />
          </div>
          <button
            className="btn btn-secondary explore-manual-btn"
            type="button"
            onClick={() => onOpenManualAdd('git')}
            disabled={loading}
          >
            <Plus size={15} />
            {t('manualAdd')}
          </button>
        </div>
        <div className="explore-source-label">
          {t('exploreSourceHint')}
        </div>
        <div
          className={`quick-install-targets${
            detectedAgentLabels.length === 0 ? ' warning' : ''
          }`}
        >
          <Bot size={17} aria-hidden="true" />
          <div>
            <strong>
              {detectedAgentLabels.length > 0
                ? t('quickInstall.autoTargetsTitle')
                : t('quickInstall.noAgentsTitle')}
            </strong>
            <span>
              {detectedAgentLabels.length > 0
                ? t('quickInstall.autoTargetsDescription', {
                    targets: detectedAgentLabels.join(', '),
                  })
                : t('quickInstall.noAgentsDescription')}
            </span>
          </div>
          {detectedAgentLabels.length === 0 ? (
            <button
              className="btn btn-secondary"
              type="button"
              onClick={onManageAgents}
            >
              <Settings2 size={14} aria-hidden="true" />
              {t('quickInstall.manageAgents')}
            </button>
          ) : null}
        </div>
      </div>

      <div className="explore-scroll">
        {/* Featured section */}
        {featuredLoading ? (
          <div className="explore-loading">{t('exploreLoading')}</div>
        ) : (
          <>
            {isSearchActive && filteredSkills.length > 0 && (
              <div className="explore-section-title">{t('exploreFeaturedTitle')}</div>
            )}
            {filteredSkills.length > 0 ? (
              <div className="explore-grid">
                {filteredSkills.map((skill) => {
                  const installed = isInstalled(skill.name, skill.source_url)
                  const displayName = formatSkillDisplayName(skill.name)
                  const purpose = formatSkillPurpose(
                    skill.summary,
                    t('skillPresentation.fallbackPurpose', { name: displayName }),
                  )
                  return (
                    <div key={skill.slug} className="explore-card">
                      <div className="explore-card-top">
                        <div className="explore-card-info">
                          <div className="explore-card-name">{displayName}</div>
                          {displayName !== skill.name ? (
                            <code className="explore-card-technical-name">{skill.name}</code>
                          ) : null}
                          <div className="explore-card-author">
                            {skill.source_url
                              .replace('https://github.com/', '')
                              .split('/tree/')[0]}
                          </div>
                        </div>
                        {installed ? (
                          <span className="explore-btn-installed">
                            {t('status.installed')}
                          </span>
                        ) : (
                          <button
                            className="explore-btn-install"
                            type="button"
                            disabled={loading || detectedAgentLabels.length === 0}
                            onClick={() => onInstallSkill(skill.source_url)}
                          >
                            {t('quickInstall.action')}
                          </button>
                        )}
                      </div>
                      <div className="explore-card-desc">{purpose}</div>
                      <div className="explore-card-bottom">
                        <div className="explore-card-stats">
                          <span className="explore-stat">
                            <Star size={12} />
                            {formatCount(skill.stars)}
                          </span>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            ) : !isSearchActive ? (
              <div className="explore-empty">{t('exploreEmpty')}</div>
            ) : null}

            {/* Online search results */}
            {isSearchActive && (
              <>
                <div className="explore-section-title">{t('exploreOnlineTitle')}</div>
                {searchLoading ? (
                  <div className="explore-loading">{t('searchLoading')}</div>
                ) : deduplicatedResults.length > 0 ? (
                  <div className="explore-grid">
                    {deduplicatedResults.map((skill) => {
                      const installed = isInstalled(skill.name, skill.source_url)
                      const displayName = formatSkillDisplayName(skill.name)
                      const purpose = formatSkillPurpose(
                        undefined,
                        t('skillPresentation.fallbackPurpose', { name: displayName }),
                      )
                      return (
                        <div key={skill.source} className="explore-card">
                          <div className="explore-card-top">
                            <div className="explore-card-info">
                              <div className="explore-card-name">{displayName}</div>
                              {displayName !== skill.name ? (
                                <code className="explore-card-technical-name">{skill.name}</code>
                              ) : null}
                              <div className="explore-card-author">{skill.source}</div>
                            </div>
                            {installed ? (
                              <span className="explore-btn-installed">
                                {t('status.installed')}
                              </span>
                            ) : (
                              <button
                                className="explore-btn-install"
                                type="button"
                                disabled={loading || detectedAgentLabels.length === 0}
                                onClick={() => onInstallSkill(skill.source_url, skill.name)}
                              >
                                {t('quickInstall.action')}
                              </button>
                            )}
                          </div>
                          <div className="explore-card-desc">{purpose}</div>
                          <div className="explore-card-bottom">
                            <div className="explore-card-stats">
                              <span className="explore-stat">
                                {formatCount(skill.installs)} installs
                              </span>
                            </div>
                          </div>
                        </div>
                      )
                    })}
                  </div>
                ) : (
                  <div className="explore-empty">{t('searchEmpty')}</div>
                )}
              </>
            )}
          </>
        )}
      </div>
    </div>
  )
}

export default memo(ExplorePage)
