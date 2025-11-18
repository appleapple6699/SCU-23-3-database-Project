# Database
本 README 按照根目录 `database.txt` 的系统设计说明进行整理。内容涵盖软件架构、业务逻辑、持久层、数据层、接口定义与数据库设计数据字典，便于前后端协作与实现对齐。

**一、软件架构**
- 表现层（通用界面模块）：注册/登录、主页 Dashboard、搜索、系统提示/错误
- 管理员界面：组审核、违规处理、系统通知、用户管理（冻结/解冻）
- 组长界面：任务管理、成员管理、组内通知、数据统计、权限转让
- 组员界面：任务浏览、条目提交、条目审核、通知查看确认、个人中心

**二、业务层**
- 用户管理：注册（重复校验→bcrypt）、登录（验证→发 Token）、冻结/解冻（`IsActive` 与 `UnfreezeDateTime`）
- 组管理：创建组（待审）→管理员审核（0→1/2）；成员加入（`UserGroups.Status` 0→1）；权限转让（事务 `BEGIN→更新权限→COMMIT`）；解散组（状态=2，级联影响相关数据）
- 任务协作：发布任务（校验组长）→通知组员；提交条目（检查截止）→插入 `Entries`；审核条目（校验不同人）→插入 `AuditEntries`；计算审核结果（汇总投票→更新 `Entries.AuditStatus`）
- 通知与确认：发布通知（权限校验→插入 `Notifications`→推送）；确认通知（插入 `NotificationConfirmations`→统计确认率）
- 数据分析（组长）：任务完成率、条目审核率、通知确认率，支持时间/成员/任务维度查询

**三、持久层**
- 目标：参数化 SQL 防注入；封装 CRUD；保证事务与外键约束
- 模块与表：
  - 用户管理：`Users`
  - 组管理：`Groups`, `UserGroups`
  - 任务管理：`Tasks`
  - 条目管理：`Entries`, `AuditEntries`
  - 通知系统：`Notifications`, `NotificationConfirmations`
  - 附件：`Attachments`
  - 活跃统计：多表联查与聚合

**四、数据层**
- 主要表：`Users`, `Groups`, `UserGroups`, `Tasks`, `Entries`, `AuditEntries`, `Notifications`, `NotificationConfirmations`, `Attachments`
- 索引设计：
  - `Users(UserID, GroupID, IsActive)`
  - `Groups(GroupID, Status)`
  - `Tasks(GroupID, Deadline)`
  - `Entries(TaskID, SubmitterID)`
  - `Notifications(GroupID, PublishTime)`
  - `AuditEntries(EntryID, AuditorID)`
- 数据安全与备份：每日全量+每小时增量；异地副本；密码 bcrypt；审计日志；恢复工具
- 性能优化：热点查询缓存；分表归档历史；分页限制；事务批处理减少锁竞争

**五、接口定义（RESTful）**
- 统一返回结构：
  - `error_code: INT`（0 成功，>0 失败）
  - `msg: STRING`（结果描述）
  - `data: OBJECT/ARRAY`（成功返回体）
- 认证与基础接口：
  - 用户注册 `POST /api/users/register`
  - 用户登录 `POST /api/users/login`
  - 修改个人信息 `PATCH /api/users/{id}`
  - 创建组申请 `POST /api/groups`
  - 搜索 `GET /api/search`
- 组员/通用操作：
  - 申请加入组 `POST /api/usergroups/apply`
  - 退出指定组 `DELETE /api/usergroups/{groupId}/leave`
  - 查看任务列表 `GET /api/tasks`
  - 提交条目 `POST /api/entries`
  - 审核他人条目 `POST /api/auditentries`
  - 确认通知 `POST /api/notificationconfirmations`
- 组长接口：
  - 发布/修改/删除任务 `POST/PUT/DELETE /api/tasks{,/id}`
  - 获取待审核申请 `GET /api/usergroups/applications`
  - 审核组员加入 `PATCH /api/usergroups/update`
  - 移除成员 `DELETE /api/usergroups/{groupId}/member/{userId}`
  - 权限转让（事务）`POST /api/groups/{id}/transfer`
  - 发布组内通知 `POST /api/notifications`
  - 查看组内活跃度 `GET /api/groups/{id}/stats`
- 管理员接口：
  - 管理员登录 `POST /api/admin/login`
  - 查看/执行组审核 `GET /api/groups`, `PATCH /api/groups/{id}/status`
  - 冻结/解冻账号 `PATCH /api/users/{id}/freeze`
  - 解散违规组 `DELETE /api/groups/{id}/disband`
  - 发布系统通知 `POST /api/notifications`（`GroupID=0`）

**六、数据库设计（摘要）**
- 表总览与约束：
  - `Users`：用户信息与组归属；一个用户仅能属于一个组
  - `Groups`：团队信息与状态（0待审/1通过/2解散）
  - `UserGroups`：多对多关系，记录权限与状态
  - `Tasks`：组内任务，截止后不可提交
  - `Entries`：任务条目，审核状态动态更新
  - `AuditEntries`：审核记录，禁止自审
  - `Notifications`：系统/组通知（`GroupID=0` 为系统）
  - `NotificationConfirmations`：通知确认记录
  - `Attachments`：附件，`OwnerType` 限定 4 类
  - `AuditLogs`（可选）：操作审计日志
- 数据字典关键字段（精简选摘）：
  - `Users(UserID PK, Password bcrypt, Nickname, GroupID FK, GroupPermission {0/1/2}, IsActive {0/1}, UnfreezeDateTime, CreatedAt)`
  - `Groups(GroupID PK, GroupName UNIQUE, Description, Status {0/1/2}, CreatedTime, CreatedByUserID FK)`
  - `UserGroups(UserGroupID PK, UserID FK, GroupID FK, GroupPermission {0/1/2}, JoinTime, Status {0/1/2})`
  - `Tasks(TaskID PK, GroupID FK, PublisherID FK, Title, Content, PublishTime, Deadline, IsValid {0/1})`
  - `Entries(EntryID PK, TaskID FK, SubmitterID FK, Summary, Content, SubmitTime, AuditStatus {0/1/2/3})`
  - `AuditEntries(AuditEntryID PK, EntryID FK, AuditorID FK ≠ SubmitterID, AuditTime, AuditResult {0/1}, Description)`
  - `Notifications(NotificationID PK, PublisherID FK, GroupID FK/0, PublishTime, Title, Content, IsPinned {0/1})`
  - `NotificationConfirmations(ConfirmationID PK, NotificationID FK, UserID FK, ConfirmTime)`
- `Attachments(AttachmentID PK, GroupID FK, OwnerType ENUM('Notification','Task','Entry','AuditEntry'), OwnerID, UploaderID FK, UploadTime)`
- `AuditLogs(LogID PK, UserID FK, Action, TableName, ActionTime)`

**七、运行与开发**
- 依赖：`Rust 1.75+`、`cargo`、本地 SQLite 文件数据库
- 启动服务：
  - `cargo run --bin server`
  - 默认地址 `127.0.0.1:8080`，默认数据库 `sqlite://data/app.db`
- 初始化管理员：
  - `cargo run --bin seed_admin -- <昵称> <密码>`（默认 `admin admin123`）
  - 管理员登录后可访问管理员接口（需 `IsAdmin=1`）
- 统一返回结构：
  - `error_code: 0` 表示成功，非 0 表示失败
  - `msg: string` 描述信息
  - `data: object|array|null`
- 鉴权：登录后前端自动携带 `Authorization: Bearer <token>`；管理员接口需管理员身份，组长接口需对应组的组长权限
- 示例接口：
  - `POST /api/users/register` 注册用户
  - `POST /api/users/login` 用户登录
  - `PATCH /api/users/{id}` 修改用户信息
  - `POST /api/groups` 创建组申请
  - `GET /api/search` 搜索
  - `GET /api/groups` 组列表
  - `POST /api/tasks` 发布任务
  - `GET /api/tasks` 任务列表
  - `POST /api/usergroups/apply` 申请加入组

**八、代码结构**
- `src/api/` 路由与控制器，统一返回结构与处理逻辑
- `src/db/` 连接池与数据库初始化（建表）
- `src/models.rs` 请求与响应模型
- `src/server.rs` 服务启动入口与依赖注入
- `cmd/server/main.rs` 二进制入口
- `src/storage/`、`src/txn/`、`src/index/` 为后续扩展的数据库引擎模块

**九、前端演示**
- 访问 `http://127.0.0.1:8080/` 基础演示（注册/登录/组/任务/申请加入）
- 管理员面板：`http://127.0.0.1:8080/admin.html`（组审核、冻结/解冻、解散组、系统通知）
- 组长面板：`http://127.0.0.1:8080/leader.html`（审核加入、组内通知、权限转让、活跃统计）
- 组员页面：`http://127.0.0.1:8080/member.html`（查看任务、提交条目、审核条目、确认通知）
- 前端为静态页面，直接随服务同源提供
  - 登录成功后自动保存并携带 `token`
  - 页面右上角显示登录状态并支持退出登录


本 README 计划依据仓库根目录的 `database.txt` 内容生成与维护；当前未检测到该文件。请将 `database.txt` 添加到项目根目录后，我会据此同步更新本 README 的各章节。
一个以 Rust 为主的可扩展数据库学习与实践项目文档。文档涵盖目标、架构、模块划分、接口约定、开发命令、测试与基准、以及迭代路线图，便于快速启动与协作。

## 项目目标
- 构建最小可用的存储引擎与事务系统，随后扩展索引与查询能力
- 在保证正确性的前提下，逐步引入性能优化与工程化能力
- 通过清晰的模块边界与测试，支撑长期演进与重构

## 架构总览
- 存储（Storage）：页管理、变长记录、WAL 日志与数据文件组织
- 索引（Index）：B+ 树为主，面向范围查询与排序；后续可探索 LSM-Tree
- 事务（Txn）：最小 MVCC + 两阶段锁（2PL）实现可串行化隔离；支持读写事务
- 查询（Query）：从 KV/表接口开始，逐步引入解析与执行器的最小子集

## 目录结构（规划）
```
Database/
├─ src/                # 核心代码
│  ├─ storage/         # 页、文件、WAL
│  ├─ index/           # B+Tree 等索引
│  ├─ txn/             # 事务与并发控制
│  ├─ sql/             # 解析与执行（可选）
│  └─ lib.rs           # 对外导出的库接口
├─ cmd/                # 可执行入口（如 demo server/cli）
├─ tests/              # 集成/基准测试
├─ benches/            # Criterion 基准测试
└─ README.md           # 文档（当前文件）
```

## 环境准备
- 安装 Rust 工具链：`rustup`、`cargo`
- 推荐组件：`clippy`（静态检查）、`rustfmt`（格式化）
- 可选：`criterion` 基准框架（通过 `cargo bench` 使用）

## 快速开始
```bash
git clone <your-repo-url>
cd Database

# 构建
cargo build

# 运行示例（若存在 cmd/ 可执行）
cargo run --bin demo

# 测试与静态检查
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all

# 基准测试（如配置了 benches/）
cargo bench
```

## 对外接口（草案）
以库模式暴露最小 KV 与事务接口（Rust 示例）：
```rust
// 打开数据库
let mut db = Database::open(Config::default())?;

// 基本 KV 操作
db.put(b"users:1", b"{name:\"Alice\"}")?;
let v = db.get(b"users:1")?;

// 事务示例
let mut tx = db.begin_txn(TxnOptions::read_write());
tx.put(b"balance:alice", b"100")?;
tx.put(b"balance:bob", b"50")?;
tx.commit()?;
```

约定：
- 所有写入通过 WAL 先写日志保证崩溃可恢复
- 读写在事务内遵循隔离级别（默认可串行化），避免脏读/不可重复读
- 错误统一通过 `Result<T, Error>` 返回，便于上层处理

## 存储引擎设计
- 页（Page）：固定大小块（如 4KB），维护页头、校验与空闲空间
- 记录（Record）：变长，采用槽目录（slot directory）管理，支持插入/删除/更新
- 文件组织：数据文件按段（segment）管理，配合页分配器与空闲列表
- WAL：记录操作的逻辑/物理日志，包含 LSN、校验与回放所需信息
- 恢复：崩溃后按 LSN 回放，保证原子性与持久性（ARIES 思路的简化变体）

## 索引（B+ 树）
- 节点：内部节点存键与子指针，叶子节点存键与记录位置（或值）
- 有序性：支持范围查询与顺序扫描；叶子间通过链表提升区间遍历效率
- 分裂/合并：在插入/删除时维持平衡；采用页层面的分裂策略与最小填充
- 读写并发：读写锁与乐观校验结合，避免长时间阻塞

## 事务与并发
- 事务模型：支持只读与读写事务，提供 `begin/commit/rollback`
- 并发控制：2PL + MVCC（读旧版本，写生成新版本，提交后可见）
- 隔离级别：默认可串行化，可按需放宽到可重复读/读已提交
- 死锁处理：超时/等待图检测（后续迭代可选）

## 查询与表接口（可选）
- 表定义：最小 `CREATE TABLE` 语义，支持主键与基础列类型
- 语句支持：`INSERT/SELECT/UPDATE/DELETE` 的子集
- 执行器：基于迭代器模型（scan → filter → project → sort）
- 解析：手写或使用组合子解析器（后续视需求选择）

## 测试与基准
- 单元测试：覆盖页管理、WAL 回放、B+Tree 基本操作、事务语义
- 集成测试：端到端 KV/表接口流程与并发场景
- 基准测试：写入吞吐、读取延迟、范围查询性能（Criterion）
- 数据集：合成数据 + 小型真实数据样例，统一通过脚本生成

## 开发规范
- 提交信息：以动词开头，简洁描述（如 `Add B+Tree split`）
- 代码风格：`cargo fmt` 格式化、`cargo clippy` 无警告
- 分支策略：`main` 保持可运行，特性在 `feature/*` 分支迭代
- 评审：PR 需附测试与性能影响说明（如涉及关键路径）

## 里程碑
1. 最小 KV：页存储 + WAL + 事务读写
2. B+Tree：索引插入/查找/范围扫描与页分裂
3. 事务增强：MVCC 版本链与冲突检测
4. 查询子集：基本 `SELECT/INSERT` 管线与算子
5. 工程化：快照/压缩、监控指标、更多基准场景

## 许可
建议采用 MIT 或 Apache-2.0 许可。如需要，请在根目录添加 `LICENSE` 文件。