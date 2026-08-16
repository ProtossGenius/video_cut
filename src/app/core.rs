use eframe::{egui, Frame};
use crate::gui::Page;
use crate::keybinding::{KeymapTrie, KeyParser, KeyEvent, Action, Mode, ParseResult};

pub struct VideoCutApp {
    current_page: Page,
    key_parser: KeyParser,
    mode: Mode,
}

impl VideoCutApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        let mut trie = KeymapTrie::new();
        // 绑定简单的测试按键进行页面跳转
        trie.insert(&[KeyEvent::new("N")], Action::LuaCommand("nav".into()));
        trie.insert(&[KeyEvent::new("M")], Action::LuaCommand("main".into()));
        trie.insert(&[KeyEvent::new("F")], Action::LuaCommand("file".into()));
        trie.insert(&[KeyEvent::new("E")], Action::LuaCommand("editor".into()));

        Self {
            current_page: Page::Navigation,
            key_parser: KeyParser::new(trie),
            mode: Mode::Normal,
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 读取我们拷入的系统字体文件，如果是 TTC 则 egui 会使用它的第一款字体
    let font_bytes = include_bytes!("../../assets/font.ttf");
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(font_bytes).into(),
    );

    // 将这款字体设为所有文本样式的首选
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());
    
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font".to_owned());

    ctx.set_fonts(fonts);
}

impl eframe::App for VideoCutApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        // --- 事件监听与按键拦截 ---
        if ui.ctx().egui_wants_keyboard_input() {
            // 如果用户正在输入文本框等，不要拦截按键
        } else {
            // 监听按键按下事件
            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, .. } = event {
                        // 简化处理：将 egui::Key 转换为字符串
                        let key_str = format!("{:?}", key);
                        // 去掉可能的前缀或包装，egui::Key Debug 输出例如 "N", "ArrowDown"
                        let res = self.key_parser.handle_key(KeyEvent::new(key_str));
                        
                        if let ParseResult::Matched(action, _count) = res {
                            // 执行对应动作
                            if let Action::LuaCommand(cmd) = action {
                                match cmd.as_str() {
                                    "nav" => self.current_page = Page::Navigation,
                                    "main" => self.current_page = Page::MainInterface,
                                    "file" => self.current_page = Page::FileBrowser,
                                    "editor" => self.current_page = Page::Editor,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            });
        }

        // --- 绘制界面 ---
        egui::Panel::top("global_header").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("VideoCut - Developer Preview | Mode: {} | Use Shift+N/M/F/E to switch pages", self.mode.as_str()));
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            match self.current_page {
                Page::Navigation => crate::gui::navigation::show(ui),
                Page::MainInterface => crate::gui::main_interface::show(ui),
                Page::FileBrowser => crate::gui::file_browser::show(ui),
                Page::Editor => crate::gui::editor::show(ui),
            }
        });
    }
}
