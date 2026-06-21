use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "lh",
    version,
    about = "LeetCode Hot100 题解速查工具",
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    #[arg(help = "题号或关键词")]
    pub query: Option<String>,

    #[arg(short = 'i', long, help = "显示提示内容")]
    pub hint: bool,

    #[arg(short = 'a', long, help = "显示答案代码")]
    pub answer: bool,

    #[arg(short = 'e', long, help = "显示扩展信息（示例、图示、API 说明）")]
    pub extra: bool,

    #[arg(short = 't', long, help = "显示算法执行追踪（TUI 交互模式）")]
    pub trace: bool,

    #[arg(long, help = "以纯文本模式输出追踪（非 TUI，需配合 -t）")]
    pub trace_text: bool,

    #[arg(long, help = "强制重新运行自动追踪（忽略缓存）")]
    pub re_trace: bool,

    #[arg(
        long,
        value_name = "INPUT",
        help = "自定义输入参数，格式: \"name1=val1, name2=val2\"。例如: --input \"nums=[1,2,3], target=5\""
    )]
    pub input: Option<String>,

    #[arg(short = 'l', long, help = "列出全部题目")]
    pub list: bool,

    #[arg(short = 's', long, help = "将输入视为关键词搜索")]
    pub search: bool,

    #[arg(
        long,
        value_name = "FILE",
        help = "语法高亮主题文件路径（TOML），默认尝试加载 ./theme.toml"
    )]
    pub theme: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum Shell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}
