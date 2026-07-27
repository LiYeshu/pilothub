import { memo } from 'react'
import { Database, ShieldCheck } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { StorageMigrationStatusDto } from '../types'

type StorageMigrationModalProps = {
  open: boolean
  migrating: boolean
  status: StorageMigrationStatusDto | null
  onLater: () => void
  onMigrate: () => void
  t: TFunction
}

const StorageMigrationModal = ({
  open,
  migrating,
  status,
  onLater,
  onMigrate,
  t,
}: StorageMigrationModalProps) => {
  if (!open || !status) return null

  return (
    <div className="modal-backdrop">
      <div className="modal" role="dialog" aria-modal="true" aria-labelledby="storage-migration-title">
        <div className="modal-header">
          <div>
            <div className="modal-title" id="storage-migration-title">
              {t('storageMigration.title')}
            </div>
            <div className="modal-subtitle">{t('storageMigration.subtitle')}</div>
          </div>
        </div>
        <div className="modal-body">
          <div className="notice">
            <Database size={18} />
            <div className="notice-copy">
              <strong>{t('storageMigration.moveTitle')}</strong>
              <span className="mono">{status.legacy_path}</span>
              <span className="mono">→ {status.target_path}</span>
            </div>
          </div>
          <div className="notice success">
            <ShieldCheck size={18} />
            <div className="notice-copy">
              <strong>{t('storageMigration.safeTitle')}</strong>
              <span>{t('storageMigration.safeBody', { path: status.backup_root })}</span>
            </div>
          </div>
        </div>
        <div className="modal-footer">
          <button className="btn btn-secondary" type="button" disabled={migrating} onClick={onLater}>
            {t('storageMigration.later')}
          </button>
          <button className="btn btn-primary" type="button" disabled={migrating} onClick={onMigrate}>
            {migrating ? t('storageMigration.migrating') : t('storageMigration.action')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(StorageMigrationModal)
