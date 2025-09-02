<!-- markdownlint-disable MD033 MD036 MD041 MD051 MD009 MD032 MD029-->

<p align="center" dir="auto">
    <img style="height:240px;width:240px" src="./public/Irena720.png" alt="FunkyLesson"/>
</p>

<div align="center">

# 🚀 FunkyLesson

**基于 Rust 生态的吉林大学智能选课应用**

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Leptos](https://img.shields.io/badge/frontend-Leptos-red.svg)](https://leptos.dev/)
[![Tauri](https://img.shields.io/badge/framework-Tauri-blue.svg)](https://tauri.app/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/Islatri/funky-lesson)](https://github.com/Islatri/funky-lesson/releases)

*一个现代化、高效、跨平台的选课助手*

![funky-lesson演示](./funky-lesson.gif)

[✨ 项目特色](#-项目特色) • [📥 快速开始](#-快速开始) • [📖 使用指南](#-使用指南) • [🛠️ 技术架构](#️-技术架构) • [📱 移动端支持](#-移动端支持) • [⚖️ 免责声明](#️-免责声明) • [🚀 本地开发](#-本地开发) • [🤝 贡献指南](#-贡献指南) • [📋 版本历史](#-版本历史) • [💖 致谢](#-致谢) • [📈 开发历程](#-开发历程与更新日志)

</div>

---

## ✨ 项目特色

### 🏗️ **现代化技术栈**

- **前端**: Leptos (Rust 编写的响应式 Web 框架)
- **后端**: Tauri (轻量级桌面应用框架)
- **核心**: 纯 Rust 实现，性能卓越
- **跨平台**: 支持 Windows、macOS、Linux 和 Android

### ⚡ **高效选课策略**

- **多线程并发**: 12线程独立轮询，各线程从不同课程开始遍历
- **智能间隔**: 500ms请求间隔，平衡效率与服务器负载
- **自动重连**: 网络中断时自动重连，无需人工干预
- **实时反馈**: 详细的选课状态和错误信息展示

### 🎯 **用户友好设计**

- **开箱即用**: 无需配置环境，下载即可使用
- **图形界面**: 直观的 GUI 操作界面
- **移动支持**: Android APP 已成功构建并测试
- **命令行版**: 提供 TUI 版本供高级用户使用

## 📥 快速开始

### 桌面端 (推荐)

1. 访问 [Releases 页面](https://github.com/Islatri/funky-lesson/releases)
2. 下载最新版本的 `funky-lesson.exe` (Windows) 或对应平台的安装包
3. 双击运行，无需额外配置

### 移动端 (Android)

1. 从 [Releases 页面](https://github.com/Islatri/funky-lesson/releases) 下载 APK 文件
2. 在 Android 设备上安装并运行

### 命令行版本

对于喜欢命令行的用户，可以直接使用核心库：

```bash
git clone https://github.com/ZoneHerobrine/funky_lesson_core.git
cd funky_lesson_core
cargo run <用户名> <密码> <选课批次ID> <是否循环>
```

## 📖 使用指南

### 基本操作流程

1. **配置课程**: 提前去[吉林大学本科生选课网站](https://icourses.jlu.edu.cn/xsxk/profile/index.html)收藏好你要抢的课
2. **启动应用**: 抢课前十分钟左右，运行 FunkyLesson 应用程序
3. **登录账户**: 输入您的学号和教学管理系统密码
4. **选择批次**: 从可用的选课批次中选择目标批次
5. **开始选课**: 点击开始按钮，应用将自动进行选课尝试
6. **监控状态**: 实时查看选课进度和结果，成功后会有提示
7. **一般流程**: 越接近选课时间，每秒发送成功的请求数会逐渐变少，然后选课网站会503一段时间。这个时候，**不要退出软件**，**不要退出软件**，**不要退出软件**，等网络恢复后，软件会自动继续尝试选课。如果在正卡的时候退出，能否再次登陆成功将会成为一个问题，影响选课成功率。

### 高级功能

- **多线程配置**: 默认12线程，可根据需要调整
- **请求间隔**: 默认500ms，平衡效率与服务器负载
- **自动重试**: 网络错误时自动重连
- **状态保存**: 应用会记住您的配置

### 常见问题解答

**Q: 选课开始时出现"请求错误"怎么办？**
A: 这是正常现象。选课刚开始时服务器负载较高，请保持应用运行，网络恢复后会自动继续。

**Q: 多少个线程比较合适？**
A: 推荐使用默认的12线程配置，既能保证效率又不会给服务器造成过大压力。

**Q: 可以同时选多门课吗？**
A: 可以，应用支持同时监控多门课程的选课状态。

## ⚠️ 重要提醒

> **程序不能保证100%选中课程**
> 
> 选课成功率受多种因素影响，包括网络状况、服务器稳定性、课程余量等。建议：
> - 保持应用持续运行
> - 同时准备手动选课作为备选方案
> - 关注官方选课通知

## 🛠️ 技术架构

### 核心技术

```bash
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Leptos 前端   │ ←→ │   Tauri 桌面    │ ←→ │ funky_lesson_   │
│   (Rust WASM)  │    │   应用框架      │    │ core 核心库     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 项目结构

```bash
funky-lesson/
├── src/                 # Leptos 前端源码
├── src-tauri/          # Tauri 后端源码
├── src-proxy/          # 代理服务器
├── note/               # 开发笔记
├── public/             # 静态资源
└── target/             # 编译输出
```

### 构建要求

- **Rust**: 1.85+
- **Tauri CLI**: 最新版本

## 📱 移动端支持

🎉 **最新更新**: Android 移动端应用已成功构建并测试！

### Android 版本特性

- ✅ 完整的选课功能
- ✅ 响应式界面设计
- ✅ 与桌面端相同的核心功能
- ✅ 优化的移动端交互体验

### 安装说明

1. 从 [Releases](https://github.com/Islatri/funky-lesson/releases) 下载最新的 APK 文件
2. 在 Android 设备上允许安装未知来源应用
3. 安装并运行应用

> **注意**: 由于应用未在 Google Play 商店发布，Android 可能会显示安全警告，这是正常现象。

## ⚖️ 免责声明

**请仔细阅读以下免责声明，使用本软件即表示您同意以下条款：**

- 📚 **用途声明**: 本软件仅供学习和研究使用，请勿将其用于任何违反学校或相关法律法规的行为
- 🛡️ **责任声明**: 使用本软件所产生的一切后果均由用户自行承担，开发者不对任何因使用本软件造成的直接或间接损失负责
- 📜 **法规遵守**: 用户在使用本软件的过程中，需遵守所在机构及国家的相关法律法规，如因使用本软件违反相关规定，责任由用户自行承担
- 🏫 **官方声明**: 本软件未经吉林大学官方授权，与吉林大学无任何直接或间接关联

> **使用本程序即代表您完全理解并同意以上免责声明的所有条款**

## 🚀 本地开发

### 环境准备

1. 安装 Rust 工具链:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. 安装 Tauri CLI:

```bash
cargo install tauri-cli
```

3. 安装前端依赖:

```bash
cargo install trunk
```

### 开发命令

```bash
# 克隆项目
git clone https://github.com/Islatri/funky-lesson.git
cd funky-lesson

# 开发模式运行
cargo tauri dev

# 构建生产版本
cargo tauri build

# 构建 Android 版本
cargo tauri android build
```

### 项目脚本

项目提供了便捷的开发脚本：

- **Windows**: `scripts/dev.ps1` - PowerShell 开发脚本
- **Unix/Linux**: `scripts/dev.sh` - Bash 开发脚本
- **Windows (新版)**: `scripts/dev-new.ps1` - 改进的 PowerShell 脚本

## 🤝 贡献指南

我们欢迎任何形式的贡献！

### 贡献方式

1. **Bug 报告**: 发现问题请创建 Issue
2. **功能建议**: 提出新功能想法
3. **代码贡献**: 提交 Pull Request
4. **文档改进**: 完善项目文档

### 开发规范

- 遵循 Rust 官方代码风格 (`cargo fmt`)
- 添加适当的测试用例
- 更新相关文档
- 提交信息使用英文并遵循 [约定式提交](https://www.conventionalcommits.org/)

### Pull Request 流程

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📋 版本历史

### v0.1.0 (最新)

- ✅ **重大突破**: Android 移动端成功构建并测试
- ✅ 完善的桌面端应用
- ✅ 12线程并发选课策略
- ✅ 自动重连机制
- ✅ 现代化 UI 界面

### v0.0.5

- ❌ Android 构建尝试（部分问题）
- ✅ Proxy 服务器实现

### v0.0.4

- ✅ 8线程并发选课策略
- ✅ 稳定的桌面端版本
- ✅ 基础选课功能完善

### v0.0.3

- ✅ Proxy 方案验证成功
- ✅ 核心功能实现

### v0.0.2

- ❌ Web 端 CORS 问题
- 🔄 技术方案调整

## 📄 开源协议

本项目采用 [MIT License](LICENSE) 开源协议。

## 🔗 相关项目

- **[funky_lesson_core](https://github.com/ZoneHerobrine/funky_lesson_core)**: 核心选课逻辑库
- **[Fuck-Lesson](https://github.com/H4ckF0rFun/Fuck-Lesson)**: 原始 Python 实现 (by H4ckF0rFun)

## 💖 致谢

特别感谢以下项目和开发者：

- **[yy4550](https://github.com/yy4550)** - 我的室友，感谢他的陪伴与试用 
- **[MoonWX](https://github.com/MoonWX)** - 复刻自 H4ckF0rFun 的 Fuck-Lesson Python 脚本
- **[H4ckF0rFun](https://github.com/H4ckF0rFun)** - 原始选课脚本 Fuck-Lesson 创作者
- **[背景图片](https://www.pixiv.net/artworks/91403676)** - pid是91403676，很有张力的ジャッジメント
- **[应用图标](https://www.pixiv.net/artworks/96308619)** - pid是96308619，可爱的屑魔女
- **Rust 社区** - 提供优秀的生态系统
- **Tauri 团队** - 现代化的桌面应用框架
- **Leptos 社区** - 强大的 Rust Web 框架

---

<div align="center">

**⭐ 如果这个项目对您有帮助，请给它一个 Star！**

*让更多同学发现这个好用的选课工具* 🎓

</div>

---

## 📈 开发历程与更新日志

### 🎯 v0.1.0 里程碑 (2025年9月)

- **✅ 重大突破**: Android 移动端成功构建并通过测试
- **✅ 技术债务清理**: 解决了之前版本中的关键问题
- **✅ 完整的跨平台支持**: Windows、macOS、Linux、Android 全平台覆盖

### 🔧 v0.0.5 技术探索 (2025年6月)

当时尝试构建 Android 版本遇到了网络请求问题：

- 开发环境 (`android dev`) 运行正常
- 构建 APK 后网络请求失效
- 推测可能与设备浏览器配置相关

**现状**: 这些问题在 v0.1.0 中已经完全解决！

### ⚡ v0.0.4 稳定版本

- **✅ 可靠的桌面端应用
- **✅ 8线程并发策略优化
- **✅ 500ms请求间隔平衡

### 🚧 v0.0.3 代理服务器时代

- **✅ Proxy 服务器方案验证成功
- **✅ 网络请求问题的有效解决方案
- **✅ 为后续版本奠定基础

### 💭 v0.0.2 技术选型反思

当时遇到的主要挑战：

- **CORS 限制**: Web 应用无法直接访问选课网站 API
- **技术路线调整**: 从纯 Web 方案转向 Tauri 混合方案
- **流式传输限制**: `tauri::command` 当时不支持流式传输

**经验总结**: 这次的技术选型调整为项目最终成功奠定了基础，证明了技术决策的重要性。

### 💡 技术演进亮点

1. **从 Web 到 Tauri**: 解决了 CORS 和权限问题
2. **从单线程到多线程**: 大幅提升选课效率
3. **从代理到原生**: 简化了部署和使用流程
4. **从桌面到移动**: 实现了真正的跨平台支持

---

*感谢所有在开发过程中提供帮助和建议的朋友们！* 🙏
