# LeetCode Helper CLI

其他语言版本: [English](README.md), [中文](README_zh.md)

一个用于按题号或关键词输出 Hot100 题解笔记的 Rust CLI。

## 功能
- 按题号查询
- 关键词搜索
- 列出全部题目
- 提示/答案/扩展信息按需显示（`-i` / `-a` / `-e`）
- **算法执行追踪**：真正运行 Java 代码，TUI 交互式逐步显示每行代码 + 变量值变化
- 自动插桩编译执行（无需 AI，无需预写数据），结果缓存秒出
- 自定义输入参数（`--input`），方便验证不同用例
- 彩色渲染支持（Markdown、代码语法高亮）
- 通过 `theme.toml` 自定义配色方案
- 紧凑输出格式
- 使用内置数据集（`data/problems.json` 编译进程序）

## 用法
```bash
# 核心查询
lh 76 -i                      # 显示提示
lh 76 -a                      # 显示答案代码
lh 76 -e                      # 显示扩展信息（示例、图示、API 注释）
lh 76 -i -a -e                # 同时显示提示、答案、扩展

# 算法执行追踪（推荐）
lh 1 -t                       # TUI 交互模式：逐步运行代码，查看每行变量变化
lh 53 -t                      # 最大子数组（Kadane），28 步追踪
lh 1 -t --input "nums=[1,2,3,4,5], target=9"  # 自定义输入，步数由数据决定

# 其他追踪选项
lh 1 -t --trace-text          # 纯文本模式（一次性输出所有步骤）
lh 1 -t --re-trace            # 强制重跑（跳过缓存）

# 通用
lh -l                         # 列出全部题目
lh -s window                  # 关键词搜索
lh 76 -e --theme my-theme.toml  # 使用自定义主题
```

可用参数：
| 参数 | 说明 |
|------|------|
| `-i, --hint` | 显示提示内容 |
| `-a, --answer` | 显示答案代码 |
| `-t, --trace` | **算法执行追踪** — 启动 TUI 交互模式（默认） |
| `--trace-text` | 纯文本输出追踪（需配合 `-t`） |
| `--input <INPUT>` | 自定义输入参数，如 `"nums=[1,2], target=3"` |
| `--re-trace` | 强制重新运行自动追踪（忽略缓存与静态数据） |
| `-e, --extra` | 显示扩展信息（示例、图示、API 注释） |
| `-l, --list` | 列出全部题目 |
| `-s, --search` | 将输入视为关键词搜索 |
| `--theme <FILE>` | 指定主题文件路径（默认读取 `theme.toml`） |

说明：
- 题号查询必须显式指定 `-i` / `-a` / `-e` / `-t` 至少一个。
- **执行追踪** (`-t`) 会真正编译运行题目的 Java 答案代码，捕获每一步的变量值。首次运行需要 1-3 秒，结果自动缓存到 `data/traces/` 目录。
- TUI 模式下，按 `Enter`/`→` 下一步、`←` 上一步、`g` 跳到开头、`G` 跳到结尾、`q` 退出。
- 使用 `--input` 可以任意指定输入参数，步数随数据规模自动变化。
- 默认启用彩色输出。

## 主题配置

默认配置见 `theme.toml`。支持三个配置区域：

### 1. Markdown 高亮
```toml
[markdown]
title = "bright_yellow"       # 标题（题号和标题行）
section_label = "bright_green"  # 小节标签（题目描述、容器选择等）
code_block = "bright_cyan"    # 代码块背景/前景
inline_code = "cyan"         # 行内代码（`code`）
bold = "bright_white"        # 加粗文本（**text**）
link = "bright_blue"         # 链接文本
blockquote = "bright_black"  # 引用（> text）
h1 = "bright_yellow"         # 一级标题
h2 = "bright_yellow"         # 二级标题
h3 = "bright_white"          # 三级标题
list_marker = "green"        # 列表项前缀
```

### 2. API 注释颜色
```toml
[api]
api_name = "bright_magenta"   # API 方法名称
usage_label = "cyan"         # "用法:" 标签
note_label = "yellow"        # "说明:" 标签
```

### 3. 代码语法高亮
```toml
[syntax]
default = "bright_white"     # 默认文本
keyword = "bright_yellow"    # if/for/while 等关键词
type_name = "bright_blue"    # int/String/HashMap 等类型
function = "bright_cyan"     # 函数调用（foo()）
string = "bright_magenta"    # 字符串字面量
number = "bright_red"        # 数字
comment = "green"            # // 和 /* */ 注释
operator = "bright_white"    # +/-/* / 等运算符
punctuation = "bright_black" # 括号、分号等
```

### 4. 执行追踪颜色
```toml
[trace]
header = "bright_green"      # "执行追踪:" 标签
step_number = "bright_cyan"  # "Step 1/9"
separator = "bright_black"   # 分隔线
arrow = "bright_green"       # "→" 前缀
code_line = "bright_white"   # 代码行默认色
var_name = "bright_blue"     # 变量名
var_value = "bright_white"   # 变量值
var_old = "bright_black"     # 旧值 "(旧: ...)"
note = "bright_black"        # 注释/备注
ds_label = "bright_magenta"  # 数据结构标签
ds_highlight = "bright_yellow" # 高亮元素
ds_pointer = "bright_green"  # 指针标记 (^L, ^R)
result = "bright_green"      # 结果高亮
loop_back = "bright_black"   # "[循环]" 标记
```

支持的颜色: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`。

## 输出示例

所有输出采用**紧凑格式**，部分标签和内容在同一行，避免多余的空行。
其中只有 `description` 和 `container` 会把连续空行压缩为单个换行，其它字段保持原始排版。

### -i (提示)
```
题目: 1. 两数之和
提示:
分类: 一、哈希
解法: 单次遍历 + O(1) 查询补数
题目描述: 给定一个整数数组 nums 和一个整数目标值 target，在数组中找出和为 target 的两个整数，返回下标。
    输入：nums = [2,7,11,15], target = 9
    输出：[0,1]
    解释：nums[0] + nums[1] == 9，返回 [0,1]
题目本质: 本质是单次遍历 + O(1) 查询补数的问题。对每个数 x，判断 target - x 是否已出现。
现实类比: 超市收银员找补：知道总价，手里有钱，想知道之前是否有人递过能凑整的另一张钱。
容器选择: 使用 HashMap<Integer, Integer>：
- key 存元素值，value 存其下标
- 需在 O(1) 内判断补数是否存在 → HashMap
- 同时返回下标 → value 必须存下标，非布尔值
三步主线:
- 遍历数组，对每个 nums[i] 计算 complement = target - nums[i]
- 查询 map 中是否已存在 complement，存在则返回 [map.get(complement), i]
- 若不存在，将 nums[i] → i 放入 map，继续遍历（先查后存，避免重复）
复杂度分析: 时间复杂度：O(n)，HashMap 查询/插入均摊 O(1)
空间复杂度：O(n)，最坏情况 map 存储 n 个元素
```

### -e (扩展)
```
题目: 1. 两数之和
扩展信息:
实际示例: 输入：nums = [2,7,11,15], target = 9
输出：[0,1]
解释：nums[0] + nums[1] == 9
图示说明: value -> index 2 -> 0 7 -> 1
API 注释:
  - HashMap 用法: Map<K, V> map = new HashMap<>(); 说明: 基于哈希表，均摊 O(1) 的插入和查找
  - Map.containsKey 用法: map.containsKey(key) 说明: 判断 key 是否存在
  - Map.put 用法: map.put(key, value) 说明: 写入或更新键值对
  - Map.get 用法: map.get(key) 说明: 读取 key 对应的值
```

### -a (答案)
```
题目: 1. 两数之和
答案代码:
    import java.util.*;
    
    class Solution {
        public int[] twoSum(int[] nums, int target) {
            Map<Integer, Integer> map = new HashMap<>();
            for (int i = 0; i < nums.length; i++) {
                int complement = target - nums[i];
                if (map.containsKey(complement)) {
                    return new int[]{map.get(complement), i};
                }
                map.put(nums[i], i);
            }
            return new int[]{};
        }
    }
```

### -t (执行追踪 — TUI 交互模式)

启动 TUI 后按 `Enter` 逐步执行，顶部始终显示 Source（原始输入），底部显示进度：

```
 Source: nums = [3, 5, 1, 8], target = 13
┌ Code ──────────────────────────────────┬ Watch ───────────────────────┐
│  5   Map<Integer,Integer> map = ...;  │ Variables (step 10/10)       │
│  6   for (int i=0; i<nums.length;..) │ Return Value  [1, 3]         │
│ ▶9       return new int[]{...};      │ complement    5               │
│ 10   }                                │ i             3               │
│                                       │ map  {1=2, 3=0, 5=1}        │
│                                       ├ Data ────────────────────────┤
│                                       │ map: { 1:2, 3:0, 5:1 }      │
│                                       │ >>> RESULT <<<               │
├───────────────────────────────────────┴──────────────────────────────┤
│ Step 10/10 (Line 9) │ Enter:next  ←:prev  g:first  G:last  q:quit  │
└──────────────────────────────────────────────────────────────────────┘
```

### --trace-text (纯文本模式)

一次性输出全部步骤，适合管道或重定向：

```
题目: 1. 两数之和
执行追踪:
输入: nums = [3, 5, 1, 8], target = 13
算法: Solution.twoSum
══════════════════════════════════

  Step 1/10  [Line 4]
  ──────────────────────
    → public int[] twoSum(int[] nums, int target) {
    变量:
      nums = [3, 5, 1, 8]
      target = 9

  ...

  Step 10/10  [Line 9]
  ──────────────────────
    → return new int[]{map.get(complement), i};
    变量:
      返回值 = [1, 3]          ← 自动捕获返回表达式
      complement = 5
      i = 3
      map = {1=2, 3=0, 5=1}
    >>> 返回结果 <<<

══════════════════════════════════
```

## 安装与运行
本地运行：

```bash
cargo run --bin lh -- 76 -i
```

构建发布：

```bash
cargo build --release
```

## 数据格式
数据根节点为 `problems`，每题包含：
- `id`, `title`, `category`, `solution`
- `description`, `essence`, `analogy`, `container`, `steps`, `complexity`
- `answer`（Java 代码）
- 可选扩展：`example`, `diagram`, `apiNotes`

## 开发
```bash
cargo test
```
