# 代码检查与格式化

本项目使用 Oxlint 检查 JavaScript/TypeScript，Oxfmt 统一格式，TypeScript 7 进行类型检查。无需 ESLint、Antfu 配置或 TypeScript 6 兼容包。

## 常用命令

| 命令 | 作用 |
| --- | --- |
| `pnpm lint` | 只检查；错误或警告使命令失败 |
| `pnpm lint:fix` | 应用 Oxlint 的安全修复 |
| `pnpm fmt` | 应用 Oxfmt 格式化 |
| `pnpm fmt:check` | 只检查格式，不改动文件 |
| `pnpm build` | TypeScript 7 类型检查及 Vite 生产构建 |

CI 依次执行 lint、格式检查和构建。类型感知 lint 尚未启用；如后续引入 `no-floating-promises` 等规则，再添加 `oxlint-tsgolint`。`tsc` 的类型检查不等同于类型感知 lint。

## 配置范围

- `.oxlintrc.json` 保留从旧配置迁移的核心、TypeScript、Import、Unicorn、Node 和 JSDoc 规则及相关选项，并显式启用 React Hook 顺序、JSX key、重复属性、未定义组件检查。
- 使用明确的扩展名集合匹配文件。迁移工具生成的 `?([cm])` 等模式在本次验证中未匹配到 TSX，已转换为可验证的集合模式。
- 未使用变量/导入由 Oxlint 和 TypeScript 检查，不再依赖 `eslint-plugin-unused-imports`。
- `.oxfmtrc.json` 使用单引号、无分号、两空格缩进、100 字符行宽；Oxfmt 负责 import 与 package.json 排序。
- 本次建立的格式基线覆盖前端源码、根目录配置和 CI 工作流。Rust、静态资源、锁文件和文档不在自动格式化范围内，避免混入全仓格式变更。Rust 后续使用 rustfmt 单独管理。
- 局部豁免使用 `oxlint-disable-next-line`，现有示例按钮的原生 alert 行为保留。

## 与 ESLint 的覆盖差异

此次按 [Oxlint 官方迁移流程](https://oxc.rs/docs/guide/usage/linter/migrate-from-eslint.html) 执行。先将 Antfu 的 `ts/` 别名还原为 `@typescript-eslint/`，再转换为原生规则，不加载 JS 插件。

工具转换了 150 条规则，报告跳过 269 条：251 条 JS 插件规则、11 条未实现规则、5 条不支持规则、2 条 nursery 规则。这不是 Antfu 的逐条等价替换。

- 样式、import 排序和 package.json 排序改由 Oxfmt 负责，但排序细节与 Antfu 不保证完全相同。
- Antfu 专有约束、部分 regexp/JSDoc/Node 规则、ESLint 注释管理，以及 JSON/YAML/TOML/Markdown 的插件语义检查不再运行。格式化不替代这些语义检查。
- 未迁移的 `no-undef` 等检查部分由 TypeScript 构建覆盖，但不声称覆盖所有文件和原规则语义。
- 旧配置未启用的类型感知规则不在本次迁移中额外开启。

验证包括：项目 lint、格式检查、生产构建、冻结锁文件安装；临时 TSX 样例中的条件 Hook、缺少 key、无说明 `@ts-ignore` 均导致 Oxlint 失败，验证后已删除样例。应用功能没有因本次工具替换而改动。

依赖版本：Oxlint 1.83.0、Oxfmt 0.68.0、TypeScript 7.0.2。具体解析版本以 `pnpm-lock.yaml` 为准。
