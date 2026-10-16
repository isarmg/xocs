import { useEffect, useRef, useState, type CSSProperties } from 'react';

export function SlidingTitle({ text }: { text: string }) {
  const viewport = useRef<HTMLSpanElement>(null), label = useRef<HTMLSpanElement>(null);
  const [distance, setDistance] = useState(0);
  useEffect(() => {
    if (!viewport.current || !label.current) return;
    let active = true;
    const measure = () => {
      if (active && viewport.current && label.current)
        setDistance(Math.max(0, label.current.scrollWidth - viewport.current.clientWidth));
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(viewport.current); observer.observe(label.current);
    void document.fonts.ready.then(measure);
    return () => { active = false; observer.disconnect(); };
  }, [text]);
  const style = { '--title-travel': `${distance}px`, '--title-duration': `${Math.max(6, distance / 28 + 3)}s` } as CSSProperties;
  return <span ref={viewport} className={`sliding-title${distance > 0 ? ' is-overflowing' : ''}`} title={text} style={style}>
    <span ref={label} className="sliding-title-text">{text}</span>
  </span>;
}
