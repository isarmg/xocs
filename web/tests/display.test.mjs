import assert from 'node:assert/strict';
import { test } from 'node:test';
import { ApiError, DisplayError, publicErrorMessage } from './fixtures/react-baseline/src/api.ts';

test('API and unexpected errors show local text rather than server diagnostics', () => {
  const error = new ApiError(409, { code: 'conflict', message: '内部路径 /private/secret', retryable: false });
  assert.equal(error.message, 'The content or state has changed. Refresh and try again.');
  assert.equal(publicErrorMessage(new Error('SECRET stack'), 'Unable to save'), 'Unable to save');
  assert.equal(publicErrorMessage({ status: 503, message: 'SECRET service' }), 'The service is temporarily unavailable. Please try again later.');
  assert.equal(publicErrorMessage({ status: 401 }), 'Check your credentials or sign in again.');
  assert.equal(publicErrorMessage(new DisplayError('Choose a valid image')), 'Choose a valid image');
});
