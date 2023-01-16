use cnmo_parse::lparse::level_data::cnmb_types::Cells;

#[derive(strum::Display)]
pub enum Tool {
    Brush,
    Eraser,
    Fill,
    TilePicker,
    Spawners,
}

pub struct EditorData {
    pub selected_tiles: Vec<usize>,
    pub foreground_placing: bool,
    pub light_placing: Option<u8>,
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
    pub cells_history: Vec<Cells>,
    pub selected_spawner: Option<usize>,
}

impl EditorData {
    pub fn new(palette: Vec<[u8; 3]>, gfx_size: (u32, u32), opaques: Vec<Vec<bool>>) -> Self {
        Self {
            selected_tiles: Vec::new(),
            foreground_placing: true,
            palette,
            dt: std::time::Duration::from_secs_f32(f32::EPSILON),
            last_update: std::time::Instant::now(),
            light_placing: None,
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
