# 资产编译边界与源码快照测量

2026-10-23，在 Linux AMD64 GNU 主机上比较同一源码的两种编译边界。测量支持把互不引用的 CSS 和 JavaScript 编译模块分别编译：冷编译耗时从 61.973 s 降到 42.820 s，CSS 修改后从 31.459 s 降到 14.223 s，JS 修改后从 31.155 s 降到 17.026 s。并行冷编译的进程树 RSS 峰值略有增加，下面同时记录该成本。

另一个独立实验只替换资产构建器的源码快照实现：六次调用的耗时中位数从 13.846737 s 降到 7.980404 s。两个实验针对不同阶段，不能相加推算完整产品的提速。

## 编译边界和维护成本

原 `blog-assets-compiler` 的 35 个模块包含两个没有相互 `crate::` 引用的子图。CSS 的 8 个模块移到 `blog-assets-style-compiler`，JS 的 27 个模块保留在原包内。两个包仍分别作为一个编译单元维护内部解析、优化和序列化实现，没有为每个来源模块恢复独立 Cargo 包，也没有增加转发层。

| 新 CSS 包的模块 | 保留的来源身份 |
| --- | --- |
| `style_color` | `cssparser-color@0.1.0` |
| `style_parser` | `cssparser@0.33.0` |
| `short_float_format` | `dtoa-short@0.3.5` |
| `float_format` | `dtoa@1.0.11` |
| `style_optimizer` | `lightningcss@1.0.0-alpha.71` |
| `style_selectors` | `parcel_selectors@0.28.2` |
| `relative_path` | `pathdiff@0.2.3` |
| `cached_hash` | `precomputed-hash@0.1.1` |

CSS 约 11.1 万行，JS 约 29.0 万行。模块内的算法不因拆分而改写。边界改动包括两个 manifest 的依赖归属、CSS derive 宏的绝对类型路径，以及资产构建器的 CSS 调用路径。代价是多一个本地包和一份来源映射；收益是改动一种语言的编译器时，无需重新编译另一种语言的完整实现。包数本身不是决定依据。

## 环境和方法

- 基线为 Git 提交 `75b525a5ab399e67489e0618e16a31d6ffc0d49c` 的完整归档。
- Rust `1.99.0 (b940084d7 2026-09-28)`，Cargo `1.99.0 (5f94df478 2026-08-27)`，LLVM 23.1.1。
- Linux `6.18.55 x86_64 GNU/Linux`，AMD EPYC 9V74 虚拟机。系统可见 5 vCPU，cgroup 配额为 4 CPU，内存上限 32 GiB。
- `cargo build --locked --offline --jobs 2`；已有仓库 profile 为 `debug = 0`、`incremental = false`。两种方案各用独立源码目录和独立 `target`。
- 冷编译指对应 `target` 为空；没有清理内核文件缓存，没有下载依赖。编译时长包含所选编译器包及其依赖，不包含整个应用和资产发布。
- CSS 修改是在 `style_parser/src/tokenizer.rs` 尾部加一个注释；JS 修改是在 `parser/src/lexer/comment.rs` 尾部加一个注释。先冷编译，再 CSS 修改，再 JS 修改。语义不变，但 Cargo 必须重新编译修改所在的包。
- 每个编译场景各一次观测。顺序运行，主机有其他协作任务，数据用于选择编译边界，不是硬件无关保证，也不是统计显著性结论。

[构建原始 JSON](compiler-split/builds.json) 保留每次调用的参数、返回码、耗时、CPU 时间和两种 RSS 指标。[采样脚本](compiler-split/run.py) 以 50 ms 间隔读取 `/proc`；存入仓库时只增加了输出目录环境变量，计时和采样方法不变。

| 场景 | 原 35 模块包 / s | JS + CSS 两包 / s | 耗时变化 |
| --- | ---: | ---: | ---: |
| 空 target 编译 | 61.973 | 42.820 | -30.9% |
| CSS 源码修改后 | 31.459 | 14.223 | -54.8% |
| JS 源码修改后 | 31.155 | 17.026 | -45.4% |

| 场景 | 原包最大单进程 RSS / GiB | 拆分最大单进程 RSS / GiB | 原包进程树 RSS 峰值 / GiB | 拆分进程树 RSS 峰值 / GiB |
| --- | ---: | ---: | ---: | ---: |
| 空 target 编译 | 2.407 | 1.529 | 2.441 | 2.569 |
| CSS 修改后 | 2.406 | 1.165 | 2.441 | 1.202 |
| JS 修改后 | 2.415 | 1.519 | 2.457 | 1.558 |

最大单进程 RSS 由 `getrusage(RUSAGE_CHILDREN).ru_maxrss` 取得。进程树峰值是在同一采样时刻汇总 Cargo 及所有后代的 RSS，可能重复统计共享页，不是 PSS，也可能漏掉采样间隔内的瞬时峰值。拆分降低单个 rustc 的峰值，但两个编译器可以并行执行，使冷编译的进程树峰值由 2,621,440,000 增至 2,758,934,528 字节（约 +5.2%）；不能把单进程的下降写成整个构建的内存下降。

## 精确输入与复现

[split.py](compiler-split/split.py) 是实验使用的源码变换：移动上述 8 个模块，调整依赖和宏路径，修改调用处，并给实验 workspace 加上新包的 exclude。实验候选未包含后来独立实施的平台守卫、原生日期实现等改动，因此这里隔离的是编译边界及对应依赖的效果。

归档的根 `Cargo.lock` SHA-256 为 `361588349aa9d4a79a30750673fde2f8cef48f3b1a0282c3e79333b9a7f23ea1`；执行变换和离线 `generate-lockfile` 后，候选锁文件为 `c44e73566dc27f8da04580a46e2f2d92a01c5ddc18dd9b8961923504183f4312`。所有依赖已经在归档中，无需访问上游。

在本仓库根目录、Rust 1.99.0 已安装的环境中执行。脚本生成临时副本，不修改当前 checkout：

```bash
compiler_repo="$PWD"
compiler_bench=$(mktemp -d)
export COMPILER_BENCH_OUTPUT="$compiler_bench/results"
unset CARGO_TARGET_DIR RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
mkdir "$compiler_bench/baseline" "$compiler_bench/split"
git archive 75b525a5ab399e67489e0618e16a31d6ffc0d49c > "$compiler_bench/source.tar"
tar -xf "$compiler_bench/source.tar" -C "$compiler_bench/baseline"
tar -xf "$compiler_bench/source.tar" -C "$compiler_bench/split"
python3 docs/measurements/compiler-split/split.py "$compiler_bench/split" --workspace
(cd "$compiler_bench/split" && cargo +1.99.0 generate-lockfile --offline)

python3 docs/measurements/compiler-split/run.py "$compiler_bench/baseline" baseline-cold blog-assets-compiler
python3 docs/measurements/compiler-split/run.py "$compiler_bench/split" split-cold blog-assets-compiler blog-assets-style-compiler

printf '\n// Compile-boundary measurement: CSS edit.\n' >> "$compiler_bench/baseline/src/foundation/assets/compiler/parts/style_parser/src/tokenizer.rs"
printf '\n// Compile-boundary measurement: CSS edit.\n' >> "$compiler_bench/split/src/foundation/assets/style-compiler/parts/style_parser/src/tokenizer.rs"
python3 docs/measurements/compiler-split/run.py "$compiler_bench/baseline" baseline-css-edit blog-assets-compiler
python3 docs/measurements/compiler-split/run.py "$compiler_bench/split" split-css-edit blog-assets-compiler blog-assets-style-compiler

printf '\n// Compile-boundary measurement: JS edit.\n' >> "$compiler_bench/baseline/src/foundation/assets/compiler/parts/parser/src/lexer/comment.rs"
printf '\n// Compile-boundary measurement: JS edit.\n' >> "$compiler_bench/split/src/foundation/assets/compiler/parts/parser/src/lexer/comment.rs"
python3 docs/measurements/compiler-split/run.py "$compiler_bench/baseline" baseline-js-edit blog-assets-compiler
python3 docs/measurements/compiler-split/run.py "$compiler_bench/split" split-js-edit blog-assets-compiler blog-assets-style-compiler
```

本次执行的构建 stdout/stderr 原始日志位于工作环境 `/workspace/.tooling/bench-compiler/{baseline,split}-{cold,css-edit,js-edit}.log`。可迁移的原始计量数据已保存在上述仓库 JSON 中。

## 源码快照复用实验

资产构建器需要在编译前、编译后和链接后分别重新记录源码，以拒绝构建过程中的源码漂移。这三个独立快照均保留。

原实现在一个快照中，为每个依赖包重新解析根 workspace manifest；记录完整依赖目录后，又对 `foundation_inputs`、builder 输入中的相同文件重复读取和计算 SHA-256。新实现将根 manifest 解析一次，使用 `BTreeSet` 记录已访问包，并在单次快照中按物理路径复用摘要。缓存仅存路径与摘要，不保留完整源码；下一次快照创建空缓存，重新读取内容，不依赖文件长度或 mtime。目录检查和不可变来源证据验证继续执行，不能声称所有读取都已去重。

实验使用上面的基线副本（已包含两个注释探针），只替换 `tools/assets/src/inventory.rs`；没有混入编译器拆分或其他重构。[精确测量补丁](compiler-split/snapshot-cache.patch) 对应文件 SHA-256 `c212da0c8304089ab6fa3dda62ac33b7aadf01725b4dc8c9b3f8b6fd4aafc98f`，最终实现仅后续完善了注释。基线文件 SHA-256 为 `25d8fbe557aed01688ab76a613b0187176050892629fdb52b7429f55d2952b7f`。

[测量消费者](compiler-split/snapshot.rs) 将每次计时限制在 `inventory::snapshot` 调用，输出快照摘要的序列化和哈希在计时之后执行。两版本均使用 dev profile，各连续调用六次；每次处理 13,650 个 foundation 文件、221 个包记录。使用同一目录，目录缓存和系统负载会影响结果，未进行随机交替或冷磁盘测量。

| 调用序号 | 原实现 / s | 单次快照复用 / s |
| --- | ---: | ---: |
| 0 | 13.718353 | 8.643960 |
| 1 | 16.450315 | 8.144605 |
| 2 | 15.124020 | 7.816202 |
| 3 | 13.905181 | 8.155515 |
| 4 | 13.634029 | 7.552076 |
| 5 | 13.788293 | 7.724204 |
| 中位数 | 13.846737 | 7.980404 |

该输入下中位数下降 42.37%。这是快照阶段的观测，未测量其独立 RSS，也不代表完整生产构建耗时变化。[原始 JSON](compiler-split/snapshots.json)、[优化前输出](compiler-split/snapshot-before.log) 与 [优化后输出](compiler-split/snapshot-after.log) 均保留。每个版本六次摘要一致；两版本之间摘要不同，因为构建器源码本身属于快照输入。

继续使用上面的 `compiler_bench` 基线副本可复现：

```bash
snapshot_driver="$compiler_repo/docs/measurements/compiler-split/snapshot.rs"
(cd "$compiler_bench/baseline" && cargo +1.99.0 build --locked --offline --jobs 2 -p blog-assets --lib)
rustc +1.99.0 --edition 2024 "$snapshot_driver" \
  --extern "blog_assets=$compiler_bench/baseline/target/debug/libblog_assets.rlib" \
  -L "dependency=$compiler_bench/baseline/target/debug/deps" -o "$compiler_bench/snapshot-before"
"$compiler_bench/snapshot-before" "$compiler_bench/baseline"

(cd "$compiler_bench/baseline" && git apply "$compiler_repo/docs/measurements/compiler-split/snapshot-cache.patch")
(cd "$compiler_bench/baseline" && cargo +1.99.0 build --locked --offline --jobs 2 -p blog-assets --lib)
rustc +1.99.0 --edition 2024 "$snapshot_driver" \
  --extern "blog_assets=$compiler_bench/baseline/target/debug/libblog_assets.rlib" \
  -L "dependency=$compiler_bench/baseline/target/debug/deps" -o "$compiler_bench/snapshot-after"
"$compiler_bench/snapshot-after" "$compiler_bench/baseline"
```

优化后在独立基线副本执行 `cargo +1.99.0 test --locked --offline --jobs 2 -p blog-assets --lib inventory::tests`，8 项测试全部通过。加强的回归会以同长度内容改写 C 输入并恢复原 mtime，确认下一次快照的包记录与完整 foundation 记录都产生新摘要。源码漂移、完整输入记录、来源证据与公开产物摘要的既有测试继续通过。

落地版本的主包集中 `cargo check` 通过；`cargo test --locked --offline -p blog-assets-compiler --test native-calendar` 的 4 项测试通过，其中时区子进程确实执行了 1 项目标测试。完整产品构建与浏览器验证由项目的统一验证流程记录，不属于上面的隔离性能测量。
