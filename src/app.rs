use std::time::{Duration, Instant};

use eframe::egui;
use eframe::egui::Color32;

use crate::document::Document;

pub struct TextEditorApp {
    docs: Vec<Document>,
    active_doc: usize,
    next_doc_id: usize,

    // Поиск / замена
    pub(crate) find_text: String,
    pub(crate) replace_text: String,
    pub(crate) last_replace_count: Option<usize>,

    // Внешний вид
    pub(crate) font_size: f32,
    pub(crate) text_color: Color32,

    // Автосохранение
    autosave_interval: Duration,
    last_autosave: Instant,
}

impl TextEditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut docs = Vec::new();
        docs.push(Document::new_untitled(1));

        Self {
            docs,
            active_doc: 0,
            next_doc_id: 2,
            find_text: String::new(),
            replace_text: String::new(),
            last_replace_count: None,
            font_size: 16.0,
            text_color: Color32::from_rgb(230, 230, 230),
            autosave_interval: Duration::from_secs(60),
            last_autosave: Instant::now(),
        }
    }

    fn current_doc(&self) -> &Document {
        &self.docs[self.active_doc]
    }

    fn current_doc_mut(&mut self) -> &mut Document {
        &mut self.docs[self.active_doc]
    }

    /// Автосохранение всех документов.
    ///
    /// - Если у документа есть путь (`path`), сохраняем в этот файл.
    /// - Если путь отсутствует (Безымянный), делаем autosave_*.txt рядом с бинарником.
    fn handle_autosave(&mut self) {
        if self.last_autosave.elapsed() >= self.autosave_interval {
            for doc in &mut self.docs {
                if !doc.dirty {
                    continue;
                }

                if doc.path.is_some() {
                    // Обычный сохранённый файл — пишем прямо в него
                    if let Err(err) = doc.save() {
                        eprintln!("Ошибка автосохранения {:?}: {err}", doc.title);
                    }
                } else {
                    // Безымянный документ — сохраняем во временный autosave-файл
                    if let Ok(mut dir) = std::env::current_dir() {
                        let filename = format!("autosave_{}.txt", doc.id);
                        dir.push(filename);
                        if let Err(err) = std::fs::write(&dir, &doc.text) {
                            eprintln!("Ошибка автосохранения в {:?}: {err}", dir);
                        } else {
                            // Для автосохранения безымянного файла dirty НЕ сбрасываем,
                            // чтобы было видно, что он ещё не сохранён "по-настоящему".
                            println!("Автосохранение безымянного документа в {:?}", dir);
                        }
                    }
                }
            }

            self.last_autosave = Instant::now();
        }
    }

    /// Меню "Файл"
    fn file_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        use rfd::FileDialog;

        ui.menu_button("Файл", |ui| {
            if ui.button("Новый").clicked() {
                self.docs.push(Document::new_untitled(self.next_doc_id));
                self.active_doc = self.docs.len() - 1;
                self.next_doc_id += 1;
                ui.close_menu(); // deprecated, но работает
            }

            if ui.button("Открыть...").clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    if let Ok(doc) = Document::from_file(self.next_doc_id, path) {
                        self.docs.push(doc);
                        self.active_doc = self.docs.len() - 1;
                        self.next_doc_id += 1;
                    }
                }
                ui.close_menu();
            }

            if ui.button("Сохранить").clicked() {
                let doc = self.current_doc_mut();
                if doc.path.is_some() {
                    let _ = doc.save();
                } else if let Some(path) = FileDialog::new().save_file() {
                    let _ = doc.save_as(path);
                }
                ui.close_menu();
            }

            if ui.button("Сохранить как...").clicked() {
                if let Some(path) = FileDialog::new().save_file() {
                    let _ = self.current_doc_mut().save_as(path);
                }
                ui.close_menu();
            }

            if ui.button("Печать...").clicked() {
                // TODO: реальная печать (через системную команду или PDF)
                println!("Печать пока не реализована");
                ui.close_menu();
            }

            ui.separator();

            if ui.button("Выход").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                ui.close_menu();
            }
        });
    }

    /// Меню "Правка"
    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Правка", |ui| {
            if ui.button("Отменить (Undo)").clicked() {
                self.current_doc_mut().undo();
                ui.close_menu();
            }
            if ui.button("Повторить (Redo)").clicked() {
                self.current_doc_mut().redo();
                ui.close_menu();
            }
        });
    }

    /// Меню "Поиск"
    fn search_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Поиск", |ui| {
            ui.label("Найти:");
            ui.text_edit_singleline(&mut self.find_text);

            ui.label("Заменить на:");
            ui.text_edit_singleline(&mut self.replace_text);

            if ui.button("Заменить всё").clicked() {
                let needle = self.find_text.clone();
                let replacement = self.replace_text.clone();

                let count = self.current_doc_mut().replace_all(&needle, &replacement);
                self.last_replace_count = Some(count);
            }

            if let Some(count) = self.last_replace_count {
                ui.label(format!("Заменено вхождений: {count}"));
            }
        });
    }

    /// Меню "Вид" — размер шрифта, цвет текста, интервал автосохранения
    fn view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Вид", |ui| {
            ui.horizontal(|ui| {
                ui.label("Размер шрифта:");
                ui.add(egui::Slider::new(&mut self.font_size, 10.0..=30.0));
            });

            ui.horizontal(|ui| {
                ui.label("Цвет текста:");
                // 🔧 Вместо кнопки-попапа используем встроенный color picker,
                // который нормально работает внутри меню.
                egui::color_picker::color_picker_color32(
                    ui,
                    &mut self.text_color,
                    egui::color_picker::Alpha::Opaque,
                );
            });

            ui.horizontal(|ui| {
                ui.label("Интервал автосохранения (сек):");
                let mut secs = self.autosave_interval.as_secs() as u32;
                if ui
                    .add(egui::DragValue::new(&mut secs).range(10..=600))
                    .changed()
                {
                    self.autosave_interval = Duration::from_secs(secs as u64);
                }
            });
        });
    }

    /// Вкладки/многодокументный интерфейс
    fn tabs_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let len = self.docs.len();
            let active = self.active_doc;

            let mut to_close: Option<usize> = None;
            let mut new_active: Option<usize> = None;

            for (i, doc) in self.docs.iter().enumerate() {
                let mut label = doc.title.clone();
                if doc.dirty {
                    label.push('*');
                }

                let selected = i == active;
                if ui.selectable_label(selected, label).clicked() {
                    new_active = Some(i);
                }

                if ui.small_button("×").clicked() && len > 1 {
                    to_close = Some(i);
                }
            }

            if let Some(i) = new_active {
                self.active_doc = i;
            }

            if let Some(idx) = to_close {
                self.docs.remove(idx);
                if self.active_doc >= self.docs.len() {
                    self.active_doc = self.docs.len() - 1;
                }
            }
        });
    }

    /// Основное текстовое поле
    fn editor_area(&mut self, ui: &mut egui::Ui) {
        // Сначала снимаем настройки в локальные переменные (чтобы не ругался borrow checker)
        let font_size = self.font_size;
        let text_color = self.text_color;

        let doc = self.current_doc_mut();
        let mut text = doc.text.clone();

        let response = egui::TextEdit::multiline(&mut text)
            .desired_rows(30)
            // Настройка шрифта прямо на виджете:
            .font(egui::FontId::monospace(font_size))
            // Настройка цвета текста прямо на виджете:
            .text_color(text_color)
            .lock_focus(true)
            .desired_width(f32::INFINITY)
            .show(ui);

        if response.response.changed() {
            doc.set_text(text);
        }
    }
}

impl eframe::App for TextEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Верхнее меню
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.file_menu(ui, ctx);
                self.edit_menu(ui);
                self.search_menu(ui);
                self.view_menu(ui);
            });
        });

        // Центральная область: вкладки и редактор
        egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs_bar(ui);
            ui.separator();
            self.editor_area(ui);
        });

        // Автосохранение
        self.handle_autosave();

        // Плавная перерисовка
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
