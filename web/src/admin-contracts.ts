import { arrayOf, isRecord, isSiteInfo, pageOf } from './public-contracts.ts';

export type User = { id:number; username:string|null; email:string|null; phone_number:string|null; user_status:number; user_type:number; create_time:string|null };
export type Comment = { id:number; source:number; comment_type:string|null; user_id:number|null; username:string|null; comment_content:string; create_time:string|null };
export type Resource = { id:number; original_name:string|null; path:string|null; resource_type:string|null; size:number|null; status:number|null; create_time:string|null };
export type Label = { id:number; sort_id:number; label_name:string; label_description:string|null };
export type Statistics = { articles:number; comments:number; members:number; categories:number };
export type Link = { id:number; title:string|null; classify:string|null; cover:string|null; url:string|null; introduction:string|null; link_type:string|null; status:number|null; create_time:string|null };
export type TreeHole = { id:number; user_id:number|null; username:string|null; avatar:string|null; message:string; image_path:string|null; create_time:string|null };
export type Note = { id:number; user_id:number|null; username:string|null; content:string; image_path:string|null; is_public:number; create_time:string|null };
export type Family = { id:number; bg_cover:string|null; man_cover:string|null; woman_cover:string|null; man_name:string|null; woman_name:string|null; timing:string|null; countdown_title:string|null; countdown_time:string|null; family_info:string|null; status:number|null };
export type UploadedImage = { id:number; path:string; mime_type:'image/png'|'image/jpeg'|'image/gif'|'image/webp'; size:number };

const integer = (value:unknown):value is number => Number.isSafeInteger(value);
const count = (value:unknown):value is number => integer(value) && value >= 0;
const id = (value:unknown):value is number => integer(value) && value > 0;
const nullableId = (value:unknown) => value === null || id(value);
const flag = (value:unknown) => value === 0 || value === 1;
const nullableInteger = (value:unknown) => value === null || integer(value);
const strings = (value:Record<string,unknown>, fields:string[]) => fields.every(key => value[key] === null || typeof value[key] === 'string');

export const isUser = (value:unknown):value is User => isRecord(value) && id(value.id) && flag(value.user_status) && integer(value.user_type) && strings(value,['username','email','phone_number','create_time']);
export const isComment = (value:unknown):value is Comment => isRecord(value) && id(value.id) && count(value.source) && nullableId(value.user_id) && typeof value.comment_content === 'string' && strings(value,['comment_type','username','create_time']);
export const isResource = (value:unknown):value is Resource => isRecord(value) && id(value.id) && (value.size === null || count(value.size)) && nullableInteger(value.status) && strings(value,['original_name','path','resource_type','create_time']);
export const isLabel = (value:unknown):value is Label => isRecord(value) && id(value.id) && id(value.sort_id) && typeof value.label_name === 'string' && strings(value,['label_description']);
export const isLink = (value:unknown):value is Link => isRecord(value) && id(value.id) && (value.status === null || flag(value.status)) && strings(value,['title','classify','cover','url','introduction','link_type','create_time']);
export const isTreeHole = (value:unknown):value is TreeHole => isRecord(value) && id(value.id) && nullableId(value.user_id) && typeof value.message === 'string' && strings(value,['username','avatar','image_path','create_time']);
export const isNote = (value:unknown):value is Note => isRecord(value) && id(value.id) && nullableId(value.user_id) && typeof value.content === 'string' && flag(value.is_public) && strings(value,['username','image_path','create_time']);
export const isFamily = (value:unknown):value is Family => isRecord(value) && id(value.id) && (value.status === null || flag(value.status)) && strings(value,['bg_cover','man_cover','woman_cover','man_name','woman_name','timing','countdown_title','countdown_time','family_info']);
export const isStatistics = (value:unknown):value is Statistics => isRecord(value) && ['articles','comments','members','categories'].every(key => count(value[key]));
export const isStatus = (value:unknown):value is {active:boolean} => isRecord(value) && typeof value.active === 'boolean';
export const isDeleted = (value:unknown):value is {deleted:true} => isRecord(value) && value.deleted === true;
export const isUploadedImage = (value:unknown):value is UploadedImage => isRecord(value) && id(value.id) && typeof value.path === 'string' && /^\/media\/[a-f0-9]{64}\.(?:png|jpg|gif|webp)$/.test(value.path) && typeof value.mime_type === 'string' && ['image/png','image/jpeg','image/gif','image/webp'].includes(value.mime_type) && count(value.size) && value.size > 0 && value.size <= 10*1024*1024;
export const isUserPage = pageOf(isUser), isCommentPage = pageOf(isComment), isResourcePage = pageOf(isResource), isLinksPage = pageOf(isLink), isTreePage = pageOf(isTreeHole), isNotesPage = pageOf(isNote);
export const isList = arrayOf(isLabel);
export const isSite = isSiteInfo;
