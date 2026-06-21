use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs,
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiNote {
    #[serde(alias = "name")]
    pub api: String,
    #[serde(default)]
    pub usage: String,
    #[serde(default)]
    pub note: String,
}

/// Full execution trace for one sample input of a problem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trace {
    /// Human-readable input description, e.g. "nums = [2,7,11,15], target = 9"
    pub input: String,
    /// Optional short algorithm label displayed in trace header
    #[serde(default)]
    pub algorithm: Option<String>,
    /// Ordered execution steps
    pub steps: Vec<TraceStep>,
}

/// One atomic step in an algorithm execution trace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceStep {
    /// Line number(s) in the answer code block, e.g. "5" or "8-9"
    pub line: String,
    /// The line(s) of code being executed at this step (plain text)
    pub code: String,
    /// Human annotation explaining what happened
    #[serde(default)]
    pub note: Option<String>,
    /// True if this step revisits a loop header
    #[serde(default)]
    pub loop_back: bool,
    /// Variables and their values visible after this step executes
    #[serde(default)]
    pub vars: Vec<TraceVar>,
    /// Data structure visualizations
    #[serde(default)]
    pub ds: Vec<TraceDs>,
    /// True on the final step that produces the answer
    #[serde(default)]
    pub is_result: bool,
    /// Current call stack (method names) at this step
    #[serde(default)]
    pub call_stack: Vec<String>,
}

/// A variable's name and current value at a point in the trace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceVar {
    /// Variable name, e.g. "i", "map", "complement"
    pub name: String,
    /// Current value as a display string, e.g. "0", "{2: 0}", "true"
    pub value: String,
    /// Previous value; when present the renderer shows "name: new (旧: old)"
    #[serde(default)]
    pub old: Option<String>,
}

/// Descriptor for rendering one data structure at a trace step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceDs {
    /// Kind of visualization: "array", "hashmap", "stack", "queue",
    /// "linkedlist", "twopointer", "window", "tree", "ascii"
    #[serde(default)]
    pub kind: Option<String>,
    /// Human-readable label shown above the visualization
    pub label: String,
    /// Pre-rendered ASCII art string (used directly when provided)
    #[serde(default)]
    pub ascii: Option<String>,
    /// Structured data that the renderer converts to ASCII
    #[serde(default)]
    pub data: Option<JsonValue>,
    /// 0-based indices to highlight in the visualization
    #[serde(default)]
    pub highlight: Option<Vec<usize>>,
    /// For "twopointer" and "window": left pointer/edge index
    #[serde(default)]
    pub ptr_left: Option<usize>,
    /// For "twopointer" and "window": right pointer/edge index
    #[serde(default)]
    pub ptr_right: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Problem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub solution: String,
    pub description: String,
    pub essence: String,
    pub analogy: String,
    pub container: String,
    pub steps: Vec<String>,
    pub complexity: String,
    pub answer: Option<String>,
    #[serde(default)]
    pub example: String,
    #[serde(default)]
    pub diagram: String,
    #[serde(default, rename = "apiNotes")]
    pub api_notes: Vec<ApiNote>,
    #[serde(default)]
    pub trace: Option<Trace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Database {
    pub problems: HashMap<String, Problem>,
}

impl Database {
    pub fn from_json_str(input: &str) -> anyhow::Result<Self> {
        let mut db: Database = serde_json::from_str(input)
            .context("解析数据失败")?;

        for (key, problem) in db.problems.iter_mut() {
            if problem.id.trim().is_empty() {
                problem.id = key.clone();
            }
        }

        Ok(db)
    }

    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("读取数据文件失败 {}", path.display()))?;
        Self::from_json_str(&contents)
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Problem> {
        if let Some(problem) = self.problems.get(id) {
            return Some(problem);
        }
        self.problems.values().find(|problem| problem.id == id)
    }

    pub fn search(&self, query: &str) -> Vec<&Problem> {
        let needle = normalize(query);
        let mut results: Vec<&Problem> = self
            .problems
            .values()
            .filter(|problem| problem_matches(problem, &needle))
            .collect();

        results.sort_by(|a, b| compare_ids(&a.id, &b.id));
        results
    }

    pub fn list_sorted(&self) -> Vec<&Problem> {
        let mut items: Vec<&Problem> = self.problems.values().collect();
        items.sort_by(|a, b| compare_ids(&a.id, &b.id));
        items
    }
}

fn normalize(input: &str) -> String {
    input.to_ascii_lowercase()
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_ascii_lowercase().contains(needle)
}

fn problem_matches(problem: &Problem, needle: &str) -> bool {
    contains_ci(&problem.id, needle)
        || contains_ci(&problem.title, needle)
        || contains_ci(&problem.category, needle)
        || contains_ci(&problem.solution, needle)
        || contains_ci(&problem.description, needle)
        || contains_ci(&problem.essence, needle)
        || contains_ci(&problem.analogy, needle)
        || contains_ci(&problem.container, needle)
        || contains_ci(&problem.complexity, needle)
        || contains_ci(&problem.example, needle)
        || contains_ci(&problem.diagram, needle)
        || problem
            .answer
            .as_deref()
            .map(|value| contains_ci(value, needle))
            .unwrap_or(false)
        || problem.steps.iter().any(|item| contains_ci(item, needle))
        || problem.api_notes.iter().any(|item| {
            contains_ci(&item.api, needle)
                || contains_ci(&item.usage, needle)
                || contains_ci(&item.note, needle)
        })
}

fn compare_ids(a: &str, b: &str) -> Ordering {
    let a_num = a.parse::<i64>();
    let b_num = b.parse::<i64>();

    match (a_num, b_num) {
        (Ok(a_val), Ok(b_val)) => a_val.cmp(&b_val).then_with(|| a.cmp(b)),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

        const SAMPLE_JSON: &str = r#"{
    "problems": {
        "3": {
            "id": "3",
            "title": "无重复字符的最长子串",
            "category": "三、滑动窗口",
            "solution": "可变长滑动窗口",
            "description": "给定字符串 s，返回最长无重复子串长度。",
            "essence": "本质是一个可变长滑动窗口问题。",
            "analogy": "点菜不重复的窗口扩张。",
            "container": "使用 HashMap 记录字符频次。",
            "steps": ["右指针扩张", "重复则收缩左边", "更新最大长度"],
            "complexity": "时间复杂度 O(n)，空间复杂度 O(字符集大小)"
        },
        "76": {
            "id": "76",
            "title": "最小覆盖子串",
            "category": "三、滑动窗口",
            "solution": "可变长滑动窗口 + 覆盖计数",
            "description": "给定 s 和 t，返回覆盖 t 的最短子串。",
            "essence": "本质是可变长窗口覆盖问题。",
            "analogy": "找最小背包装齐食材。",
            "container": "使用两个 HashMap 统计需求和窗口。",
            "steps": ["右扩张直到满足", "左收缩最小化", "记录最短结果"],
            "complexity": "时间复杂度 O(n)，空间复杂度 O(字符集大小)"
        }
    }
}"#;

    #[test]
    fn parse_json_database() {
        let db = Database::from_json_str(SAMPLE_JSON).expect("db parse");
        assert_eq!(db.problems.len(), 2);
    }

    #[test]
    fn lookup_by_id() {
        let db = Database::from_json_str(SAMPLE_JSON).expect("db parse");
        let problem = db.get_by_id("76").expect("id 76");
        assert_eq!(problem.title, "最小覆盖子串");
    }

    #[test]
    fn search_by_keyword() {
        let db = Database::from_json_str(SAMPLE_JSON).expect("db parse");
        let results = db.search("覆盖");
        assert!(results.iter().any(|problem| problem.id == "76"));
    }
}
