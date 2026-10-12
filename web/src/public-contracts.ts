import type { Article, ArticleSummary, Category, Page, PublicLabel, ResponseValidator, SiteInfo } from './api';

// These are the consumed read models of src/content/*.rs and src/home.rs.
// Nullable fields are required; missing fields and coercion are not accepted.
export const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === 'object' && value !== null && !Array.isArray(value);
const isString = (value: unknown): value is string => typeof value === 'string';
const isNullableString = (value: unknown): value is string | null => value === null || isString(value);
const isInteger = (value: unknown): value is number => typeof value === 'number' && Number.isSafeInteger(value);
const isCount = (value: unknown): value is number => isInteger(value) && value >= 0;
const isId = (value: unknown): value is number => isInteger(value) && value > 0;
const isNullableId = (value: unknown): value is number | null => value === null || isId(value);
const isFlag = (value: unknown): value is number => value === 0 || value === 1;
const strings = (value: Record<string, unknown>, fields: string[]) => fields.every(field => isNullableString(value[field]));

export function arrayOf<T>(validate: ResponseValidator<T>): ResponseValidator<T[]> {
  return (value): value is T[] => Array.isArray(value) && value.every(validate);
}
export function pageOf<T>(validate: ResponseValidator<T>, maxSize = 100): ResponseValidator<Page<T>> {
  return (value): value is Page<T> => isRecord(value) && Array.isArray(value.items) && value.items.every(validate)
    && isCount(value.total) && isId(value.page) && value.page <= 100_000
    && isId(value.size) && value.size <= maxSize && value.items.length <= value.size;
}

function articleFields(value: Record<string, unknown>): boolean {
  return isId(value.id) && isString(value.article_title) && isCount(value.sort_id) && isCount(value.label_id)
    && strings(value, ['article_cover', 'sort_name', 'label_name', 'create_time'])
    && isCount(value.view_count) && isCount(value.like_count) && isCount(value.comment_count)
    && isFlag(value.recommend_status) && isFlag(value.view_status);
}
export const isArticleSummary = (value: unknown): value is ArticleSummary => isRecord(value) && articleFields(value)
  && strings(value, ['excerpt', 'search_snippet']);
export const isArticlePage = pageOf(isArticleSummary);
export const isArticle = (value: unknown): value is Article => isRecord(value) && articleFields(value)
  && isId(value.user_id) && isString(value.article_content) && isFlag(value.comment_status) && isFlag(value.password_required)
  && strings(value, ['video_url', 'username', 'tips', 'update_time']);
export const isArticleAccess = (value: unknown): value is { password_required: number; tips: string | null } => isRecord(value)
  && isFlag(value.password_required) && isNullableString(value.tips);
export const isCategory = (value: unknown): value is Category => isRecord(value) && isId(value.id)
  && isString(value.sort_name) && isNullableString(value.sort_description)
  && (value.priority === null || isInteger(value.priority)) && isCount(value.article_count);
export const isCategories = arrayOf(isCategory);
export const isPublicLabel = (value: unknown): value is PublicLabel => isRecord(value) && isId(value.id) && isId(value.sort_id)
  && isString(value.label_name) && isNullableString(value.label_description) && isCount(value.article_count);
export const isPublicLabels = arrayOf(isPublicLabel);
export type EditorLabel = { id: number; sort_id: number; label_name: string; label_description: string | null };
export const isEditorLabels = arrayOf((value: unknown): value is EditorLabel => isRecord(value) && isId(value.id) && isId(value.sort_id)
  && isString(value.label_name) && isNullableString(value.label_description));
export const isSiteInfo = (value: unknown): value is SiteInfo => isRecord(value)
  && strings(value, ['web_name', 'web_title', 'notices', 'footer', 'background_image', 'avatar', 'random_cover']);
export const isSiteStats = (value: unknown): value is { article_count: number; view_count: number } => isRecord(value)
  && isCount(value.article_count) && isCount(value.view_count);

export type HomeSection = { id: number; title: string; kind: 'latest' | 'recommended' | 'category'; sort_id: number | null; priority: number; enabled: boolean };
export const isHomeSection = (value: unknown): value is HomeSection => isRecord(value) && isId(value.id) && isString(value.title)
  && (value.kind === 'latest' || value.kind === 'recommended' || value.kind === 'category')
  && (value.kind === 'category' ? isId(value.sort_id) : value.sort_id === null) && isInteger(value.priority) && typeof value.enabled === 'boolean';
export const isHomeSections = arrayOf(isHomeSection);
export type PublicComment = { id: number; user_id: number | null; username: string | null; avatar: string | null; comment_content: string; create_time: string | null; parent_username: string | null; reply_count: number };
export const isPublicComment = (value: unknown): value is PublicComment => isRecord(value) && isId(value.id) && isNullableId(value.user_id)
  && strings(value, ['username', 'avatar', 'create_time', 'parent_username']) && isString(value.comment_content) && isCount(value.reply_count);
export const isPublicCommentPage = pageOf(isPublicComment, 50);
export type NewsEntry = { id: number; content: string; create_time: string | null };
export const isNewsEntry = (value: unknown): value is NewsEntry => isRecord(value) && isId(value.id) && isString(value.content) && isNullableString(value.create_time);
export const isNews = arrayOf(isNewsEntry);
export type Note = { id: number; user_id: number | null; username: string | null; content: string; image_path: string | null; like_count: number; is_public: number; create_time: string | null };
export const isNote = (value: unknown): value is Note => isRecord(value) && isId(value.id) && isNullableId(value.user_id)
  && strings(value, ['username', 'image_path', 'create_time']) && isString(value.content) && isCount(value.like_count) && isFlag(value.is_public);
export const isNotePage = pageOf(isNote, 50);
export type WallPost = { id: number; user_id: number | null; username: string | null; avatar: string | null; message: string; image_path: string | null; create_time: string | null };
export const isWallPost = (value: unknown): value is WallPost => isRecord(value) && isId(value.id) && isNullableId(value.user_id)
  && strings(value, ['username', 'avatar', 'image_path', 'create_time']) && isString(value.message);
export const isWallPosts = arrayOf(isWallPost);
export type Link = { id: number; title: string | null; classify: string | null; cover: string | null; url: string | null; introduction: string | null; link_type: string | null; create_time: string | null };
export const isLink = (value: unknown): value is Link => isRecord(value) && isId(value.id)
  && strings(value, ['title', 'classify', 'cover', 'url', 'introduction', 'link_type', 'create_time']);
export const isLinkPage = pageOf(isLink);
export type LinkClass = { classify: string; count: number };
export const isLinkClasses = arrayOf((value: unknown): value is LinkClass => isRecord(value) && isString(value.classify) && isCount(value.count));
export type Family = { id: number; bg_cover: string | null; man_cover: string | null; woman_cover: string | null; man_name: string | null; woman_name: string | null; timing: string | null; countdown_title: string | null; countdown_time: string | null; family_info: string | null };
export const isFamilies = arrayOf((value: unknown): value is Family => isRecord(value) && isId(value.id)
  && strings(value, ['bg_cover', 'man_cover', 'woman_cover', 'man_name', 'woman_name', 'timing', 'countdown_title', 'countdown_time', 'family_info']));
