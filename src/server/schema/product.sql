-- Product-owned Blog schema. xcss metadata and administrator DDL are composed by build.rs.
PRAGMA foreign_keys = ON;

CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE,
    password TEXT,
    phone_number TEXT UNIQUE,
    email TEXT UNIQUE,
    user_status INTEGER NOT NULL DEFAULT 1,
    gender INTEGER,
    open_id TEXT,
    avatar TEXT,
    admire TEXT,
    subscribe TEXT,
    introduction TEXT,
    user_type INTEGER NOT NULL DEFAULT 2,
    create_time TEXT DEFAULT CURRENT_TIMESTAMP,
    update_time TEXT DEFAULT CURRENT_TIMESTAMP,
    update_by TEXT,
    deleted INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE article (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES user(id),
    sort_id INTEGER NOT NULL,
    label_id INTEGER NOT NULL,
    article_cover TEXT,
    article_title TEXT NOT NULL,
    article_content TEXT NOT NULL,
    video_url TEXT,
    view_count INTEGER NOT NULL DEFAULT 0,
    like_count INTEGER NOT NULL DEFAULT 0,
    view_status INTEGER NOT NULL DEFAULT 1,
    password TEXT,
    tips TEXT,
    recommend_status INTEGER NOT NULL DEFAULT 0,
    comment_status INTEGER NOT NULL DEFAULT 1,
    create_time TEXT DEFAULT CURRENT_TIMESTAMP,
    update_time TEXT DEFAULT CURRENT_TIMESTAMP,
    update_by TEXT,
    deleted INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX article_public_latest ON article(deleted,view_status,create_time DESC,id DESC);
CREATE INDEX article_sort ON article(sort_id,label_id,deleted,create_time DESC);
CREATE VIRTUAL TABLE article_search USING fts5(article_title,article_content,content='article',content_rowid='id',tokenize='trigram');
CREATE TRIGGER article_search_insert AFTER INSERT ON article BEGIN
    INSERT INTO article_search(rowid,article_title,article_content) VALUES(new.id,new.article_title,new.article_content);
END;
CREATE TRIGGER article_search_delete AFTER DELETE ON article BEGIN
    INSERT INTO article_search(article_search,rowid,article_title,article_content) VALUES('delete',old.id,old.article_title,old.article_content);
END;
CREATE TRIGGER article_search_update AFTER UPDATE OF article_title,article_content ON article BEGIN
    INSERT INTO article_search(article_search,rowid,article_title,article_content) VALUES('delete',old.id,old.article_title,old.article_content);
    INSERT INTO article_search(rowid,article_title,article_content) VALUES(new.id,new.article_title,new.article_content);
END;
CREATE TABLE comment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source INTEGER NOT NULL,
    type TEXT,
    parent_comment_id INTEGER,
    user_id INTEGER,
    floor_comment_id INTEGER,
    parent_user_id INTEGER,
    like_count INTEGER NOT NULL DEFAULT 0,
    comment_content TEXT NOT NULL,
    comment_info TEXT,
    create_time TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX comment_source ON comment(source,type,create_time,id);
-- A durable receipt prevents a retried submission from being inserted twice,
-- including after a moderator removes the original comment.
CREATE TABLE comment_submission (
    request_id TEXT PRIMARY KEY NOT NULL CHECK(length(request_id)=36),
    request_hash BLOB NOT NULL CHECK(length(request_hash)=32),
    comment_id INTEGER REFERENCES comment(id) ON DELETE SET NULL
);
CREATE TABLE sort (id INTEGER PRIMARY KEY AUTOINCREMENT,sort_name TEXT NOT NULL,sort_description TEXT,sort_type INTEGER,priority INTEGER);
CREATE TABLE home_sections (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 32),
    kind TEXT NOT NULL CHECK(kind IN ('latest','recommended','category')),
    sort_id INTEGER REFERENCES sort(id) ON DELETE CASCADE,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)),
    CHECK((kind='category' AND sort_id IS NOT NULL) OR (kind!='category' AND sort_id IS NULL))
);
CREATE INDEX home_sections_visible ON home_sections(enabled,priority,id);
INSERT INTO home_sections(title,kind,priority) VALUES('最新', 'latest', 0),('推荐阅读', 'recommended', 10);
CREATE TABLE label (id INTEGER PRIMARY KEY AUTOINCREMENT,sort_id INTEGER NOT NULL,label_name TEXT NOT NULL,label_description TEXT);
CREATE TABLE tree_hole (id INTEGER PRIMARY KEY AUTOINCREMENT,avatar TEXT,message TEXT,image_path TEXT REFERENCES resource(path),user_id INTEGER REFERENCES user(id),create_time TEXT DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE wei_yan (id INTEGER PRIMARY KEY AUTOINCREMENT,user_id INTEGER,like_count INTEGER NOT NULL DEFAULT 0,content TEXT,image_path TEXT REFERENCES resource(path),type TEXT,source INTEGER,is_public INTEGER,create_time TEXT DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE web_info (id INTEGER PRIMARY KEY,web_name TEXT,web_title TEXT,notices TEXT,footer TEXT,background_image TEXT,avatar TEXT,random_cover TEXT,status INTEGER);
CREATE TABLE resource_path (id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT,classify TEXT,cover TEXT,url TEXT,introduction TEXT,type TEXT,status INTEGER,remark TEXT,create_time TEXT DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE resource (id INTEGER PRIMARY KEY,user_id INTEGER,type TEXT,path TEXT UNIQUE,size INTEGER,original_name TEXT,mime_type TEXT,status INTEGER,store_type TEXT,create_time TEXT DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE family (id INTEGER PRIMARY KEY AUTOINCREMENT,user_id INTEGER,bg_cover TEXT,man_cover TEXT,woman_cover TEXT,man_name TEXT,woman_name TEXT,timing TEXT,countdown_title TEXT,countdown_time TEXT,status INTEGER,family_info TEXT,create_time TEXT DEFAULT CURRENT_TIMESTAMP,update_time TEXT DEFAULT CURRENT_TIMESTAMP);

-- Initial create identity is immutable, even when the article is later edited or deleted.
CREATE TABLE article_submission (
    request_id TEXT PRIMARY KEY NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    request_hash BLOB NOT NULL CHECK(length(request_hash)=32),
    article_id INTEGER NOT NULL REFERENCES article(id)
);

-- Preserve publish identity after deletion so a retry cannot restore an update.
CREATE TABLE article_news_submission (
    request_id TEXT PRIMARY KEY NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    request_hash BLOB NOT NULL CHECK(length(request_hash)=32),
    news_id INTEGER REFERENCES wei_yan(id) ON DELETE SET NULL
);
