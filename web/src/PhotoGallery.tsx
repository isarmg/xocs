import { t } from '@xcss/web/admin-ui/i18n';
import { safeImageUrl } from './MediaPreview';

export type PhotoItem = { id: number; title: string | null; classify: string | null; cover: string | null; create_time: string | null };

export function PhotoGrid({ items, onPreview }: { items: PhotoItem[]; onPreview: (src: string | null) => void }) {
  return <div className="travel-grid">
    {items.map(item => {
      const cover = safeImageUrl(item.cover);
      const title = item.title || item.classify || t('相册照片', 'Album photo');
      return <button className="travel-photo-card" type="button" key={item.id} disabled={!cover}
        aria-label={t('查看照片：{0}', 'View photo: {0}', [title])} onClick={() => onPreview(cover)}>
        <span className="travel-photo-image">{cover ? <img src={cover} alt="" loading="lazy" decoding="async" />
          : <span>{t('记录美好瞬间', 'Capture beautiful moments')}</span>}</span>
        <span className="travel-photo-body"><strong>{title}</strong>
          <small>{t('日期（UTC）：', 'Date (UTC): ')}{item.create_time?.slice(0, 10) || t('最近', 'Recently')}</small></span>
      </button>;
    })}
    {items.length === 0 && <div className="travel-photo-empty public-empty-state">
      <span aria-hidden="true">▧</span><h3>{t('这一组还没有照片', 'No photos in this collection yet')}</h3>
      <p>{t('切换其它相册，或稍后回来看看新的回忆。', 'Try another collection, or come back for new memories.')}</p>
    </div>}
  </div>;
}
