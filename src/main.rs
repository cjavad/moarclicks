#[macro_use]
extern crate clap;
use clap::App;
use crossbeam_channel::{unbounded, Receiver};
use enigo::{Enigo, MouseButton, MouseControllable};
use inputbot::{
    handle_input_events,
    MouseButton::{LeftButton, RightButton},
};
use rand::prelude::*;
use std::{
    thread::{sleep, spawn},
    time::{Duration, SystemTime},
};

fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub enum Click {
    LeftClick(MouseButton, u128),
    RightClick(MouseButton, u128),
}

pub enum Action {
    Click(MouseButton),
    Delay(Duration),
}

pub struct ClickHistory {
    pub last_clicked: u128,
    pub queue_renew: bool,
    pub skip_clicks: i32,
}

impl ClickHistory {
    pub fn new() -> Self {
        ClickHistory {
            last_clicked: 0,
            skip_clicks: 0,
            queue_renew: true
        }
    }
}

pub struct Clicker {
    pub enigo: Enigo,
    pub click_queue: Vec<Action>,
    pub min_cps: i32,
    pub extra_clicks: i32,
    pub min_delay_ns: u32,
    pub max_delay_ns: u32,
    pub weighted_rand_delay: f32,
    pub left_clicks: ClickHistory,
    pub right_clicks: ClickHistory,
    pub receiver: Receiver<Click>,
}

impl Clicker {
    pub fn new(
        min_cps: i32,
        extra_clicks: i32,
        min_delay_ns: u32,
        max_delay_ns: u32,
        weighted_rand_delay: f32,
    ) -> Self {
        let (s1, r1) = unbounded();
        let s2 = s1.clone();
        let s3 = s2.clone();

        RightButton.bind(move || {
            s2.send(Click::RightClick(MouseButton::Right, get_time()))
                .unwrap();
        });

        LeftButton.bind(move || {
            s3.send(Click::LeftClick(MouseButton::Left, get_time()))
                .unwrap();
        });

        Clicker {
            enigo: Enigo::new(),
            click_queue: Vec::new(),
            min_cps,
            extra_clicks,
            min_delay_ns,
            max_delay_ns,
            weighted_rand_delay,
            left_clicks: ClickHistory::new(),
            right_clicks: ClickHistory::new(),
            receiver: r1,
        }
    }

    pub fn add_queue(&mut self, amount: i32, button: MouseButton) {
        let mut rng = thread_rng();

        for _ in 0..amount {
            self.click_queue.push(Action::Click(button));
            self.click_queue.push(Action::Delay(Duration::new(
                0,
                match rng.gen_range(0.0..1.0) {
                    n if n < self.weighted_rand_delay => self.min_delay_ns,
                    _ => rng.gen_range(self.min_delay_ns..self.max_delay_ns),
                },
            )));
        }
    }

    pub fn enhance_click(&mut self, button: MouseButton) {
        self.add_queue(self.extra_clicks, button)
    }

    pub fn execute_queue(&mut self) {
        for action in &self.click_queue {
            match action {
                Action::Click(button) => match button {
                    MouseButton::Left => {
                        if self.left_clicks.queue_renew {
                            self.enigo.mouse_click(MouseButton::Left)
                        }
                    },
                    MouseButton::Right => {
                        if self.right_clicks.queue_renew {
                            self.enigo.mouse_click(MouseButton::Right)
                        }
                    },
                    _ => {}
                },
                Action::Delay(duration) => sleep(duration.clone()),
            }
        }
    }

    pub fn next_tick(&mut self) {
        match self.receiver.recv().unwrap() {
            Click::LeftClick(button, time) => {
                self.left_clicks.queue_renew = false;
                self.click_queue = Vec::new();

                if self.left_clicks.skip_clicks > 0 {
                    self.left_clicks.skip_clicks -= 1;
                    return;
                }

                if time - self.left_clicks.last_clicked < (1000 / self.min_cps) as u128 {
                    self.left_clicks.queue_renew = true;
                    self.left_clicks.skip_clicks += self.extra_clicks;
                    self.enhance_click(button);
                } else {
                    self.click_queue = Vec::new();
                }
                self.left_clicks.last_clicked = time;
            }
            Click::RightClick(button, time) => {
                self.right_clicks.queue_renew = false;
                self.click_queue = Vec::new();

                if self.right_clicks.skip_clicks > 0 {
                    self.right_clicks.skip_clicks -= 1;
                    return;
                }

                if time - self.right_clicks.last_clicked < (1000 / self.min_cps) as u128 {
                    self.right_clicks.queue_renew = true;

                    self.right_clicks.skip_clicks += self.extra_clicks;
                    self.enhance_click(button);
                } else {
                    self.click_queue = Vec::new();
                }

                self.right_clicks.last_clicked = time;
            }
        }
    }
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let min_cps = match matches.value_of("min_cps") {
        None => panic!(),
        Some(s) => match s.parse::<i32>() {
            Ok(n) => n,
            Err(_) => panic!(),
        },
    };

    let extra_clicks = match matches.value_of("extra_clicks") {
        None => panic!(),
        Some(s) => match s.parse::<i32>() {
            Ok(n) => n,
            Err(_) => panic!(),
        },
    };

    let min_delay_ms = match matches.value_of("min_delay_ms") {
        None => panic!(),
        Some(s) => match s.parse::<u32>() {
            Ok(n) => n,
            Err(_) => panic!(),
        },
    };

    let max_delay_ms = match matches.value_of("max_delay_ms") {
        None => panic!(),
        Some(s) => match s.parse::<u32>() {
            Ok(n) => n,
            Err(_) => panic!(),
        },
    };

    let weighted_rand_delay = match matches.value_of("weighted_rand_delay") {
        None => panic!(),
        Some(s) => match s.parse::<f32>() {
            Ok(n) => n,
            Err(_) => panic!(),
        },
    };

    let mut clicker = Clicker::new(
        min_cps,
        extra_clicks,
        min_delay_ms * u32::pow(10, 6),
        max_delay_ms * u32::pow(10, 6),
        weighted_rand_delay,
    );

    spawn(|| {
        handle_input_events();
    });

    loop {
        clicker.next_tick();
        clicker.execute_queue();
    }
}
