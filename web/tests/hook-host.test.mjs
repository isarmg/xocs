import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, jsx, useCallback, useEffect, useId, useLayoutEffect, useRef, useState, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';

const { Select, TextField } = await originalModule(new URL('../node_modules/@xcss/web/dist/admin-ui/index.js', import.meta.url));

for (const Component of [Select, TextField]) {
  test(`expansion preserves the installed ${Component.name}${Component === Select ? ' native list' : ''} validation ref across value changes`, () => {
    const host = new HookHost(({ value }) => jsx(Component, { value, ...(Component === Select ? { size: 2 } : {}) }), { value: '' });
    try {
      host.render();
      const field = {
        validity: { valueMissing: true, customError: false },
        validationMessage: '',
        setCustomValidity(message) { this.validationMessage = message; },
      };
      expand(host.tree).props.onInvalid({ currentTarget: field });
      assert.equal(field.validationMessage, 'Please fill out this field.');
      // Merely inspecting the same render must not reset the component's ref.
      expand(host.tree);
      assert.equal(field.validationMessage, 'Please fill out this field.');
      host.props = { value: 'valid' };
      host.render();
      assert.equal(expand(host.tree).props.value, 'valid');
      assert.equal(field.validationMessage, '');
    } finally { host.unmount(); }
  });
}

test('expanded siblings retain independent hooks by key and clean up removed or replaced children', () => {
  const effects = [], cleanups = [];
  function Child({ id }) {
    const identity = useRef({ id });
    const generatedId = useId();
    const [count, setCount] = useState(0);
    useEffect(() => {
      effects.push(id);
      return () => cleanups.push(id);
    }, [id]);
    return jsx('button', { id, generatedId, identity: identity.current, count, onClick: () => setCount(value => value + 1) });
  }
  const Replacement = () => jsx('span', { children: 'replacement' });
  const host = new HookHost(({ ids, replace }) => jsx('div', {
    children: ids.map(id => jsx(replace ? Replacement : Child, { id }, id)),
  }), { ids: ['a', 'b'] });
  const buttons = () => walk(expand(host.tree), node => node.type === 'button');
  host.render();
  const initial = buttons();
  assert.deepEqual(effects, ['a', 'b']);
  assert.notEqual(initial[0].props.identity, initial[1].props.identity);
  assert.notEqual(initial[0].props.generatedId, initial[1].props.generatedId);
  initial[0].props.onClick();
  host.props = { ids: ['b', 'a'] };
  host.render();
  const reordered = buttons();
  assert.equal(reordered[0].props.count, 0);
  assert.equal(reordered[1].props.count, 1);
  assert.equal(reordered[1].props.identity, initial[0].props.identity);
  assert.equal(reordered[1].props.generatedId, initial[0].props.generatedId);
  assert.deepEqual(effects, ['a', 'b']);
  host.props = { ids: ['a'] };
  host.render(); buttons();
  assert.deepEqual(cleanups, ['b']);
  host.props = { ids: ['a'], replace: true };
  host.render(); expand(host.tree);
  assert.deepEqual(cleanups, ['b', 'a']);
  host.unmount();
  assert.deepEqual(cleanups, ['b', 'a']);
});

test('callback identity follows dependency values and captures the current render only when dependencies change', () => {
  const host = new HookHost(({ dependency }) => useCallback(() => dependency, [dependency]), { dependency: 1 });
  const first = host.render();
  assert.equal(host.render(), first);
  host.props = { dependency: 2 };
  const second = host.render();
  assert.notEqual(second, first);
  assert.equal(first(), 1);
  assert.equal(second(), 2);
  host.unmount();
});

test('layout effects precede passive effects and cleanups keep dependency and unmount ordering', () => {
  const events = [];
  const host = new HookHost(({ version }) => {
    useEffect(() => { events.push(`passive ${version}`); return () => events.push(`passive cleanup ${version}`); }, [version]);
    useLayoutEffect(() => { events.push(`layout ${version}`); return () => events.push(`layout cleanup ${version}`); }, [version]);
    return version;
  }, { version: 1 });
  host.render(); host.render();
  assert.deepEqual(events, ['layout 1', 'passive 1']);
  host.props = { version: 2 }; host.render();
  assert.deepEqual(events, ['layout 1', 'passive 1', 'layout cleanup 1', 'layout 2', 'passive cleanup 1', 'passive 2']);
  host.unmount();
  assert.deepEqual(events.slice(-2), ['layout cleanup 2', 'passive cleanup 2']);
});

test('the shared single-select boundary preserves caller form values, flags and handlers without pretending to render a DOM popup', () => {
  const change = () => {};
  const children = [jsx('option', { value: '', children: 'Choose' }), jsx('option', { value: 'a', children: 'Alpha' })];
  const props = { value: 'a', name: 'category', required: true, disabled: true, onChange: change, children };
  const host = new HookHost(() => jsx(Select, props));
  host.render();
  const field = expand(host.tree);
  assert.equal(field.type, 'select');
  assert.deepEqual(field.props, props);
  assert.equal(field.props.onChange, change);
  assert.deepEqual(field.props.children, children);
  host.unmount();
});

test('root unmount cleans expanded descendants once', () => {
  let cleanups = 0;
  const Child = () => {
    useEffect(() => () => { cleanups++; }, []);
    return null;
  };
  const host = new HookHost(() => jsx(Child, {}));
  host.render(); expand(host.tree);
  host.unmount(); host.unmount();
  assert.equal(cleanups, 1);
});

test('a throwing nested render restores the surrounding hook context', () => {
  const broken = new HookHost(() => { throw new Error('render failed'); });
  const outer = new HookHost(() => {
    const first = useRef('before');
    assert.throws(() => broken.render(), /render failed/);
    const second = useRef('after');
    return [first.current, second.current];
  });
  assert.deepEqual(outer.render(), ['before', 'after']);
  assert.deepEqual(outer.render(), ['before', 'after']);
});
