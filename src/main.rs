use macroquad::{
    audio::{PlaySoundParams, Sound, load_sound, play_sound},
    prelude::*,
};

mod animation_data;

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;
const FLOOR_Y: f32 = 535.0;
const LEFT_WALL: f32 = 75.0;
const RIGHT_WALL: f32 = WIDTH - 75.0;
const FIXED_DT: f32 = 1.0 / 60.0;
const MAX_HEALTH: i32 = 100;
const MAX_METER: i32 = 100;
const GRAVITY: f32 = 1_650.0;
const JUMP_SPEED: f32 = 700.0;
const PUSH_HALF: f32 = 42.0;
const COMBO_LINK_FRAMES: u16 = 30;
const COMBO_BUFFER_OPEN: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Character {
    DeeJay,
    Rose,
    FeiLong,
    Cammy,
}

impl Character {
    const ALL: [Self; 4] = [Self::DeeJay, Self::Rose, Self::FeiLong, Self::Cammy];

    fn index(self) -> usize {
        match self {
            Self::DeeJay => 0,
            Self::Rose => 1,
            Self::FeiLong => 2,
            Self::Cammy => 3,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::DeeJay => "DEE JAY",
            Self::Rose => "ROSE",
            Self::FeiLong => "FEI LONG",
            Self::Cammy => "CAMMY",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::DeeJay => Color::from_rgba(255, 132, 48, 255),
            Self::Rose => Color::from_rgba(215, 83, 255, 255),
            Self::FeiLong => Color::from_rgba(255, 151, 58, 255),
            Self::Cammy => Color::from_rgba(91, 238, 193, 255),
        }
    }

    fn special_name(self, variant: u8) -> &'static str {
        match (self, variant) {
            (Self::DeeJay, 0) => "Air Slasher",
            (Self::DeeJay, 1) => "Machine Gun Upper",
            (Self::DeeJay, _) => "Double Rolling Sobat",
            (Self::Rose, 0) => "Soul Spark",
            (Self::Rose, 1) => "Soul Throw",
            (Self::Rose, _) => "Soul Spiral",
            (Self::FeiLong, 0) => "Rekka Chain",
            (Self::FeiLong, 1) => "Flame Rise",
            (Self::FeiLong, _) => "Flying Kick",
            (Self::Cammy, 0) => "Spiral Arrow",
            (Self::Cammy, 1) => "Cannon Spike",
            (Self::Cammy, _) => "Hooligan Rush",
        }
    }

    fn super_name(self) -> &'static str {
        match self {
            Self::DeeJay => "SOBAT CARNIVAL",
            Self::Rose => "AURA SOUL STORM",
            Self::FeiLong => "DRAGON RUSH",
            Self::Cammy => "SPIN DRIVE SMASHER",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Walk,
    WalkBackward,
    Crouch,
    Jump,
    JumpForward,
    JumpBackward,
    Block,
    CrouchBlock,
    Punch,
    PunchMedium,
    PunchHeavy,
    ForwardPunch,
    BackPunch,
    Kick,
    KickMedium,
    KickHeavy,
    ForwardKick,
    BackKick,
    CrouchPunch,
    CrouchForwardPunch,
    CrouchBackPunch,
    CrouchKick,
    CrouchForwardKick,
    CrouchBackKick,
    AirPunch,
    AirForwardPunch,
    AirBackPunch,
    AirKick,
    AirForwardKick,
    AirBackKick,
    Throw,
    Special(u8),
    Super,
    Hit,
    KnockedOut,
    Victory,
}

impl State {
    fn is_attack(self) -> bool {
        matches!(
            self,
            Self::Punch
                | Self::PunchMedium
                | Self::PunchHeavy
                | Self::ForwardPunch
                | Self::BackPunch
                | Self::Kick
                | Self::KickMedium
                | Self::KickHeavy
                | Self::ForwardKick
                | Self::BackKick
                | Self::CrouchPunch
                | Self::CrouchForwardPunch
                | Self::CrouchBackPunch
                | Self::CrouchKick
                | Self::CrouchForwardKick
                | Self::CrouchBackKick
                | Self::AirPunch
                | Self::AirForwardPunch
                | Self::AirBackPunch
                | Self::AirKick
                | Self::AirForwardKick
                | Self::AirBackKick
                | Self::Throw
                | Self::Special(_)
                | Self::Super
        )
    }

    fn is_airborne(self) -> bool {
        matches!(
            self,
            Self::Jump
                | Self::JumpForward
                | Self::JumpBackward
                | Self::AirPunch
                | Self::AirForwardPunch
                | Self::AirBackPunch
                | Self::AirKick
                | Self::AirForwardKick
                | Self::AirBackKick
        )
    }

    fn is_kick(self) -> bool {
        matches!(
            self,
            Self::Kick
                | Self::KickMedium
                | Self::KickHeavy
                | Self::ForwardKick
                | Self::BackKick
                | Self::CrouchKick
                | Self::CrouchForwardKick
                | Self::CrouchBackKick
                | Self::AirKick
                | Self::AirForwardKick
                | Self::AirBackKick
        )
    }

    fn label(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Walk => "WALK FORWARD",
            Self::WalkBackward => "WALK BACKWARD",
            Self::Crouch => "CROUCH",
            Self::Jump => "NEUTRAL JUMP",
            Self::JumpForward => "FORWARD JUMP",
            Self::JumpBackward => "BACK JUMP",
            Self::Block => "BLOCK",
            Self::CrouchBlock => "CROUCH BLOCK",
            Self::Punch => "PUNCH",
            Self::PunchMedium => "PUNCH 2",
            Self::PunchHeavy => "PUNCH 3",
            Self::ForwardPunch => "FORWARD PUNCH",
            Self::BackPunch => "HEAVY PUNCH",
            Self::Kick => "KICK",
            Self::KickMedium => "KICK 2",
            Self::KickHeavy => "KICK 3",
            Self::ForwardKick => "FORWARD KICK",
            Self::BackKick => "HEAVY KICK",
            Self::CrouchPunch => "CROUCH PUNCH",
            Self::CrouchForwardPunch => "CROUCH FORWARD PUNCH",
            Self::CrouchBackPunch => "CROUCH HEAVY PUNCH",
            Self::CrouchKick => "CROUCH KICK",
            Self::CrouchForwardKick => "CROUCH FORWARD KICK",
            Self::CrouchBackKick => "CROUCH HEAVY KICK",
            Self::AirPunch => "AIR PUNCH",
            Self::AirForwardPunch => "AIR FORWARD PUNCH",
            Self::AirBackPunch => "AIR HEAVY PUNCH",
            Self::AirKick => "AIR KICK",
            Self::AirForwardKick => "AIR FORWARD KICK",
            Self::AirBackKick => "AIR HEAVY KICK",
            Self::Throw => "THROW",
            Self::Special(_) => "SPECIAL",
            Self::Super => "SUPER",
            Self::Hit => "HIT",
            Self::KnockedOut => "KNOCKED OUT",
            Self::Victory => "VICTORY",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComboKind {
    Punch,
    Kick,
}

fn combo_state(kind: ComboKind, step: u8) -> State {
    match (kind, step) {
        (ComboKind::Punch, 0) => State::Punch,
        (ComboKind::Punch, 1) => State::PunchMedium,
        (ComboKind::Punch, _) => State::PunchHeavy,
        (ComboKind::Kick, 0) => State::Kick,
        (ComboKind::Kick, 1) => State::KickMedium,
        (ComboKind::Kick, _) => State::KickHeavy,
    }
}

#[derive(Clone, Copy)]
struct Fighter {
    character: Character,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    facing: f32,
    health: i32,
    meter: i32,
    grounded: bool,
    state: State,
    action_frame: u16,
    action_total: u16,
    hit_mask: u8,
    projectile_spawned: bool,
    animation_tick: u32,
    combo_kind: Option<ComboKind>,
    combo_step: u8,
    combo_timer: u16,
    combo_queued: bool,
}

impl Fighter {
    fn new(character: Character, x: f32, facing: f32) -> Self {
        Self {
            character,
            x,
            y: FLOOR_Y,
            vx: 0.0,
            vy: 0.0,
            facing,
            health: MAX_HEALTH,
            meter: 0,
            grounded: true,
            state: State::Idle,
            action_frame: 0,
            action_total: 0,
            hit_mask: 0,
            projectile_spawned: false,
            animation_tick: 0,
            combo_kind: None,
            combo_step: 0,
            combo_timer: 0,
            combo_queued: false,
        }
    }

    fn reset_combo(&mut self) {
        self.combo_kind = None;
        self.combo_step = 0;
        self.combo_timer = 0;
        self.combo_queued = false;
    }

    fn start_action(&mut self, state: State) {
        self.reset_combo();
        self.state = state;
        self.action_frame = 0;
        self.animation_tick = 0;
        self.action_total = attack_spec(self.character, state).map_or(1, |spec| spec.total);
        self.hit_mask = 0;
        self.projectile_spawned = false;

        match (self.character, state) {
            (_, State::Special(1)) => {
                self.vy = -470.0;
                self.grounded = false;
                self.vx = self.facing * 105.0;
            }
            (Character::Cammy, State::Special(0)) => self.vx = self.facing * 430.0,
            (Character::FeiLong, State::Special(0)) => self.vx = self.facing * 315.0,
            (Character::DeeJay, State::Special(2)) => self.vx = self.facing * 285.0,
            (Character::Rose, State::Special(2)) => self.vx = self.facing * 350.0,
            (_, State::Special(2)) => self.vx = self.facing * 410.0,
            (Character::Rose, State::Super) => self.vx = 0.0,
            (_, State::Super) => self.vx = self.facing * 465.0,
            _ => self.vx *= 0.25,
        }
    }

    fn start_combo_stage(&mut self, kind: ComboKind, step: u8) {
        self.start_action(combo_state(kind, step));
        self.combo_kind = Some(kind);
        self.combo_step = step.min(2);
    }

    fn next_combo_step(&self, kind: ComboKind) -> u8 {
        if self.combo_kind == Some(kind) && self.combo_timer > 0 && self.combo_step < 2 {
            self.combo_step + 1
        } else {
            0
        }
    }

    fn finish_combo_stage(&mut self) {
        self.state = State::Idle;
        self.action_frame = 0;
        self.action_total = 0;
        self.animation_tick = 0;
        self.hit_mask = 0;
        self.projectile_spawned = false;
        self.combo_timer = COMBO_LINK_FRAMES;
        self.combo_queued = false;
    }

    fn set_state(&mut self, state: State) {
        if self.state != state {
            self.state = state;
            self.animation_tick = 0;
        }
    }

    fn finish_action(&mut self) {
        self.state = if self.grounded {
            State::Idle
        } else {
            State::Jump
        };
        self.action_frame = 0;
        self.action_total = 0;
        self.animation_tick = 0;
        self.hit_mask = 0;
        self.projectile_spawned = false;
        self.reset_combo();
    }

    fn receive_hit(&mut self, damage: i32, stun: u16, knockback: f32, direction: f32) -> bool {
        self.health = (self.health - damage).max(0);
        self.meter = (self.meter + 8).min(MAX_METER);
        self.vx = direction * knockback;
        self.vy = if damage >= 20 { -180.0 } else { self.vy };
        if self.vy < 0.0 {
            self.grounded = false;
        }
        self.action_frame = 0;
        self.action_total = stun;
        self.animation_tick = 0;
        self.hit_mask = 0;
        self.projectile_spawned = false;
        self.reset_combo();

        if self.health == 0 {
            self.state = State::KnockedOut;
            true
        } else {
            self.state = State::Hit;
            false
        }
    }
}

#[derive(Clone, Copy)]
struct AttackSpec {
    total: u16,
    active_start: u16,
    active_end: u16,
    damage: i32,
    stun: u16,
    range: f32,
    hit_y: f32,
    hit_w: f32,
    hit_h: f32,
    knockback: f32,
    projectile: bool,
    hitstop: u8,
}

fn attack_spec(character: Character, state: State) -> Option<AttackSpec> {
    let spec = match state {
        State::Punch => AttackSpec {
            total: 20,
            active_start: 6,
            active_end: 10,
            damage: 7,
            stun: 16,
            range: 67.0,
            hit_y: 112.0,
            hit_w: 48.0,
            hit_h: 38.0,
            knockback: 190.0,
            projectile: false,
            hitstop: 4,
        },
        State::PunchMedium => AttackSpec {
            total: 25,
            active_start: 8,
            active_end: 13,
            damage: 9,
            stun: 20,
            range: 79.0,
            hit_y: 108.0,
            hit_w: 56.0,
            hit_h: 42.0,
            knockback: 235.0,
            projectile: false,
            hitstop: 5,
        },
        State::PunchHeavy => AttackSpec {
            total: 33,
            active_start: 11,
            active_end: 18,
            damage: 13,
            stun: 27,
            range: 95.0,
            hit_y: 111.0,
            hit_w: 68.0,
            hit_h: 50.0,
            knockback: 345.0,
            projectile: false,
            hitstop: 8,
        },
        State::Kick => AttackSpec {
            total: 27,
            active_start: 9,
            active_end: 15,
            damage: 11,
            stun: 22,
            range: 88.0,
            hit_y: 93.0,
            hit_w: 60.0,
            hit_h: 46.0,
            knockback: 285.0,
            projectile: false,
            hitstop: 6,
        },
        State::KickMedium => AttackSpec {
            total: 32,
            active_start: 10,
            active_end: 17,
            damage: 12,
            stun: 25,
            range: 101.0,
            hit_y: 96.0,
            hit_w: 68.0,
            hit_h: 50.0,
            knockback: 315.0,
            projectile: false,
            hitstop: 7,
        },
        State::KickHeavy => AttackSpec {
            total: 41,
            active_start: 14,
            active_end: 23,
            damage: 17,
            stun: 31,
            range: 119.0,
            hit_y: 102.0,
            hit_w: 82.0,
            hit_h: 58.0,
            knockback: 410.0,
            projectile: false,
            hitstop: 9,
        },
        State::ForwardPunch => AttackSpec {
            total: 27,
            active_start: 8,
            active_end: 14,
            damage: 10,
            stun: 21,
            range: 86.0,
            hit_y: 105.0,
            hit_w: 60.0,
            hit_h: 44.0,
            knockback: 265.0,
            projectile: false,
            hitstop: 6,
        },
        State::BackPunch => AttackSpec {
            total: 32,
            active_start: 10,
            active_end: 17,
            damage: 13,
            stun: 25,
            range: 94.0,
            hit_y: 108.0,
            hit_w: 66.0,
            hit_h: 48.0,
            knockback: 320.0,
            projectile: false,
            hitstop: 8,
        },
        State::ForwardKick => AttackSpec {
            total: 34,
            active_start: 11,
            active_end: 19,
            damage: 14,
            stun: 27,
            range: 108.0,
            hit_y: 96.0,
            hit_w: 72.0,
            hit_h: 52.0,
            knockback: 345.0,
            projectile: false,
            hitstop: 8,
        },
        State::BackKick => AttackSpec {
            total: 39,
            active_start: 13,
            active_end: 22,
            damage: 17,
            stun: 30,
            range: 116.0,
            hit_y: 101.0,
            hit_w: 78.0,
            hit_h: 55.0,
            knockback: 390.0,
            projectile: false,
            hitstop: 9,
        },
        State::CrouchPunch | State::CrouchForwardPunch => AttackSpec {
            total: 22,
            active_start: 6,
            active_end: 11,
            damage: 6,
            stun: 15,
            range: 63.0,
            hit_y: 54.0,
            hit_w: 48.0,
            hit_h: 34.0,
            knockback: 175.0,
            projectile: false,
            hitstop: 4,
        },
        State::CrouchBackPunch => AttackSpec {
            total: 29,
            active_start: 9,
            active_end: 15,
            damage: 11,
            stun: 23,
            range: 79.0,
            hit_y: 61.0,
            hit_w: 58.0,
            hit_h: 39.0,
            knockback: 270.0,
            projectile: false,
            hitstop: 7,
        },
        State::CrouchKick | State::CrouchForwardKick => AttackSpec {
            total: 31,
            active_start: 11,
            active_end: 16,
            damage: 10,
            stun: 28,
            range: 90.0,
            hit_y: 31.0,
            hit_w: 70.0,
            hit_h: 28.0,
            knockback: 250.0,
            projectile: false,
            hitstop: 6,
        },
        State::CrouchBackKick => AttackSpec {
            total: 38,
            active_start: 13,
            active_end: 21,
            damage: 15,
            stun: 32,
            range: 108.0,
            hit_y: 27.0,
            hit_w: 82.0,
            hit_h: 28.0,
            knockback: 330.0,
            projectile: false,
            hitstop: 8,
        },
        State::AirKick | State::AirForwardKick => AttackSpec {
            total: 24,
            active_start: 5,
            active_end: 15,
            damage: 10,
            stun: 21,
            range: 78.0,
            hit_y: 98.0,
            hit_w: 62.0,
            hit_h: 52.0,
            knockback: 245.0,
            projectile: false,
            hitstop: 5,
        },
        State::AirBackKick => AttackSpec {
            total: 30,
            active_start: 7,
            active_end: 18,
            damage: 14,
            stun: 25,
            range: 91.0,
            hit_y: 98.0,
            hit_w: 70.0,
            hit_h: 57.0,
            knockback: 305.0,
            projectile: false,
            hitstop: 7,
        },
        State::AirPunch | State::AirForwardPunch => AttackSpec {
            total: 20,
            active_start: 4,
            active_end: 12,
            damage: 8,
            stun: 18,
            range: 70.0,
            hit_y: 92.0,
            hit_w: 55.0,
            hit_h: 48.0,
            knockback: 215.0,
            projectile: false,
            hitstop: 5,
        },
        State::AirBackPunch => AttackSpec {
            total: 26,
            active_start: 6,
            active_end: 15,
            damage: 11,
            stun: 22,
            range: 80.0,
            hit_y: 96.0,
            hit_w: 62.0,
            hit_h: 52.0,
            knockback: 270.0,
            projectile: false,
            hitstop: 6,
        },
        State::Throw => AttackSpec {
            total: 36,
            active_start: 7,
            active_end: 12,
            damage: 16,
            stun: 32,
            range: 50.0,
            hit_y: 80.0,
            hit_w: 45.0,
            hit_h: 95.0,
            knockback: 390.0,
            projectile: false,
            hitstop: 8,
        },
        State::Special(variant) => {
            let projectile =
                variant == 0 && matches!(character, Character::DeeJay | Character::Rose);
            AttackSpec {
                total: match variant {
                    0 => 37,
                    1 => 34,
                    _ => 42,
                },
                active_start: 9,
                active_end: if variant == 2 { 25 } else { 18 },
                damage: match variant {
                    0 => 13,
                    1 => 14,
                    _ => 15,
                },
                stun: 28,
                range: if variant == 2 { 112.0 } else { 80.0 },
                hit_y: if variant == 1 { 133.0 } else { 82.0 },
                hit_w: if variant == 2 { 84.0 } else { 62.0 },
                hit_h: if variant == 1 { 90.0 } else { 54.0 },
                knockback: 360.0,
                projectile,
                hitstop: 7,
            }
        }
        State::Super => AttackSpec {
            total: 66,
            active_start: 12,
            active_end: 45,
            damage: 26,
            stun: 43,
            range: 145.0,
            hit_y: 94.0,
            hit_w: 135.0,
            hit_h: 110.0,
            knockback: 525.0,
            projectile: character == Character::Rose,
            hitstop: 11,
        },
        _ => return None,
    };
    Some(spec)
}

fn strike_window(_character: Character, _state: State, frame: u16, spec: AttackSpec) -> Option<u8> {
    (frame >= spec.active_start && frame <= spec.active_end).then_some(0)
}

#[derive(Clone, Copy, Default)]
struct Controls {
    horizontal: f32,
    down: bool,
    jump: bool,
    punch: bool,
    kick: bool,
    special: bool,
    super_move: bool,
    nav: i8,
}

#[derive(Default)]
struct InputBuffer {
    horizontal: f32,
    down: bool,
    jump: bool,
    punch: bool,
    kick: bool,
    special: bool,
    super_move: bool,
    nav: i8,
}

#[derive(Clone, Copy)]
struct KeyMap {
    left: KeyCode,
    right: KeyCode,
    up: KeyCode,
    down: KeyCode,
    punch: KeyCode,
    kick: KeyCode,
    special: KeyCode,
    super_move: KeyCode,
}

impl InputBuffer {
    fn sample(&mut self, keys: KeyMap) {
        self.horizontal = axis(keys.left, keys.right);
        self.down = is_key_down(keys.down);
        self.jump |= is_key_pressed(keys.up);
        self.punch |= is_key_pressed(keys.punch);
        self.kick |= is_key_pressed(keys.kick);
        self.special |= is_key_pressed(keys.special);
        self.super_move |= is_key_pressed(keys.super_move);
        if is_key_pressed(keys.left) {
            self.nav = -1;
        }
        if is_key_pressed(keys.right) {
            self.nav = 1;
        }
    }

    fn consume(&mut self) -> Controls {
        let controls = Controls {
            horizontal: self.horizontal,
            down: self.down,
            jump: self.jump,
            punch: self.punch,
            kick: self.kick,
            special: self.special,
            super_move: self.super_move,
            nav: self.nav,
        };
        self.jump = false;
        self.punch = false;
        self.kick = false;
        self.special = false;
        self.super_move = false;
        self.nav = 0;
        controls
    }
}

fn axis(left: KeyCode, right: KeyCode) -> f32 {
    match (is_key_down(left), is_key_down(right)) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

#[derive(Clone, Copy)]
struct Projectile {
    owner: usize,
    x: f32,
    y: f32,
    vx: f32,
    radius: f32,
    hit_radius: f32,
    damage: i32,
    color: Color,
    super_move: bool,
}

#[derive(Clone, Copy)]
struct Spark {
    position: Vec2,
    velocity: Vec2,
    life: f32,
    color: Color,
}

#[derive(Clone, Copy)]
enum Phase {
    Select,
    Intro(u16),
    Fight,
    Knockout { winner: usize, frames: u16 },
    RoundResult { winner: usize, frames: u16 },
    MatchOver { winner: usize },
}

struct Sounds {
    select: Sound,
    punch: Sound,
    kick: Sound,
    special: Sound,
    knockout: Sound,
}

impl Sounds {
    async fn load() -> Self {
        Self {
            select: load_sound("assets/audio/select_002.wav").await.unwrap(),
            punch: load_sound("assets/audio/punch.ogg").await.unwrap(),
            kick: load_sound("assets/audio/kick.ogg").await.unwrap(),
            special: load_sound("assets/audio/confirmation_003.wav")
                .await
                .unwrap(),
            knockout: load_sound("assets/audio/error_004.wav").await.unwrap(),
        }
    }
}

struct Art {
    sheets: [Texture2D; 4],
    portraits: [Texture2D; 4],
}

impl Art {
    async fn load() -> Self {
        Self {
            sheets: [
                load_private_sheet("assets/private/dee_jay.png").await,
                load_private_sheet("assets/private/rose.png").await,
                load_private_sheet("assets/private/fei_long.png").await,
                load_private_sheet("assets/private/cammy.png").await,
            ],
            portraits: [
                load_portrait("assets/private/portraits/dee_jay.png").await,
                load_portrait("assets/private/portraits/rose.png").await,
                load_portrait("assets/private/portraits/fei_long.png").await,
                load_portrait("assets/private/portraits/cammy.png").await,
            ],
        }
    }

    fn sheet(&self, character: Character) -> &Texture2D {
        &self.sheets[character.index()]
    }

    fn portrait(&self, character: Character) -> &Texture2D {
        &self.portraits[character.index()]
    }
}

async fn load_portrait(path: &str) -> Texture2D {
    let texture = load_texture(path)
        .await
        .unwrap_or_else(|error| panic!("Could not load portrait {path}: {error}"));
    texture.set_filter(FilterMode::Linear);
    texture
}

async fn load_private_sheet(path: &str) -> Texture2D {
    let bytes = load_file(path)
        .await
        .unwrap_or_else(|error| panic!("Could not read private sheet {path}: {error}"));
    let mut image = Image::from_file_with_format(&bytes, None)
        .unwrap_or_else(|error| panic!("Could not decode private sheet {path}: {error}"));
    remove_flat_background(&mut image);
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn remove_flat_background(image: &mut Image) {
    let key = image.get_image_data()[0];
    for pixel in image.get_image_data_mut() {
        let distance = pixel[0].abs_diff(key[0]) as u16
            + pixel[1].abs_diff(key[1]) as u16
            + pixel[2].abs_diff(key[2]) as u16;
        if distance <= 6 {
            pixel[3] = 0;
        }
    }
}

#[derive(Clone, Copy)]
struct SpriteFrame {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl SpriteFrame {
    const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    fn rect(self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h)
    }
}

fn animation_frames(character: Character, state: State) -> &'static [u16] {
    match character {
        Character::DeeJay => match state {
            State::Idle => &[10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
            State::Walk => &[25, 26, 27, 28, 29],
            State::WalkBackward => &[30, 31, 32, 33, 34],
            State::Crouch => &[20, 21, 22],
            // Row 9 must stay in its authored order.
            State::Jump | State::JumpForward | State::JumpBackward => &[44, 45, 42, 46, 43, 41],
            State::Block => &[47, 48],
            State::CrouchBlock => &[49, 50, 51, 52],
            State::Punch => &[54, 57, 55],
            State::PunchMedium => &[56, 58, 60],
            State::PunchHeavy => &[59, 53],
            State::ForwardPunch => &[63, 64, 65, 66, 61, 62, 67],
            State::BackPunch => &[58, 60, 59, 53],
            State::Kick => &[72, 68, 70],
            State::KickMedium => &[69, 71, 73],
            State::KickHeavy => &[74, 75, 76, 77, 79, 78],
            State::ForwardKick => &[69, 71, 73],
            State::BackKick => &[74, 75, 76, 77, 79, 78],
            State::CrouchPunch => &[114, 110, 109, 111, 113, 116, 112, 115],
            State::CrouchForwardPunch => &[137, 136, 134, 135],
            State::CrouchBackPunch => &[150, 158, 159, 153, 156, 157, 152, 151, 154, 155],
            State::CrouchKick => &[92, 88, 89, 87, 86, 85, 90, 91, 84],
            State::CrouchForwardKick => &[99, 100, 96, 101, 103, 102, 95, 94, 93, 97, 98],
            State::CrouchBackKick => &[106, 107, 104, 108, 105],
            // Row 30 is Dee Jay's only normal-air family.
            State::AirPunch | State::AirForwardPunch | State::AirBackPunch => &[197, 194],
            State::AirKick | State::AirForwardKick | State::AirBackKick => &[195, 196, 193],
            State::Throw => &[151, 152, 153, 154, 155, 156, 157, 158, 159],
            State::Special(0) => &[216, 217, 218, 219, 220, 221, 222, 223, 224],
            State::Special(1) => &[
                161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174,
            ],
            State::Special(_) => &[
                175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190,
            ],
            State::Super => &[
                161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176,
                177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190,
            ],
            State::Hit => &[
                227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243,
            ],
            State::KnockedOut => &[
                244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255, 256, 257, 258, 259,
                260, 261, 262, 263,
            ],
            State::Victory => &[198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208],
        },
        Character::Rose => match state {
            State::Idle => &[11, 12, 13, 14, 0, 1, 2, 3, 4, 5, 4, 3, 2, 1],
            State::Walk => &[6, 7, 8, 9, 10, 11, 12, 13, 14],
            State::WalkBackward => &[53, 54, 55, 46, 47, 48, 49, 50, 51, 56],
            State::Crouch => &[57, 58, 59, 60],
            State::Jump | State::JumpForward | State::JumpBackward => {
                &[73, 70, 68, 65, 66, 62, 61, 63, 64, 67, 69, 72, 71]
            }
            State::Block => &[309, 310, 311, 312],
            State::CrouchBlock => &[57, 58, 59, 60],
            State::Punch => &[83, 84, 78, 79],
            State::PunchMedium => &[97, 86, 98, 87, 88, 89, 90, 91],
            State::PunchHeavy => &[104, 105, 106, 107, 108, 109],
            State::ForwardPunch => &[97, 86, 98, 87, 88, 89, 90, 91],
            State::BackPunch => &[104, 105, 106, 107, 108, 109],
            State::Kick => &[80, 74, 75, 76, 81, 77, 85, 82],
            State::KickMedium => &[92, 93, 94, 95, 96],
            State::KickHeavy => &[99, 100, 101, 102, 103],
            State::ForwardKick => &[92, 93, 94, 95, 96],
            State::BackKick => &[99, 100, 101, 102, 103],
            State::CrouchPunch => &[113, 114, 115, 116, 117, 118, 119],
            State::CrouchForwardPunch => &[125, 126, 127, 128],
            State::CrouchBackPunch => &[129, 130, 131],
            State::CrouchKick => &[124, 110, 111, 112],
            State::CrouchForwardKick => &[120, 121, 122, 123],
            State::CrouchBackKick => &[132, 133, 134],
            State::AirPunch => &[143, 144, 148, 145, 141, 142, 137, 138, 139, 140],
            State::AirForwardPunch => &[154, 155, 156, 157, 158, 159, 160],
            State::AirBackPunch => &[170, 171, 172, 173, 174],
            State::AirKick => &[149, 150, 152, 151, 153],
            State::AirForwardKick => &[161, 162, 163, 164, 165, 166, 167, 168, 169],
            State::AirBackKick => &[175, 176, 177, 178, 179],
            // Rose's labelled throw row is the victim reaction.
            State::Throw => &[204, 205, 206, 207, 208, 209, 210, 211],
            State::Special(0) => &[233, 234, 235, 236, 237, 238, 239, 240, 241],
            State::Special(1) => &[125, 126, 127, 128, 135],
            State::Special(_) => &[220, 221, 222, 223, 224, 225, 232, 233],
            State::Super => &[
                242, 243, 244, 245, 246, 247, 248, 249, 252, 253, 254, 255, 256, 257, 258, 259,
                260, 261, 262, 263, 264, 265, 266, 267, 268,
            ],
            State::Hit => &[277, 278, 279, 280, 281, 282, 283, 284],
            State::KnockedOut => &[286, 287, 288, 289, 290, 291, 292, 293, 294, 295],
            State::Victory => &[269, 270, 271, 272, 273, 274, 275, 276],
        },
        Character::FeiLong => match state {
            State::Idle => &[0, 5, 1, 2, 3, 4, 3, 2, 1, 5],
            State::Walk => &[38, 44, 45, 39, 40, 41, 42, 43],
            State::WalkBackward => &[51, 52, 53, 46, 47, 48, 49, 50],
            State::Crouch => &[35, 36, 37],
            State::Jump | State::JumpForward | State::JumpBackward => {
                &[59, 66, 63, 67, 60, 61, 64, 62, 65]
            }
            State::Block => &[29, 30, 31],
            State::CrouchBlock => &[32, 33, 34],
            State::Punch => &[84, 76],
            State::PunchMedium => &[77, 78, 79, 80, 74],
            State::PunchHeavy => &[81, 82, 83, 75],
            State::ForwardPunch => &[85, 94, 86, 87, 88, 89, 90, 91, 92, 93],
            State::BackPunch => &[228, 229, 230, 226, 227, 231, 232, 233],
            State::Kick => &[110, 106, 100, 107],
            State::KickMedium => &[108, 109, 101, 102, 103, 104, 105],
            State::KickHeavy => &[118, 119, 120, 121, 111, 112, 113, 114, 115, 116, 117],
            State::ForwardKick => &[118, 119, 120, 121, 111, 112, 113, 114, 115, 116, 117],
            State::BackKick => &[122, 123, 124, 125, 126, 127, 131, 133, 128, 132, 129, 130],
            State::CrouchPunch => &[170, 164, 171, 172, 174, 165, 166, 167, 168, 173, 169, 163],
            State::CrouchForwardPunch | State::CrouchBackPunch => {
                &[170, 164, 171, 172, 174, 165, 166, 167, 168, 173, 169, 163]
            }
            State::CrouchKick => &[140, 141, 134, 135, 142, 143, 136, 137, 138, 139],
            State::CrouchForwardKick => &[145, 146, 147, 148, 144, 149, 150],
            State::CrouchBackKick => &[154, 155, 156, 151, 152, 153, 157, 158, 159, 160, 161, 162],
            State::AirPunch => &[190, 185, 186, 187, 191, 195, 192, 193, 188, 194, 189, 196],
            State::AirForwardPunch => &[208, 212, 215, 209, 210, 213, 214, 211],
            State::AirBackPunch => &[190, 185, 186, 187, 191, 195, 192, 193, 188, 194, 189, 196],
            State::AirKick => &[202, 198, 203, 204, 199, 200, 205, 206, 207, 201, 197],
            State::AirForwardKick => &[216, 217, 218, 223, 224, 219, 220, 221, 225, 222],
            State::AirBackKick => &[271, 272, 273],
            State::Throw => &[208, 212, 215, 209, 210, 213, 214, 211],
            State::Special(0) => &[217, 218, 223, 224, 216, 219, 220, 221, 225, 222],
            State::Special(1) => &[235, 236, 237, 238, 239, 240, 241, 244, 242, 243, 234, 245],
            State::Special(_) => &[
                246, 258, 247, 259, 248, 256, 249, 257, 252, 253, 254, 255, 250, 251,
            ],
            State::Super => &[
                230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245,
                246, 247, 248, 249, 250, 251, 252, 253, 254, 255, 256, 257, 258, 259, 260, 261,
                262, 263, 264, 265, 266, 267, 268, 269, 273, 274, 275, 276, 277, 278, 279,
            ],
            State::Hit => &[
                288, 277, 278, 289, 279, 280, 281, 290, 282, 283, 284, 285, 286, 287,
            ],
            State::KnockedOut => &[311, 313, 314, 317, 319, 320, 321, 315, 318, 316, 312],
            State::Victory => &[6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        },
        Character::Cammy => match state {
            State::Idle => &[37, 38, 33, 34, 35, 36, 35, 34, 33, 38],
            State::Walk => &[39, 40, 45, 46, 47, 48, 41, 42, 43, 44],
            State::WalkBackward => &[53, 54, 55, 56, 57, 49, 50, 51, 58, 52],
            State::Crouch => &[60, 61],
            State::Jump => &[68, 66, 62, 63, 64, 65, 67],
            State::JumpForward => &[74, 72, 69, 70, 71, 73, 75],
            State::JumpBackward => &[80, 78, 76, 77, 79, 81],
            State::Block => &[82, 83],
            State::CrouchBlock => &[84, 85],
            State::Punch => &[86, 87],
            State::PunchMedium => &[88, 89],
            State::PunchHeavy => &[90, 91, 93, 94, 95, 92],
            State::ForwardPunch => &[97, 98, 102, 96, 99, 100, 101],
            State::BackPunch => &[93, 94, 95, 92],
            State::Kick => &[107, 108, 109],
            State::KickMedium => &[110, 103, 104, 105],
            State::KickHeavy => &[111, 106, 112, 113, 114, 115],
            State::ForwardKick => &[
                118, 119, 120, 121, 122, 123, 117, 125, 127, 128, 126, 116, 124, 129,
            ],
            State::BackKick => &[111, 106, 112, 113, 114, 115],
            State::CrouchPunch => &[135, 136, 139, 140],
            State::CrouchForwardPunch => &[137, 138],
            State::CrouchBackPunch => &[132, 130, 131, 133, 134],
            State::CrouchKick => &[149, 150, 151, 152],
            State::CrouchForwardKick => &[143, 141, 144],
            State::CrouchBackKick => &[145, 146, 147, 142, 148],
            State::AirPunch | State::AirForwardPunch | State::AirBackPunch => {
                &[158, 159, 153, 154, 155, 156, 157, 160]
            }
            State::AirKick | State::AirForwardKick | State::AirBackKick => {
                &[165, 166, 161, 162, 163, 164, 167, 168]
            }
            State::Throw => &[200, 202, 201, 203, 204, 206, 205, 207, 208, 210, 211, 209],
            State::Special(0) => &[169, 170, 172, 173, 174, 175, 171],
            State::Special(1) => &[177, 176, 178, 179, 180, 181, 182, 183],
            State::Special(_) => &[
                189, 190, 185, 186, 184, 187, 188, 192, 193, 194, 191, 195, 196,
            ],
            State::Super => &[
                170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185,
                186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199,
            ],
            State::Hit => &[
                226, 231, 227, 232, 233, 228, 224, 234, 225, 229, 230, 235, 237, 236,
            ],
            State::KnockedOut => &[
                253, 251, 249, 245, 250, 252, 246, 247, 248, 254, 257, 255, 256, 258, 260, 259,
            ],
            State::Victory => &[
                261, 262, 263, 264, 265, 266, 267, 268, 273, 274, 269, 270, 271, 272,
            ],
        },
    }
}

fn sprite_frame(fighter: &Fighter) -> SpriteFrame {
    let frames = animation_frames(fighter.character, fighter.state);
    let last = frames.len().saturating_sub(1);
    let index = if fighter.state.is_attack() || fighter.state == State::Hit {
        let total = fighter.action_total.max(1) as usize;
        ((fighter.action_frame as usize * frames.len()) / total).min(last)
    } else if matches!(
        fighter.state,
        State::Jump | State::JumpForward | State::JumpBackward
    ) {
        (fighter.action_frame as usize / 3).min(last)
    } else if fighter.state == State::KnockedOut {
        (fighter.animation_tick as usize / 7).min(last)
    } else if fighter.state == State::Crouch {
        (fighter.animation_tick as usize / 6).min(last)
    } else {
        let ticks_per_frame = match fighter.state {
            State::Walk | State::WalkBackward => 4,
            State::Victory => 7,
            State::Block | State::CrouchBlock => 6,
            _ => 8,
        };
        (fighter.animation_tick as usize / ticks_per_frame) % frames.len()
    };
    animation_data::component_frame(fighter.character, frames[index])
}

fn sprite_scale(character: Character) -> f32 {
    match character {
        Character::DeeJay => 1.8,
        Character::Rose => 2.72,
        Character::FeiLong => 2.05,
        Character::Cammy => 2.12,
    }
}

fn sprite_flip_x(character: Character, state: State, facing: f32) -> bool {
    // Rose's super strip is authored facing left.
    if character == Character::Rose && state == State::Super {
        facing > 0.0
    } else {
        facing < 0.0
    }
}

#[cfg(test)]
fn sheet_dimensions(character: Character) -> (f32, f32) {
    match character {
        Character::DeeJay => (1454.0, 7321.0),
        Character::Rose => (991.0, 3664.0),
        Character::FeiLong => (1344.0, 7777.0),
        Character::Cammy => (1494.0, 4935.0),
    }
}

struct Game {
    fighters: [Fighter; 2],
    picks: [Character; 2],
    locked: [bool; 2],
    wins: [u8; 2],
    round: u8,
    phase: Phase,
    projectiles: Vec<Projectile>,
    sparks: Vec<Spark>,
    hitstop: u8,
    banner: Option<(&'static str, u16, Color)>,
    paused: bool,
}

impl Game {
    fn new() -> Self {
        Self {
            fighters: [
                Fighter::new(Character::DeeJay, 380.0, 1.0),
                Fighter::new(Character::Rose, 900.0, -1.0),
            ],
            picks: [Character::DeeJay, Character::Rose],
            locked: [false, false],
            wins: [0, 0],
            round: 1,
            phase: Phase::Select,
            projectiles: Vec::new(),
            sparks: Vec::new(),
            hitstop: 0,
            banner: None,
            paused: false,
        }
    }

    fn return_to_character_select(&mut self) {
        self.wins = [0, 0];
        self.round = 1;
        self.locked = [false, false];
        self.projectiles.clear();
        self.sparks.clear();
        self.hitstop = 0;
        self.banner = None;
        self.paused = false;
        self.phase = Phase::Select;
    }

    fn handle_global_input(&mut self) {
        if matches!(self.phase, Phase::Select) {
            self.paused = false;
            return;
        }

        if self.paused {
            if is_key_pressed(KeyCode::Enter) {
                self.return_to_character_select();
            } else if is_key_pressed(KeyCode::Escape) {
                self.paused = false;
            }
        } else if is_key_pressed(KeyCode::Escape) {
            self.paused = true;
        }
    }

    fn reset_round(&mut self, preserve_meter: bool) {
        let meters = if preserve_meter {
            [self.fighters[0].meter, self.fighters[1].meter]
        } else {
            [0, 0]
        };
        self.fighters = [
            Fighter::new(self.picks[0], 380.0, 1.0),
            Fighter::new(self.picks[1], 900.0, -1.0),
        ];
        self.fighters[0].meter = meters[0];
        self.fighters[1].meter = meters[1];
        self.projectiles.clear();
        self.sparks.clear();
        self.hitstop = 0;
        self.banner = None;
    }

    fn tick(&mut self, controls: [Controls; 2], sounds: &Sounds) {
        if self.paused {
            return;
        }

        self.update_sparks();
        if let Some((text, frames, color)) = self.banner {
            self.banner = if frames <= 1 {
                None
            } else {
                Some((text, frames - 1, color))
            };
        }

        match self.phase {
            Phase::Select => self.tick_select(controls, sounds),
            Phase::Intro(frames) => {
                if frames <= 1 {
                    self.phase = Phase::Fight;
                } else {
                    self.phase = Phase::Intro(frames - 1);
                }
            }
            Phase::Fight => {
                if self.hitstop > 0 {
                    self.hitstop -= 1;
                    return;
                }
                self.update_facing();
                self.update_fighter(0, controls[0], sounds);
                self.update_fighter(1, controls[1], sounds);
                self.resolve_pushboxes();
                self.update_facing();
                self.update_projectiles(sounds);
                self.resolve_strikes(sounds);
            }
            Phase::Knockout { winner, frames } => {
                self.fighters[1 - winner].animation_tick =
                    self.fighters[1 - winner].animation_tick.wrapping_add(1);
                if frames <= 1 {
                    self.wins[winner] += 1;
                    self.fighters[winner].set_state(State::Victory);
                    if self.wins[winner] >= 2 {
                        self.phase = Phase::MatchOver { winner };
                    } else {
                        self.phase = Phase::RoundResult { winner, frames: 90 };
                    }
                } else {
                    self.phase = Phase::Knockout {
                        winner,
                        frames: frames - 1,
                    };
                }
            }
            Phase::RoundResult { winner, frames } => {
                self.fighters[winner].animation_tick =
                    self.fighters[winner].animation_tick.wrapping_add(1);
                if frames <= 1 {
                    self.round += 1;
                    self.reset_round(true);
                    self.phase = Phase::Intro(95);
                } else {
                    self.phase = Phase::RoundResult {
                        winner,
                        frames: frames - 1,
                    };
                }
            }
            Phase::MatchOver { winner } => {
                self.fighters[winner].animation_tick =
                    self.fighters[winner].animation_tick.wrapping_add(1);
                if is_key_pressed(KeyCode::Enter) {
                    self.return_to_character_select();
                }
            }
        }
    }

    fn tick_select(&mut self, controls: [Controls; 2], sounds: &Sounds) {
        for player in 0..2 {
            if self.locked[player] {
                continue;
            }
            if controls[player].nav != 0 {
                let current = self.picks[player].index() as i8;
                let next = (current + controls[player].nav).rem_euclid(4) as usize;
                self.picks[player] = Character::ALL[next];
                play_once(&sounds.select, 0.55);
            }
            if controls[player].punch {
                self.locked[player] = true;
                play_once(&sounds.select, 0.8);
            }
        }

        if self.locked[0] && self.locked[1] {
            self.reset_round(false);
            self.phase = Phase::Intro(100);
        }
    }

    fn update_facing(&mut self) {
        if self.fighters[0].x <= self.fighters[1].x {
            self.fighters[0].facing = 1.0;
            self.fighters[1].facing = -1.0;
        } else {
            self.fighters[0].facing = -1.0;
            self.fighters[1].facing = 1.0;
        }
    }

    fn update_fighter(&mut self, index: usize, controls: Controls, sounds: &Sounds) {
        let opponent = self.fighters[1 - index];
        let threatened = (opponent.state.is_attack()
            && (opponent.x - self.fighters[index].x).abs() < 235.0)
            || self.projectiles.iter().any(|projectile| {
                projectile.owner == 1 - index
                    && (projectile.x - self.fighters[index].x).abs() < 260.0
            });
        let fighter = &mut self.fighters[index];
        fighter.animation_tick = fighter.animation_tick.wrapping_add(1);

        if matches!(fighter.state, State::KnockedOut | State::Victory) {
            return;
        }

        if !fighter.state.is_attack() && fighter.combo_timer > 0 {
            fighter.combo_timer -= 1;
            if fighter.combo_timer == 0 {
                fighter.reset_combo();
            }
        }

        if matches!(fighter.state, State::Hit) {
            fighter.action_frame += 1;
            fighter.vx *= 0.88;
            if fighter.action_frame >= fighter.action_total {
                fighter.finish_action();
            }
        } else if fighter.state.is_attack() {
            fighter.action_frame += 1;
            if let Some(kind) = fighter.combo_kind {
                let matching_press = match kind {
                    ComboKind::Punch => controls.punch && !controls.kick,
                    ComboKind::Kick => controls.kick && !controls.punch,
                };
                if matching_press
                    && fighter.combo_step < 2
                    && fighter.action_frame >= COMBO_BUFFER_OPEN
                {
                    fighter.combo_queued = true;
                }
            }
            if matches!(fighter.state, State::Special(2) | State::Super) {
                fighter.x += fighter.vx * FIXED_DT;
                fighter.vx *= 0.95;
            }

            if let Some(spec) = attack_spec(fighter.character, fighter.state) {
                if spec.projectile
                    && !fighter.projectile_spawned
                    && fighter.action_frame >= spec.active_start
                {
                    fighter.projectile_spawned = true;
                    let super_move = fighter.state == State::Super;
                    self.projectiles.push(Projectile {
                        owner: index,
                        x: fighter.x + fighter.facing * 65.0,
                        y: fighter.y - 103.0,
                        vx: fighter.facing * if super_move { 560.0 } else { 365.0 },
                        radius: if super_move { 34.0 } else { 20.0 },
                        hit_radius: if super_move { 22.0 } else { 14.0 },
                        damage: spec.damage,
                        color: fighter.character.color(),
                        super_move,
                    });
                    play_once(&sounds.special, if super_move { 0.9 } else { 0.65 });
                }
            }

            if fighter.action_frame >= fighter.action_total {
                if let Some(kind) = fighter.combo_kind {
                    if fighter.combo_queued && fighter.combo_step < 2 {
                        fighter.start_combo_stage(kind, fighter.combo_step + 1);
                    } else {
                        fighter.finish_combo_stage();
                    }
                } else {
                    fighter.finish_action();
                }
            }
        } else {
            if controls.super_move && fighter.meter >= MAX_METER && fighter.grounded {
                fighter.meter = 0;
                fighter.start_action(State::Super);
                self.banner = Some((
                    fighter.character.super_name(),
                    70,
                    fighter.character.color(),
                ));
                play_once(&sounds.special, 1.0);
            } else if controls.special && fighter.grounded {
                let toward = controls.horizontal * fighter.facing > 0.1;
                let variant = if controls.down {
                    1
                } else if toward {
                    2
                } else {
                    0
                };
                fighter.start_action(State::Special(variant));
                self.banner = Some((
                    fighter.character.special_name(variant),
                    38,
                    fighter.character.color(),
                ));
                play_once(&sounds.special, 0.58);
            } else if controls.punch && controls.kick && fighter.grounded {
                fighter.start_action(State::Throw);
            } else if controls.kick {
                let relative = controls.horizontal * fighter.facing;
                if fighter.grounded && !controls.down && relative.abs() <= 0.1 {
                    let step = fighter.next_combo_step(ComboKind::Kick);
                    fighter.start_combo_stage(ComboKind::Kick, step);
                    return;
                }
                let state = if !fighter.grounded {
                    if relative > 0.1 {
                        State::AirForwardKick
                    } else if relative < -0.1 {
                        State::AirBackKick
                    } else {
                        State::AirKick
                    }
                } else if controls.down {
                    if relative > 0.1 {
                        State::CrouchForwardKick
                    } else if relative < -0.1 {
                        State::CrouchBackKick
                    } else {
                        State::CrouchKick
                    }
                } else if relative > 0.1 {
                    State::ForwardKick
                } else if relative < -0.1 {
                    State::BackKick
                } else {
                    State::Kick
                };
                fighter.start_action(state);
            } else if controls.punch {
                let relative = controls.horizontal * fighter.facing;
                if fighter.grounded && !controls.down && relative.abs() <= 0.1 {
                    let step = fighter.next_combo_step(ComboKind::Punch);
                    fighter.start_combo_stage(ComboKind::Punch, step);
                    return;
                }
                fighter.start_action(if !fighter.grounded {
                    if relative > 0.1 {
                        State::AirForwardPunch
                    } else if relative < -0.1 {
                        State::AirBackPunch
                    } else {
                        State::AirPunch
                    }
                } else if controls.down {
                    if relative > 0.1 {
                        State::CrouchForwardPunch
                    } else if relative < -0.1 {
                        State::CrouchBackPunch
                    } else {
                        State::CrouchPunch
                    }
                } else if relative > 0.1 {
                    State::ForwardPunch
                } else if relative < -0.1 {
                    State::BackPunch
                } else {
                    State::Punch
                });
            } else if fighter.grounded && controls.jump {
                fighter.reset_combo();
                fighter.vy = -JUMP_SPEED;
                fighter.grounded = false;
                fighter.vx = controls.horizontal * 185.0;
                fighter.action_frame = 0;
                let relative = controls.horizontal * fighter.facing;
                fighter.set_state(if relative > 0.1 {
                    State::JumpForward
                } else if relative < -0.1 {
                    State::JumpBackward
                } else {
                    State::Jump
                });
            } else if fighter.grounded && threatened && controls.horizontal * fighter.facing < -0.1
            {
                fighter.reset_combo();
                fighter.vx = 0.0;
                fighter.set_state(if controls.down {
                    State::CrouchBlock
                } else {
                    State::Block
                });
            } else if fighter.grounded && controls.down {
                fighter.reset_combo();
                fighter.vx = 0.0;
                fighter.set_state(State::Crouch);
            } else if fighter.grounded {
                if controls.horizontal.abs() > 0.1 {
                    fighter.reset_combo();
                }
                fighter.vx = controls.horizontal * 255.0;
                fighter.set_state(if fighter.vx.abs() > 1.0 {
                    if controls.horizontal * fighter.facing < 0.0 {
                        State::WalkBackward
                    } else {
                        State::Walk
                    }
                } else {
                    State::Idle
                });
            } else {
                fighter.vx = controls.horizontal * 150.0;
                fighter.action_frame = fighter.action_frame.saturating_add(1);
            }
        }

        if !matches!(fighter.state, State::Special(2) | State::Super) {
            fighter.x += fighter.vx * FIXED_DT;
        }
        fighter.x = fighter
            .x
            .clamp(LEFT_WALL + PUSH_HALF, RIGHT_WALL - PUSH_HALF);

        if !fighter.grounded {
            fighter.vy += GRAVITY * FIXED_DT;
            fighter.y += fighter.vy * FIXED_DT;
            if fighter.y >= FLOOR_Y {
                fighter.y = FLOOR_Y;
                fighter.vy = 0.0;
                fighter.grounded = true;
                if fighter.state.is_airborne() {
                    fighter.finish_action();
                }
            }
        }
    }

    fn resolve_pushboxes(&mut self) {
        if can_cross_over(&self.fighters[0]) || can_cross_over(&self.fighters[1]) {
            return;
        }

        let (left, right) = self.fighters.split_at_mut(1);
        let delta = right[0].x - left[0].x;
        let distance = delta.abs();
        let minimum = PUSH_HALF * 2.0;
        if distance < minimum {
            let direction = if delta >= 0.0 { 1.0 } else { -1.0 };
            let separation = (minimum - distance) * 0.5 + 0.01;
            left[0].x -= direction * separation;
            right[0].x += direction * separation;
            left[0].x = left[0]
                .x
                .clamp(LEFT_WALL + PUSH_HALF, RIGHT_WALL - PUSH_HALF);
            right[0].x = right[0]
                .x
                .clamp(LEFT_WALL + PUSH_HALF, RIGHT_WALL - PUSH_HALF);
        }
    }

    fn resolve_strikes(&mut self, sounds: &Sounds) {
        for attacker_index in 0..2 {
            let defender_index = 1 - attacker_index;
            let attacker = self.fighters[attacker_index];
            let Some(spec) = attack_spec(attacker.character, attacker.state) else {
                continue;
            };
            let Some(window) = strike_window(
                attacker.character,
                attacker.state,
                attacker.action_frame,
                spec,
            ) else {
                continue;
            };
            if spec.projectile || attacker.hit_mask & (1 << window) != 0 {
                continue;
            }

            let hitbox = attack_box(&attacker, spec);
            if !rects_overlap(hitbox, hurtbox(&self.fighters[defender_index])) {
                continue;
            }

            self.land_hit(attacker_index, defender_index, spec, window, sounds);
            break;
        }
    }

    fn land_hit(
        &mut self,
        attacker_index: usize,
        defender_index: usize,
        spec: AttackSpec,
        window: u8,
        sounds: &Sounds,
    ) {
        let direction = self.fighters[attacker_index].facing;
        let impact = vec2(
            self.fighters[defender_index].x - direction * 20.0,
            self.fighters[defender_index].y - spec.hit_y,
        );
        self.fighters[attacker_index].hit_mask |= 1 << window;
        self.fighters[attacker_index].meter = (self.fighters[attacker_index].meter
            + if spec.damage >= 20 { 20 } else { 13 })
        .min(MAX_METER);
        let blocked = matches!(
            self.fighters[defender_index].state,
            State::Block | State::CrouchBlock
        );
        let knocked_out = if blocked {
            let defender = &mut self.fighters[defender_index];
            let chip = (spec.damage / 5).max(1);
            defender.health = (defender.health - chip).max(1);
            defender.vx = direction * spec.knockback * 0.28;
            defender.animation_tick = 0;
            false
        } else {
            self.fighters[defender_index].receive_hit(
                spec.damage,
                spec.stun,
                spec.knockback,
                direction,
            )
        };
        self.spawn_sparks(
            impact,
            self.fighters[attacker_index].character.color(),
            spec.damage >= 20,
        );
        self.hitstop = if blocked { 3 } else { spec.hitstop };
        let impact_sound = if self.fighters[attacker_index].state.is_kick() {
            &sounds.kick
        } else {
            &sounds.punch
        };
        play_once(
            impact_sound,
            if blocked {
                0.4
            } else if spec.damage >= 20 {
                0.9
            } else {
                0.62
            },
        );

        if knocked_out {
            play_once(&sounds.knockout, 0.92);
            self.phase = Phase::Knockout {
                winner: attacker_index,
                frames: 120,
            };
        }
    }

    fn update_projectiles(&mut self, sounds: &Sounds) {
        let mut index = 0;
        while index < self.projectiles.len() {
            self.projectiles[index].x += self.projectiles[index].vx * FIXED_DT;
            let projectile = self.projectiles[index];
            let target = 1 - projectile.owner;
            let projectile_rect = projectile_hitbox(&projectile);

            if rects_overlap(projectile_rect, hurtbox(&self.fighters[target])) {
                let blocked = matches!(
                    self.fighters[target].state,
                    State::Block | State::CrouchBlock
                );
                let direction = self.fighters[projectile.owner].facing;
                let knocked_out = if blocked {
                    let defender = &mut self.fighters[target];
                    defender.health = (defender.health - (projectile.damage / 5).max(1)).max(1);
                    defender.vx = direction * 95.0;
                    defender.animation_tick = 0;
                    false
                } else {
                    self.fighters[target].receive_hit(
                        projectile.damage,
                        if projectile.super_move { 42 } else { 27 },
                        if projectile.super_move { 520.0 } else { 340.0 },
                        direction,
                    )
                };
                self.fighters[projectile.owner].meter =
                    (self.fighters[projectile.owner].meter + 15).min(MAX_METER);
                self.spawn_sparks(
                    vec2(projectile.x, projectile.y),
                    projectile.color,
                    projectile.super_move,
                );
                self.hitstop = if blocked {
                    3
                } else if projectile.super_move {
                    11
                } else {
                    7
                };
                play_once(
                    &sounds.punch,
                    if blocked {
                        0.4
                    } else if projectile.super_move {
                        0.92
                    } else {
                        0.67
                    },
                );
                self.projectiles.swap_remove(index);
                if knocked_out {
                    play_once(&sounds.knockout, 0.92);
                    self.phase = Phase::Knockout {
                        winner: projectile.owner,
                        frames: 120,
                    };
                }
                continue;
            }

            if projectile.x < LEFT_WALL - 80.0 || projectile.x > RIGHT_WALL + 80.0 {
                self.projectiles.swap_remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn spawn_sparks(&mut self, position: Vec2, color: Color, large: bool) {
        let count = if large { 20 } else { 11 };
        for i in 0..count {
            let angle = i as f32 * std::f32::consts::TAU / count as f32;
            let speed = if large { 250.0 } else { 165.0 };
            self.sparks.push(Spark {
                position,
                velocity: vec2(angle.cos(), angle.sin()) * speed,
                life: if large { 0.42 } else { 0.25 },
                color: if i % 2 == 0 { YELLOW } else { color },
            });
        }
    }

    fn update_sparks(&mut self) {
        for spark in &mut self.sparks {
            spark.position += spark.velocity * FIXED_DT;
            spark.velocity *= 0.91;
            spark.life -= FIXED_DT;
        }
        self.sparks.retain(|spark| spark.life > 0.0);
    }

    fn draw(&self, art: &Art) {
        draw_stage();
        match self.phase {
            Phase::Select => self.draw_select(art),
            _ => {
                draw_fighter(&self.fighters[0], art);
                draw_fighter(&self.fighters[1], art);
                draw_projectiles(&self.projectiles);
                draw_sparks(&self.sparks);
                self.draw_hud();
                self.draw_phase_text();
            }
        }
        if self.paused {
            self.draw_pause_menu();
        }
    }

    fn draw_pause_menu(&self) {
        draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, Color::from_rgba(2, 5, 13, 190));
        draw_rectangle(
            WIDTH * 0.5 - 230.0,
            HEIGHT * 0.5 - 105.0,
            460.0,
            210.0,
            Color::from_rgba(14, 24, 43, 250),
        );
        draw_rectangle_lines(
            WIDTH * 0.5 - 230.0,
            HEIGHT * 0.5 - 105.0,
            460.0,
            210.0,
            3.0,
            Color::from_rgba(83, 223, 213, 220),
        );
        centered_text("PAUSED", HEIGHT * 0.5 - 35.0, 46, WHITE);
        centered_text(
            "ESC  Resume",
            HEIGHT * 0.5 + 20.0,
            24,
            Color::from_rgba(170, 207, 232, 255),
        );
        centered_text(
            "ENTER  Character Select",
            HEIGHT * 0.5 + 62.0,
            24,
            Color::from_rgba(255, 190, 105, 255),
        );
    }

    fn draw_select(&self, art: &Art) {
        centered_text("CHOOSE YOUR FIGHTERS", 84.0, 48, WHITE);
        centered_text(
            "P1: A/D + F     P2: arrows + K",
            125.0,
            22,
            Color::from_rgba(163, 188, 222, 255),
        );

        for (index, character) in Character::ALL.iter().enumerate() {
            let x = 100.0 + index as f32 * 280.0;
            let y = 210.0;
            let selected_p1 = self.picks[0] == *character;
            let selected_p2 = self.picks[1] == *character;
            let color = character.color();
            draw_rectangle(x, y, 240.0, 265.0, Color::from_rgba(14, 24, 43, 245));
            draw_rectangle_lines(x, y, 240.0, 265.0, 3.0, Color::from_rgba(63, 82, 117, 255));
            draw_rectangle(
                x + 12.0,
                y + 12.0,
                216.0,
                140.0,
                Color::new(color.r, color.g, color.b, 0.12),
            );
            draw_portrait_contained(
                art.portrait(*character),
                Rect::new(x + 20.0, y + 16.0, 200.0, 132.0),
            );
            card_text(character.name(), x + 120.0, y + 174.0, 25, WHITE);
            if selected_p1 {
                draw_rectangle_lines(
                    x - 6.0,
                    y - 6.0,
                    252.0,
                    277.0,
                    5.0,
                    Color::from_rgba(78, 241, 230, 255),
                );
                draw_text(
                    if self.locked[0] { "P1 READY" } else { "P1" },
                    x + 7.0,
                    y - 18.0,
                    20.0,
                    Color::from_rgba(78, 241, 230, 255),
                );
            }
            if selected_p2 {
                draw_rectangle_lines(
                    x - 12.0,
                    y - 12.0,
                    264.0,
                    289.0,
                    4.0,
                    Color::from_rgba(255, 153, 86, 255),
                );
                draw_text(
                    if self.locked[1] { "P2 READY" } else { "P2" },
                    x + 135.0,
                    y - 18.0,
                    20.0,
                    Color::from_rgba(255, 153, 86, 255),
                );
            }
        }

        centered_text(
            "Punch confirms selection",
            545.0,
            20,
            Color::from_rgba(173, 197, 224, 255),
        );
    }

    fn draw_hud(&self) {
        draw_health_meter(54.0, 48.0, 475.0, &self.fighters[0], false);
        draw_health_meter(WIDTH - 529.0, 48.0, 475.0, &self.fighters[1], true);
        card_text(
            &format!("ROUND {}", self.round),
            WIDTH * 0.5,
            72.0,
            25,
            WHITE,
        );

        for player in 0..2 {
            for win in 0..2 {
                let x = if player == 0 {
                    WIDTH * 0.5 - 83.0 - win as f32 * 22.0
                } else {
                    WIDTH * 0.5 + 83.0 + win as f32 * 22.0
                };
                draw_circle(
                    x,
                    92.0,
                    7.0,
                    if self.wins[player] > win {
                        self.fighters[player].character.color()
                    } else {
                        Color::from_rgba(47, 62, 89, 255)
                    },
                );
            }
        }

        if let Some((text, _, color)) = self.banner {
            centered_text(text, 160.0, 27, color);
        }

        #[cfg(debug_assertions)]
        draw_text(
            "STANDING COMBOS R6 - DEBUG BUILD",
            WIDTH * 0.5 - 155.0,
            118.0,
            17.0,
            Color::from_rgba(255, 220, 96, 230),
        );

        draw_text(
            "P1  A/D W/S  F punch  G kick  H special  R super  ESC pause",
            28.0,
            670.0,
            17.0,
            Color::from_rgba(112, 214, 213, 220),
        );
        let right_help = "P2  arrows  K punch  L kick  ; special  O super";
        let width = measure_text(right_help, None, 17, 1.0).width;
        draw_text(
            right_help,
            WIDTH - width - 28.0,
            670.0,
            17.0,
            Color::from_rgba(244, 175, 126, 220),
        );
    }

    fn draw_phase_text(&self) {
        match self.phase {
            Phase::Intro(frames) => {
                if frames > 38 {
                    centered_text(&format!("ROUND {}", self.round), 245.0, 54, WHITE);
                } else {
                    centered_text("FIGHT!", 245.0, 70, YELLOW);
                }
            }
            Phase::Knockout { .. } => {
                centered_text("K.O.!", 250.0, 84, Color::from_rgba(255, 81, 101, 255))
            }
            Phase::RoundResult { winner, .. } => centered_text(
                &format!("PLAYER {} TAKES THE ROUND", winner + 1),
                245.0,
                42,
                self.fighters[winner].character.color(),
            ),
            Phase::MatchOver { winner } => {
                centered_text(
                    &format!("PLAYER {} WINS", winner + 1),
                    235.0,
                    58,
                    self.fighters[winner].character.color(),
                );
                centered_text("PRESS ENTER FOR CHARACTER SELECT", 285.0, 22, WHITE);
            }
            Phase::Select | Phase::Fight => {}
        }
    }
}

fn attack_box(fighter: &Fighter, spec: AttackSpec) -> Rect {
    let center_x = fighter.x + fighter.facing * spec.range;
    Rect::new(
        center_x - spec.hit_w * 0.5,
        fighter.y - spec.hit_y - spec.hit_h * 0.5,
        spec.hit_w,
        spec.hit_h,
    )
}

fn projectile_hitbox(projectile: &Projectile) -> Rect {
    Rect::new(
        projectile.x - projectile.hit_radius,
        projectile.y - projectile.hit_radius,
        projectile.hit_radius * 2.0,
        projectile.hit_radius * 2.0,
    )
}

fn hurtbox(fighter: &Fighter) -> Rect {
    if fighter.state == State::KnockedOut {
        Rect::new(fighter.x - 78.0, fighter.y - 58.0, 156.0, 52.0)
    } else if !fighter.grounded {
        // Ignore empty space below compact air poses.
        Rect::new(fighter.x - 35.0, fighter.y - 166.0, 70.0, 132.0)
    } else if matches!(
        fighter.state,
        State::Crouch
            | State::CrouchBlock
            | State::CrouchPunch
            | State::CrouchForwardPunch
            | State::CrouchBackPunch
            | State::CrouchKick
            | State::CrouchForwardKick
            | State::CrouchBackKick
    ) {
        Rect::new(fighter.x - 42.0, fighter.y - 126.0, 84.0, 120.0)
    } else {
        Rect::new(fighter.x - 37.0, fighter.y - 202.0, 74.0, 196.0)
    }
}

fn can_cross_over(fighter: &Fighter) -> bool {
    !fighter.grounded && fighter.state.is_airborne() && fighter.y <= FLOOR_Y - 64.0
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

fn play_once(sound: &Sound, volume: f32) {
    play_sound(
        sound,
        PlaySoundParams {
            looped: false,
            volume,
        },
    );
}

fn draw_stage() {
    clear_background(Color::from_rgba(7, 11, 22, 255));
    for i in 0..8 {
        let x = 80.0 + i as f32 * 170.0;
        draw_triangle(
            vec2(x, 110.0),
            vec2(x - 160.0, FLOOR_Y),
            vec2(x + 160.0, FLOOR_Y),
            Color::from_rgba(24, 38, 69, 55),
        );
    }
    for y in (125..525).step_by(45) {
        draw_line(
            0.0,
            y as f32,
            WIDTH,
            y as f32,
            1.0,
            Color::from_rgba(33, 52, 82, 120),
        );
    }
    draw_rectangle(
        0.0,
        FLOOR_Y,
        WIDTH,
        HEIGHT - FLOOR_Y,
        Color::from_rgba(12, 20, 36, 255),
    );
    draw_line(
        0.0,
        FLOOR_Y,
        WIDTH,
        FLOOR_Y,
        3.0,
        Color::from_rgba(83, 223, 213, 150),
    );
    draw_line(
        0.0,
        FLOOR_Y + 18.0,
        WIDTH,
        FLOOR_Y + 18.0,
        1.0,
        Color::from_rgba(77, 108, 150, 110),
    );
}

fn draw_fighter(fighter: &Fighter, art: &Art) {
    draw_ellipse(
        fighter.x,
        FLOOR_Y + 7.0,
        if fighter.state == State::KnockedOut {
            105.0
        } else {
            64.0
        },
        12.0,
        0.0,
        Color::from_rgba(0, 0, 0, 125),
    );

    let frame = sprite_frame(fighter);
    let scale = sprite_scale(fighter.character);
    let destination_width = frame.w * scale;
    let destination_height = frame.h * scale;
    let tint = if fighter.state == State::Hit && (fighter.action_frame / 3) % 2 == 0 {
        WHITE
    } else {
        Color::from_rgba(245, 248, 255, 255)
    };
    draw_texture_ex(
        art.sheet(fighter.character),
        fighter.x - destination_width * 0.5,
        fighter.y - destination_height,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(destination_width, destination_height)),
            source: Some(frame.rect()),
            flip_x: sprite_flip_x(fighter.character, fighter.state, fighter.facing),
            ..Default::default()
        },
    );

    #[cfg(debug_assertions)]
    {
        let label = fighter.state.label();
        let width = measure_text(label, None, 15, 1.0).width;
        draw_text(
            label,
            fighter.x - width * 0.5,
            (fighter.y - destination_height - 10.0).max(145.0),
            15.0,
            fighter.character.color(),
        );
    }

    if fighter.state == State::Super {
        let pulse = 36.0 + (fighter.animation_tick as f32 * 0.2).sin().abs() * 28.0;
        draw_circle_lines(
            fighter.x,
            fighter.y - 105.0,
            pulse,
            4.0,
            fighter.character.color(),
        );
    }
}

fn draw_projectiles(projectiles: &[Projectile]) {
    for projectile in projectiles {
        draw_circle(
            projectile.x,
            projectile.y,
            projectile.radius + 8.0,
            Color::new(
                projectile.color.r,
                projectile.color.g,
                projectile.color.b,
                0.18,
            ),
        );
        draw_circle(
            projectile.x,
            projectile.y,
            projectile.radius,
            projectile.color,
        );
        draw_circle(
            projectile.x - projectile.radius * 0.25,
            projectile.y - projectile.radius * 0.25,
            projectile.radius * 0.36,
            WHITE,
        );
    }
}

fn draw_sparks(sparks: &[Spark]) {
    for spark in sparks {
        let alpha = (spark.life * 3.0).clamp(0.0, 1.0);
        draw_circle(
            spark.position.x,
            spark.position.y,
            3.0 + alpha * 4.0,
            Color::new(spark.color.r, spark.color.g, spark.color.b, alpha),
        );
    }
}

fn draw_health_meter(x: f32, y: f32, width: f32, fighter: &Fighter, reverse: bool) {
    draw_text(
        fighter.character.name(),
        x,
        y - 8.0,
        22.0,
        fighter.character.color(),
    );
    draw_rectangle(
        x - 3.0,
        y + 5.0,
        width + 6.0,
        28.0,
        Color::from_rgba(2, 4, 10, 240),
    );
    draw_rectangle(x, y + 8.0, width, 22.0, Color::from_rgba(49, 59, 77, 255));
    let health_width = width * fighter.health as f32 / MAX_HEALTH as f32;
    let health_x = if reverse { x + width - health_width } else { x };
    draw_rectangle(
        health_x,
        y + 8.0,
        health_width,
        22.0,
        fighter.character.color(),
    );

    draw_rectangle(x, y + 37.0, width, 9.0, Color::from_rgba(35, 44, 64, 255));
    let meter_width = width * fighter.meter as f32 / MAX_METER as f32;
    let meter_x = if reverse { x + width - meter_width } else { x };
    draw_rectangle(
        meter_x,
        y + 37.0,
        meter_width,
        9.0,
        if fighter.meter >= MAX_METER {
            YELLOW
        } else {
            Color::from_rgba(73, 146, 255, 255)
        },
    );
    if fighter.meter >= MAX_METER {
        draw_text(
            "SUPER",
            if reverse { x } else { x + width - 57.0 },
            y + 65.0,
            16.0,
            YELLOW,
        );
    }
}

fn centered_text(text: &str, y: f32, size: u16, color: Color) {
    let measure = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        WIDTH * 0.5 - measure.width * 0.5,
        y,
        size as f32,
        color,
    );
}

fn card_text(text: &str, x: f32, y: f32, size: u16, color: Color) {
    let measure = measure_text(text, None, size, 1.0);
    draw_text(text, x - measure.width * 0.5, y, size as f32, color);
}

fn draw_portrait_contained(texture: &Texture2D, bounds: Rect) {
    let scale = (bounds.w / texture.width()).min(bounds.h / texture.height());
    let width = texture.width() * scale;
    let height = texture.height() * scale;
    draw_texture_ex(
        texture,
        bounds.x + (bounds.w - width) * 0.5,
        bounds.y + bounds.h - height,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            ..Default::default()
        },
    );
}

fn sample_inputs(buffers: &mut [InputBuffer; 2]) {
    buffers[0].sample(KeyMap {
        left: KeyCode::A,
        right: KeyCode::D,
        up: KeyCode::W,
        down: KeyCode::S,
        punch: KeyCode::F,
        kick: KeyCode::G,
        special: KeyCode::H,
        super_move: KeyCode::R,
    });
    buffers[1].sample(KeyMap {
        left: KeyCode::Left,
        right: KeyCode::Right,
        up: KeyCode::Up,
        down: KeyCode::Down,
        punch: KeyCode::K,
        kick: KeyCode::L,
        special: KeyCode::Semicolon,
        super_move: KeyCode::O,
    });
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Private Alpha Fighter".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let art = Art::load().await;
    let sounds = Sounds::load().await;
    let mut game = Game::new();
    let mut buffers = [InputBuffer::default(), InputBuffer::default()];
    let mut accumulator = 0.0;

    loop {
        accumulator += get_frame_time().min(0.1);
        game.handle_global_input();
        sample_inputs(&mut buffers);
        while accumulator >= FIXED_DT {
            game.tick([buffers[0].consume(), buffers[1].consume()], &sounds);
            accumulator -= FIXED_DT;
        }
        game.draw(&art);
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sprite_crops_fit_their_source_sheet() {
        let states = [
            State::Idle,
            State::Walk,
            State::WalkBackward,
            State::Crouch,
            State::Jump,
            State::JumpForward,
            State::JumpBackward,
            State::Block,
            State::CrouchBlock,
            State::Punch,
            State::PunchMedium,
            State::PunchHeavy,
            State::ForwardPunch,
            State::BackPunch,
            State::Kick,
            State::KickMedium,
            State::KickHeavy,
            State::ForwardKick,
            State::BackKick,
            State::CrouchPunch,
            State::CrouchForwardPunch,
            State::CrouchBackPunch,
            State::CrouchKick,
            State::CrouchForwardKick,
            State::CrouchBackKick,
            State::AirPunch,
            State::AirForwardPunch,
            State::AirBackPunch,
            State::AirKick,
            State::AirForwardKick,
            State::AirBackKick,
            State::Throw,
            State::Special(0),
            State::Special(1),
            State::Special(2),
            State::Super,
            State::Hit,
            State::KnockedOut,
            State::Victory,
        ];
        for character in Character::ALL {
            let (width, height) = sheet_dimensions(character);
            for state in states {
                for component in animation_frames(character, state) {
                    let frame = animation_data::component_frame(character, *component);
                    assert!(frame.x >= 0.0 && frame.y >= 0.0);
                    assert!(
                        frame.x + frame.w <= width,
                        "{} {:?} component {} exceeds width",
                        character.name(),
                        state,
                        component
                    );
                    assert!(
                        frame.y + frame.h <= height,
                        "{} {:?} component {} exceeds height",
                        character.name(),
                        state,
                        component
                    );
                }
            }
        }
    }

    #[test]
    fn crouching_renders_shorter_than_idle() {
        for character in Character::ALL {
            let mut fighter = Fighter::new(character, 300.0, 1.0);
            let standing_height = hurtbox(&fighter).h;
            fighter.state = State::Crouch;
            let crouching_height = hurtbox(&fighter).h;
            assert!(
                crouching_height <= standing_height * 0.75,
                "{} crouch should have a substantially shorter hurtbox",
                character.name()
            );
        }
    }

    #[test]
    fn super_requires_a_full_meter() {
        let mut fighter = Fighter::new(Character::Rose, 300.0, 1.0);
        assert!(fighter.meter < MAX_METER);
        fighter.meter = MAX_METER;
        fighter.start_action(State::Super);
        assert_eq!(fighter.state, State::Super);
    }

    #[test]
    fn damage_never_makes_health_negative() {
        let mut fighter = Fighter::new(Character::Cammy, 300.0, 1.0);
        fighter.receive_hit(500, 10, 100.0, 1.0);
        assert_eq!(fighter.health, 0);
        assert_eq!(fighter.state, State::KnockedOut);
    }

    #[test]
    fn standing_combo_stages_are_separate_attacks() {
        assert_eq!(combo_state(ComboKind::Punch, 0), State::Punch);
        assert_eq!(combo_state(ComboKind::Punch, 1), State::PunchMedium);
        assert_eq!(combo_state(ComboKind::Punch, 2), State::PunchHeavy);
        assert_eq!(combo_state(ComboKind::Kick, 0), State::Kick);
        assert_eq!(combo_state(ComboKind::Kick, 1), State::KickMedium);
        assert_eq!(combo_state(ComboKind::Kick, 2), State::KickHeavy);

        assert_eq!(animation_frames(Character::Cammy, State::Punch), &[86, 87]);
        assert_eq!(
            animation_frames(Character::Cammy, State::PunchMedium),
            &[88, 89]
        );

        let mut fighter = Fighter::new(Character::Cammy, 300.0, 1.0);
        fighter.start_combo_stage(ComboKind::Punch, 0);
        fighter.finish_combo_stage();
        assert_eq!(fighter.combo_timer, COMBO_LINK_FRAMES);
        assert_eq!(fighter.next_combo_step(ComboKind::Punch), 1);
        assert_eq!(fighter.next_combo_step(ComboKind::Kick), 0);
    }

    #[test]
    fn rose_super_uses_its_left_facing_source_orientation() {
        assert!(sprite_flip_x(Character::Rose, State::Super, 1.0));
        assert!(!sprite_flip_x(Character::Rose, State::Super, -1.0));
        assert!(!sprite_flip_x(Character::Rose, State::Idle, 1.0));
    }

    #[test]
    fn super_meter_carries_between_rounds_but_not_into_a_new_match() {
        let mut game = Game::new();
        game.fighters[0].meter = MAX_METER;
        game.fighters[1].meter = 43;
        game.reset_round(true);
        assert_eq!(game.fighters[0].meter, MAX_METER);
        assert_eq!(game.fighters[1].meter, 43);

        game.reset_round(false);
        assert_eq!(game.fighters[0].meter, 0);
        assert_eq!(game.fighters[1].meter, 0);
    }

    #[test]
    fn every_character_has_a_victory_animation() {
        for character in Character::ALL {
            assert!(animation_frames(character, State::Victory).len() > 1);
        }
    }

    #[test]
    fn combat_impact_assets_are_valid_ogg_files() {
        assert!(include_bytes!("../assets/audio/punch.ogg").starts_with(b"OggS"));
        assert!(include_bytes!("../assets/audio/kick.ogg").starts_with(b"OggS"));
        assert!(State::Kick.is_kick());
        assert!(!State::Punch.is_kick());
    }

    #[test]
    fn pushbox_overlap_math_detects_contact() {
        assert!(rects_overlap(
            Rect::new(0.0, 0.0, 10.0, 10.0),
            Rect::new(9.0, 2.0, 4.0, 4.0)
        ));
        assert!(!rects_overlap(
            Rect::new(0.0, 0.0, 10.0, 10.0),
            Rect::new(10.0, 0.0, 3.0, 3.0)
        ));
    }

    #[test]
    fn a_high_airborne_fighter_can_cross_over_an_opponent() {
        let mut game = Game::new();
        game.fighters[0].x = 500.0;
        game.fighters[1].x = 510.0;
        game.fighters[0].y = FLOOR_Y - 80.0;
        game.fighters[0].grounded = false;
        game.fighters[0].state = State::JumpForward;
        let positions = [game.fighters[0].x, game.fighters[1].x];

        game.resolve_pushboxes();

        assert_eq!(positions, [game.fighters[0].x, game.fighters[1].x]);
        game.fighters[0].grounded = true;
        game.fighters[0].y = FLOOR_Y;
        game.fighters[0].state = State::Idle;
        game.resolve_pushboxes();
        assert!((game.fighters[1].x - game.fighters[0].x).abs() >= PUSH_HALF * 2.0);
    }

    #[test]
    fn returning_to_character_select_resets_match_flow() {
        let mut game = Game::new();
        game.phase = Phase::Fight;
        game.paused = true;
        game.locked = [true, true];
        game.wins = [1, 1];
        game.round = 3;

        game.return_to_character_select();

        assert!(matches!(game.phase, Phase::Select));
        assert!(!game.paused);
        assert_eq!(game.locked, [false, false]);
        assert_eq!(game.wins, [0, 0]);
        assert_eq!(game.round, 1);
    }

    #[test]
    fn a_preemptive_jump_clears_even_the_super_projectile_hitbox() {
        let apex_y = FLOOR_Y - JUMP_SPEED * JUMP_SPEED / (2.0 * GRAVITY);
        let mut fighter = Fighter::new(Character::FeiLong, 300.0, 1.0);
        fighter.y = apex_y;
        fighter.grounded = false;
        fighter.state = State::Jump;
        let projectile = Projectile {
            owner: 0,
            x: 300.0,
            y: FLOOR_Y - 103.0,
            vx: 0.0,
            radius: 34.0,
            hit_radius: 22.0,
            damage: 26,
            color: WHITE,
            super_move: true,
        };

        assert!(!rects_overlap(
            hurtbox(&fighter),
            projectile_hitbox(&projectile)
        ));
    }
}
