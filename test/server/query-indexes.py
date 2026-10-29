#!/usr/bin/env python3
"""Compare current content indexes against the same schema without them.

Run with: python3 -I -B test/server/query-indexes.py
Uses real SQLite, verifies identical rows and query plans, and reports median
warm query time. Timings are diagnostic; no machine-dependent speed threshold.
"""

from pathlib import Path
import sqlite3
from statistics import median
from time import perf_counter_ns


ROOT = Path(__file__).resolve().parents[2]
INDEXES = ("comment_floor", "wei_yan_source", "article_label")
CASES = (
    (
        "comment_floor",
        "SELECT COUNT(*) FROM comment WHERE floor_comment_id=? AND type='article'",
        (120,),
    ),
    (
        "comment_floor",
        "SELECT id,comment_content FROM comment WHERE floor_comment_id=? AND source=? AND type='article' "
        "ORDER BY create_time,id LIMIT 5",
        (120, 12),
    ),
    (
        "wei_yan_source",
        "SELECT id,content FROM wei_yan WHERE type='news' AND source=? AND is_public=1 "
        "ORDER BY create_time DESC,id DESC LIMIT 5",
        (120,),
    ),
    (
        "article_label",
        "SELECT id FROM article WHERE label_id=? AND deleted=0 "
        "ORDER BY create_time DESC,id DESC LIMIT 12",
        (20,),
    ),
)


def measure(connection, sql, parameters):
    rows = connection.execute(sql, parameters).fetchall()
    samples = []
    for _ in range(20):
        started = perf_counter_ns()
        assert connection.execute(sql, parameters).fetchall() == rows
        samples.append(perf_counter_ns() - started)
    plan = [row[3] for row in connection.execute("EXPLAIN QUERY PLAN " + sql, parameters)]
    return rows, median(samples) / 1_000, plan


def main():
    with sqlite3.connect(":memory:") as connection:
        connection.executescript((ROOT / "src/server/schema/product.sql").read_text())
        definitions = []
        for name in INDEXES:
            (definition,) = connection.execute(
                "SELECT sql FROM sqlite_schema WHERE type='index' AND name=?", (name,)
            ).fetchone()
            definitions.append(definition)
            connection.execute(f"DROP INDEX {name}")
        connection.execute("INSERT INTO user(id,username) VALUES(1,'author')")
        connection.executemany(
            "INSERT INTO comment(id,source,type,parent_comment_id,floor_comment_id,comment_content) "
            "VALUES(?,?,'article',?,?,'reply')",
            ((i, i // 10, 1 if i % 10 else None, i - i % 10 if i % 10 else None)
             for i in range(1, 50_001)),
        )
        connection.executemany(
            "INSERT INTO wei_yan(id,user_id,type,source,content,is_public) "
            "VALUES(?,1,'news',?,'body',1)",
            ((i, i // 10) for i in range(1, 30_001)),
        )
        connection.executemany(
            "INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) "
            "VALUES(?,1,?,?,'title','body')",
            ((i, i % 50, i % 100) for i in range(1, 10_001)),
        )
        before = [measure(connection, sql, parameters) for _, sql, parameters in CASES]
        for definition in definitions:
            connection.execute(definition)
        print(f"SQLite {sqlite3.sqlite_version}; 50k comments, 30k updates, 10k articles")
        for (name, sql, parameters), (old_rows, old_us, old_plan) in zip(CASES, before):
            rows, elapsed_us, plan = measure(connection, sql, parameters)
            assert rows == old_rows, name
            assert any(
                f"USING INDEX {name}" in step or f"USING COVERING INDEX {name}" in step
                for step in plan
            ), plan
            assert not any("TEMP B-TREE" in step for step in plan), plan
            print(f"{name}: {old_us:.1f} -> {elapsed_us:.1f} us median")
            print(f"  before: {'; '.join(old_plan)}")
            print(f"  after:  {'; '.join(plan)}")
        # Reply totals and root deletion use the same floor index, without
        # losing the indexed primary-key branch for deleting the root itself.
        count_sql = "SELECT COUNT(*) FROM comment WHERE floor_comment_id=120 AND type='article'"
        assert connection.execute(count_sql).fetchone() == (9,)
        delete_plan = list(connection.execute(
            "EXPLAIN QUERY PLAN DELETE FROM comment WHERE id=120 OR floor_comment_id=120"
        ))
        assert any("comment_floor" in row[3] for row in delete_plan), delete_plan


if __name__ == "__main__":
    main()
