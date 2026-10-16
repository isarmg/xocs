import type { CSSProperties } from 'react';

export function ReferencePetals() {
  return <div className="reference-petals" aria-hidden="true">{Array.from({ length: 18 }, (_, index) =>
    <span key={index} style={{ '--petal-left': `${(index * 37 + 9) % 100}%`, '--petal-delay': `${-(index * 2.7)}s`, '--petal-duration': `${16 + index % 7}s`, '--petal-drift': `${index % 2 ? -70 : 90}px` } as CSSProperties} />
  )}</div>;
}
