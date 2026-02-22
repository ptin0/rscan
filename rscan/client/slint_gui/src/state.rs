use log::debug;

pub enum GeneralState {
    Idle,
    Programming,
    Measure
}

pub enum AckState {
    Normal,
    Awaiting,
}

struct MState {
    steps: u8,
    lines: u8,
    total_steps: u16,
    current_step: u16,
}

pub struct ClientState {
    pub general: GeneralState,
    pub ack: AckState,
    pub last_pack: Vec<u8>,
    pub consec_error_counter: u8,
    mes_state: MState,
    pub out_file: std::fs::File,
}

impl ClientState {
    pub fn new(out_file: std::fs::File) -> Self {
        ClientState {
            general: GeneralState::Idle,
            ack: AckState::Normal,
            last_pack: Vec::<u8>::new(),
            consec_error_counter: 0,
            mes_state: MState {
                steps: 0,
                lines: 0,
                total_steps: 0,
                current_step: 0,
            },
            out_file,
        }
    }
    pub fn set_steps(&mut self, steps: u8) {
        self.mes_state.steps = steps;
        self.mes_state.total_steps = self.mes_state.steps as u16 * self.mes_state.lines as u16;
        debug!("Points set to: {:?} total steps {:?}", self.mes_state.steps, self.mes_state.total_steps);
    }
    pub fn get_steps(&self) -> u8 {
        self.mes_state.steps
    }
    pub fn set_lines(&mut self, lines: u8) {
        self.mes_state.lines = lines;
        self.mes_state.total_steps = self.mes_state.steps as u16 * self.mes_state.lines as u16;
        debug!("Lines set to: {:?} total steps {:?}", self.mes_state.steps, self.mes_state.total_steps);
    }
    pub fn get_lines(&self) -> u8 {
        self.mes_state.lines
    }
    pub fn get_total_steps(&self) -> u16 {
        self.mes_state.total_steps
    }
    pub fn make_step(&mut self) -> u16 {
        self.mes_state.current_step += 1;
        debug!("Registered step");
        self.mes_state.current_step
    }
    #[allow(dead_code)]
    pub fn reset_step_cnt(&mut self) {
        self.mes_state.current_step = 0;
        debug!("Step counter reset");
    }
    pub fn get_step_cnt(&self) -> u16 {
        self.mes_state.current_step
    }
}