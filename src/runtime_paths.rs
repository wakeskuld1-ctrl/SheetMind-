use std::path::PathBuf;

// Resolve the shared runtime root used by local persistence stores.
pub fn workspace_runtime_dir() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("EXCEL_SKILL_RUNTIME_DIR") {
        return Ok(PathBuf::from(path));
    }

    if let Ok(db_path) = std::env::var("EXCEL_SKILL_RUNTIME_DB") {
        let db_path = PathBuf::from(db_path);
        return db_path.parent().map(PathBuf::from).ok_or_else(|| {
            format!(
                "EXCEL_SKILL_RUNTIME_DB `{}` 缺少父目录，无法推导 runtime 根目录",
                db_path.display()
            )
        });
    }

    let current_dir = std::env::current_dir().map_err(|error| error.to_string())?;
    Ok(current_dir.join(".excel_skill_runtime"))
}
