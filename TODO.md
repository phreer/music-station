# Music Station TODO

## 🔴 Critical（性能 / 安全）

- [x] **音频元数据操作使用了阻塞 I/O** — `src/library.rs` 中所有阻塞的音频 I/O 操作已用 `tokio::task::spawn_blocking()` 包装
- [ ] **文件上传无大小限制** — `src/server.rs` 的 `upload_cover()` 和 `upload_lyrics()` 未校验上传大小，可被恶意上传超大文件
- [ ] **流式传输将整个文件读入内存** — `stream_track()` 使用 `file.read_to_end(&mut buffer)` 加载完整音频文件到内存，大文件可能达数百 MB。应改为流式 response
- [ ] **库扫描未并行化** — `src/library.rs` 的 `scan_directory()` 顺序处理每个文件，大型音乐库启动很慢

## 🟠 High Priority（功能完善）

- [ ] **`/tracks` 端点缺少分页** — 当前返回所有 track，万级曲库下性能会有问题。需要 `?page=&limit=` 支持
- [ ] **Album/Artist 集合无缓存** — `get_albums()` 和 `get_artists()` 每次请求都从 tracks 重建 HashMap，需缓存
- [ ] **API 缺少排序和过滤** — 无 `?sort=title`、`?genre=rock` 等查询参数支持
- [ ] **API handler 零测试** — `src/server.rs` 无任何测试；`src/audio.rs`、`src/library.rs`、`src/playlist.rs`、`src/stats.rs` 同样缺失

## 🟡 Medium Priority（功能 / 质量）

- [ ] **OGG 元数据写入未实现** — `src/audio.rs` 中 OGG 的 `write_metadata()`、`set_cover()`、`remove_cover()` 标记为 TODO
- [ ] **Cover 上传无 MIME 类型校验** — 仅信任客户端发送的 content-type header，应做服务端校验
- [ ] **CORS 过于宽松** — `src/server.rs` 使用 `CorsLayer::permissive()` 允许所有来源，生产环境应收紧
- [ ] **日志级别不可配** — 当前日志级别硬编码，应通过环境变量 `RUST_LOG` 或 CLI 参数控制
- [ ] **Web 客户端无障碍** — `static/app.js` 缺少 ARIA 标签、键盘导航支持、alt 文字等

## 🟢 Low Priority（代码质量 / 文档）

- [ ] **`unwrap()` 应替换为 `.expect()`** — `src/main.rs`、`src/server.rs`、`src/lyrics.rs` 中有多处 `unwrap()` 应提供 panic 信息
- [ ] **README.md 过时** — 仍提及 "FLAC and MP3"，实际支持 4 种格式；未记录歌词、封面、播放统计等功能
- [ ] **缺少部署文档** — 无 Docker / systemd 部署指南
- [ ] **数据库 schema 未文档化** — 三个 SQLite 数据库的表结构仅存在于代码中
- [ ] **无增量扫描** — 每次启动都全量扫描，无变更检测机制
