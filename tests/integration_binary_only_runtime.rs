use std::fs;
use std::path::{Path, PathBuf};

fn read_utf8(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("读取 {path} 失败: {error}"))
}

fn collect_files(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_recursive(root, extension, &mut files);
    files
}

fn collect_files_recursive(root: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("遍历目录 {} 失败: {error}", root.display()))
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, extension, files);
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(extension))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

#[test]
fn runtime_source_tree_does_not_depend_on_python_stack() {
    // 2026-03-22: 新增这个守护测试，原因是客户侧要求只交付 Rust 二进制；目的是真正锁住运行时代码不能回退到 Python/pandas/Jupyter 依赖。
    let banned_tokens = ["python", "pandas", "jupyter", "openpyxl", "pyo3", "cpython"];

    let cargo_toml = read_utf8("Cargo.toml").to_ascii_lowercase();
    for token in banned_tokens {
        assert!(
            !cargo_toml.contains(token),
            "Cargo.toml 不应包含运行时 Python 依赖标记: {token}"
        );
    }

    for path in collect_files(Path::new("src"), "rs") {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("读取 {} 失败: {error}", path.display()))
            .to_ascii_lowercase();
        for token in banned_tokens {
            assert!(
                !content.contains(token),
                "运行时代码 {} 不应包含 Python 运行时依赖标记: {token}",
                path.display()
            );
        }
    }
}

#[test]
fn top_level_skills_explicitly_require_binary_only_runtime() {
    // 2026-03-22: 新增这个守护测试，原因是产品面对普通业务用户；目的是锁住四层 Skill 必须明确“不要求 Python 环境，只接受二进制运行”。
    let required_phrases = ["Rust 二进制", "不依赖 Python", "不要求用户安装 Python"];
    let skill_paths = [
        "skills/excel-orchestrator-v1/SKILL.md",
        "skills/table-processing-v1/SKILL.md",
        "skills/analysis-modeling-v1/SKILL.md",
        "skills/decision-assistant-v1/SKILL.md",
    ];

    for path in skill_paths {
        let content = read_utf8(path);
        for phrase in required_phrases {
            assert!(
                content.contains(phrase),
                "{path} 需要明确声明客户运行时约束，缺少短语: {phrase}"
            );
        }
    }
}
