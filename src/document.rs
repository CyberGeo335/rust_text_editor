use std::fs;
use std::path::PathBuf;

pub struct Document {
    pub id: usize,
    pub path: Option<PathBuf>,
    pub title: String,
    pub text: String,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    pub dirty: bool,
}

impl Document {
    pub fn new_untitled(id: usize) -> Self {
        Self {
            id,
            path: None,
            title: format!("Безымянный {}", id),
            text: String::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty: false,
        }
    }

    pub fn from_file(id: usize, path: PathBuf) -> std::io::Result<Self> {
        let text = fs::read_to_string(&path)?;

        let title = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Документ")
            .to_string();

        Ok(Self {
            id,
            path: Some(path),
            title,
            text,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty: false,
        })
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            fs::write(path, &self.text)?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn save_as(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.path = Some(path);
        self.save()
    }

    /// Устанавливаем новый текст с поддержкой undo/redo
    pub fn set_text(&mut self, new_text: String) {
        if new_text != self.text {
            self.undo_stack.push(self.text.clone());
            self.redo_stack.clear();
            self.text = new_text;
            self.dirty = true;
        }
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.text.clone());
            self.text = prev;
            self.dirty = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.text.clone());
            self.text = next;
            self.dirty = true;
        }
    }

    /// Глобальная замена подстроки.
    /// Возвращает, сколько вхождений было заменено.
    pub fn replace_all(&mut self, needle: &str, replacement: &str) -> usize {
        if needle.is_empty() {
            return 0;
        }
        let count = self.text.matches(needle).count();
        if count > 0 {
            self.set_text(self.text.replace(needle, replacement));
        }
        count
    }
}
