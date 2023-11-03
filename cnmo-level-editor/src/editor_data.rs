use cnmo_parse::lparse::level_data::{cnmb_types::Cells, cnms_types::{Spawner, wobj_type::WobjType}};
use eframe::egui;

#[derive(strum::Display)]
pub enum Tool {
    Brush,
    Eraser,
    Fill,
    TilePicker,
    Spawners,
    Light,
}

pub struct EditorData {
    pub selected_tiles: Vec<usize>,
    pub foreground_placing: bool,
    //pub light_placing: Option<u8>,
    pub light_tool_level: u8,
    pub palette: Vec<[u8; 3]>,
    pub dt: std::time::Duration,
    pub time_past: std::time::Duration,
    last_update: std::time::Instant,
    pub gfx_size: (u32, u32),
    pub tool: Tool,
    pub viewer_selection: Option<Cells>,
    pub has_copied_tiles: bool,
    pub opaques: Vec<Vec<bool>>,
    pub current_background: usize,
    pub gray_out_background: bool,
    pub selecting_background_color: bool,
    pub selecting_background_image: bool,
    pub cells_history: Vec<(Cells, Vec<Spawner>)>,
    pub selected_spawner: Option<usize>,
    pub spawner_template: Spawner,
    pub spawner_grid_size: f32,
    pub editing_text: Option<egui::Id>,
    pub game_config_file: cnmo_parse::cnma::Cnma,
    pub info_bar: String,
    pub level_file_name: String,
}

impl EditorData {
    pub fn new(palette: Vec<[u8; 3]>, gfx_size: (u32, u32), opaques: Vec<Vec<bool>>) -> Self {
        Self {
            selected_tiles: Vec::new(),
            foreground_placing: true,
            palette,
            dt: std::time::Duration::from_secs_f32(f32::EPSILON),
            last_update: std::time::Instant::now(),
            light_tool_level: cnmo_parse::lparse::level_data::consts::LIGHT_NORMAL,
            //light_placing: None,
            gfx_size,
            time_past: std::time::Duration::ZERO,
            tool: Tool::Brush,
            viewer_selection: None,
            has_copied_tiles: false,
            opaques,
            current_background: 0,
            gray_out_background: true,
            selecting_background_color: false,
            selecting_background_image: false,
            cells_history: vec![],
            selected_spawner: None,
            spawner_template: Spawner {
                pos: cnmo_parse::lparse::level_data::Point(0.0, 0.0),
                type_data: WobjType::Slime { flying: false },
                spawning_criteria: cnmo_parse::lparse::level_data::cnms_types::SpawningCriteria {
                    spawn_delay_secs: 0.0,
                    mode: cnmo_parse::lparse::level_data::cnms_types::SpawnerMode::MultiAndSingleplayer,
                    max_concurrent_spawns: 0,
                },
                dropped_item: None,
                spawner_group: None,
            },
            spawner_grid_size: 8.0,
            editing_text: None,
            game_config_file: cnmo_parse::cnma::Cnma::from_file("audio.cnma").expect("Expected audio.cnma in current directory!"),
            info_bar: "Welcome to the CNM Online Editor!".to_string(),
            level_file_name: "newlvl".to_string(),
        }
    }

    pub fn reset_selected_tiles(&mut self) {
        self.selected_tiles = vec![];
        self.has_copied_tiles = false;
        self.viewer_selection = None;
    }

    pub fn update_delta_time(&mut self) {
        let now = std::time::Instant::now();
        self.dt = now - self.last_update;
        self.last_update = now;
        self.time_past += self.dt;
    }
}
