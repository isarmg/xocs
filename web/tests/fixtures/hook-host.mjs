// Deterministic hook/unit host, not a browser or a full React renderer.
// Components and event handlers are loaded unchanged from their original TSX.
let current;
let nextHostId = 0;
const owners = new WeakMap();
export function useState(initial) {
  const host = current, index = host.index++;
  if (!(index in host.slots)) {
    const slot = { value: typeof initial === 'function' ? initial() : initial };
    slot.set = next => {
      const value = typeof next === 'function' ? next(slot.value) : next;
      if (!Object.is(value, slot.value)) { slot.value = value; host.dirty = true; }
    };
    host.slots[index] = slot;
  }
  const slot = host.slots[index];
  return [slot.value, slot.set];
}
export function useRef(value) {
  const host = current, index = host.index++;
  return host.slots[index] ??= { current: value };
}
export function useMemo(factory, deps) {
  const memo = useRef(null);
  if (!memo.current || !deps || deps.some((dep, index) => !Object.is(dep, memo.current.deps[index])))
    memo.current = { deps, value: factory() };
  return memo.current.value;
}
export const StrictMode = ({ children }) => children;
export function useId() { return useRef(`unit-${current.id}-${current.index}`).current; }
export function useEffect(effect, deps) {
  const host = current, index = host.index++;
  const previous = host.slots[index];
  if (!previous || deps === undefined || deps.some((dep, i) => !Object.is(dep, previous.deps[i]))) {
    host.effects.push(() => {
      previous?.cleanup?.();
      host.slots[index] = { deps, cleanup: effect() };
    });
  }
}
export function jsx(type, props, key) { return { type, props: props ?? {}, key }; }
export const jsxs = jsx;
export const Fragment = 'fragment';
export class HookHost {
  slots = []; effects = []; dirty = true; index = 0; children = new Map();
  id = nextHostId++;
  constructor(component, props = {}) { this.component = component; this.props = props; }
  render(beforeEffects) {
    for (let i = 0; i < 30; i++) {
      const previous = current;
      current = this; this.index = 0; this.effects = []; this.dirty = false;
      try { this.tree = this.component(this.props); }
      finally { current = previous; }
      if (this.tree && typeof this.tree === 'object') owners.set(this.tree, this);
      beforeEffects?.(this.tree);
      const effects = this.effects; this.effects = [];
      for (const effect of effects) effect();
      if (!this.dirty) return this.tree;
    }
    throw new Error('State did not settle');
  }
  unmount() {
    for (const child of this.children.values()) child.unmount();
    this.children.clear();
    for (const slot of this.slots) slot?.cleanup?.();
    this.slots = [];
  }
}
export function walk(node, predicate, result = []) {
  if (Array.isArray(node)) { for (const item of node) walk(item, predicate, result); return result; }
  if (!node || typeof node !== 'object') return result;
  if (predicate(node)) result.push(node);
  walk(node.props?.children, predicate, result);
  return result;
}
// Expansion is lazy so callsite tests can still inspect the original component
// boundaries. Each rendered child needs its own persistent hook slots, just as
// the root does; invoking function components directly bypasses that context.
export function expand(node) {
  if (!node || typeof node !== 'object') return node;
  let owner = owners.get(node);
  if (!owner) {
    owner = { children: new Map() };
    owners.set(node, owner);
  }
  const visited = new Set();
  function visit(value, path) {
    if (Array.isArray(value)) return value.map((item, index) =>
      visit(item, `${path}/${item?.key == null ? `index:${index}` : `key:${JSON.stringify(item.key)}`}`));
    if (!value || typeof value !== 'object') return value;
    if (typeof value.type === 'function') {
      visited.add(path);
      let child = owner.children.get(path);
      if (child?.component !== value.type) {
        child?.unmount();
        child = new HookHost(value.type, value.props);
        owner.children.set(path, child);
      }
      child.props = value.props;
      return expand(child.render());
    }
    return { ...value, props: { ...value.props, children: visit(value.props.children, `${path}/children`) } };
  }
  const tree = visit(node, 'root');
  for (const [path, child] of owner.children) {
    if (!visited.has(path)) { child.unmount(); owner.children.delete(path); }
  }
  return tree;
}
export function textContent(node) {
  if (Array.isArray(node)) return node.map(textContent).join(' ');
  if (node === null || node === undefined || typeof node === 'boolean') return '';
  if (typeof node !== 'object') return String(node);
  return textContent(node.props?.children);
}
