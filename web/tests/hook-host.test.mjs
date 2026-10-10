import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, jsx, useEffect, useId, useRef, useState, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';

const { Select, TextField } = await originalModule(new URL('../node_modules/@xcss/web/dist/admin-ui/index.js', import.meta.url));

for (const Component of [Select, TextField]) {
  test(`expansion preserves the installed ${Component.name} validation ref across value changes`, () => {
    const host = new HookHost(({ value }) => jsx(Component, { value }), { value: '' });
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
