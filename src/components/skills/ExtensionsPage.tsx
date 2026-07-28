import { memo } from 'react'
import { Boxes, ChevronRight, PackageOpen } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { Extension, ManagedSkill } from './types'
import { resolveExtensionSkills } from './extensionView'

type ExtensionsPageProps = {
  extensions: Extension[]
  managedSkills: ManagedSkill[]
  onOpenSkill: (skill: ManagedSkill) => void
  t: TFunction
}

const ExtensionsPage = ({
  extensions,
  managedSkills,
  onOpenSkill,
  t,
}: ExtensionsPageProps) => (
  <div className="extensions-page">
    <header className="extensions-header">
      <div>
        <h1>{t('extensions.title')}</h1>
        <p>{t('extensions.subtitle')}</p>
      </div>
      <div className="extensions-summary" aria-label={t('extensions.summary')}>
        <Boxes size={18} />
        <strong>{extensions.length}</strong>
        <span>{t('extensions.collections')}</span>
      </div>
    </header>

    {extensions.length === 0 ? (
      <div className="extensions-empty">
        <PackageOpen size={28} />
        <strong>{t('extensions.emptyTitle')}</strong>
        <span>{t('extensions.emptyBody')}</span>
      </div>
    ) : (
      <div className="extension-collection-list">
        {extensions.map((extension) => {
          const skills = resolveExtensionSkills(extension, managedSkills)
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
                <span className="extension-count">
                  {t('extensions.skillCount', { count: skills.length })}
                </span>
              </div>
              <div className="extension-components">
                {skills.map(({ componentId, skill }) => (
                  <button
                    className="extension-component"
                    type="button"
                    key={componentId}
                    onClick={() => onOpenSkill(skill)}
                  >
                    <span>
                      <strong>{skill.name}</strong>
                      <small>{skill.description || t('skillDescriptionEmpty')}</small>
                    </span>
                    <ChevronRight size={17} aria-hidden="true" />
                  </button>
                ))}
              </div>
            </section>
          )
        })}
      </div>
    )}
  </div>
)

export default memo(ExtensionsPage)
