use iced::advanced::graphics::layer;
use iced::advanced::svg::Handle;
use iced::alignment::Horizontal::Left;
use iced::border::width;
use iced::futures::io::Lines;
use iced::widget::{Button, Checkbox, Row, button, column, container, image, row, stack, text};
use iced::window::Settings;
use iced::{Application, Background, Length, Shadow, Subscription, run};
use iced::{Color, Element, alignment};
use iced::{Fill, window};
use regex::Regex;
use std::time::Duration;
use std::time::Instant;
use std::vec;

use crate::lib::Timer;
pub mod lib;
pub fn main() -> iced::Result {
    // iced::application(App::new, App::update, App::view).run()
    iced::application(Karaoke::new, Karaoke::update, Karaoke::view)
        .subscription(Karaoke::subscription)
        .run()
}

#[derive(Debug, Clone)]
struct Word {
    start: u64, // ms
    end: u64,
    ch: char,
}

struct Karaoke {
    lrc: String,
    timer: Timer,
    pos: u64,
    is_playing: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
    TogglePlay,
    Reset,
}

impl Karaoke {
    fn new() -> Self {
        Self {
            lrc: include_str!("lrc.txt").to_string(),
            timer: Timer::new(),
            pos: 0,
            is_playing: false,
        }
    }

    fn title(&self) -> String {
        String::from("Iced 逐字歌词")
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::Tick => {
                self.pos = self.timer.elapsed().as_millis() as u64;
                // println!(
                //     "pos:{}\ntimer:{}",
                //     self.pos,
                //     self.timer.elapsed().as_millis()
                // )
            }
            Message::TogglePlay => {
                self.is_playing = !self.is_playing;
                if self.timer.is_paused() {
                    self.timer.resume();
                } else {
                    self.timer.pause();
                }
            }
            Message::Reset => {
                self.timer.reset();
            }
        }
    }
    fn view(&self) -> Element<Message> {
        let art: iced::widget::Image<image::Handle> = image("danni.jpg").into();
        let now = self.pos;
        let lrc_lines = self.lrc.lines();

        let all_line = column(
            lrc_lines
                .map(|line| {
                    let mut line_row = iced::widget::row![];
                    let words = parse_line(line);
                    let line_start = words.first().map(|w| w.start).unwrap_or(u64::MAX);
                    let line_end = words.last().map(|w| w.end).unwrap_or(u64::MIN);
                    let is_past_line = line_end < now;
                    let is_current_line = !is_past_line && line_start <= now;
                    for (w) in words.iter() {
                        let (size, color) = if is_current_line {
                            // 当前行：已过字符高亮，未过字符默认
                            let is_passed = now >= w.start;
                            if is_passed {
                                (28, Color::from_rgb(1.0, 0.3, 0.5)) // 高亮样式
                            } else {
                                (22, Color::WHITE) // 未高亮样式
                            }
                        } else {
                            // 已过行/未到行：全部用未高亮样式
                            (22, Color::WHITE)
                        };

                        line_row = line_row.push(
                            text(w.ch.to_string()).color(color).size(size), // .horizontal_alignment(alignment::Horizontal::Center),
                        );
                    }
                    line_row.into()
                })
                .collect::<Vec<_>>(),
        );

        let play_button: Element<Message> = button("TogglePlay")
            .on_press(Message::TogglePlay)
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(40.0))
            .into();
        let now_s = now / 1000;
        let hours = now_s / 3600;
        let remaining_secs = now_s % 3600;
        let minutes = remaining_secs / 60;
        let seconds = remaining_secs % 60;
        let now_text: Element<Message> = text(format!(
            "时分秒格式：{:02}:{:02}:{:02}",
            hours, minutes, seconds
        ))
        .width(Length::Fixed(120.0))
        .height(Length::Fixed(40.0))
        .size(20)
        .into();

        let reset: Element<Message> = button("Reset")
            .on_press(Message::Reset)
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(40.0))
            .into();

        let main_layout = row![]
            .push(play_button)
            .push(reset)
            .push(now_text)
            .push(
                // 歌词容器填充右侧剩余空间
                container(all_line)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(20);
        let bg = container(art).width(Length::Fill).height(Length::Fill);
        let main_layout = container(main_layout)
            .width(Length::Fill)
            .height(Length::Fill);

        let overlay_stack = stack![bg, main_layout];
        let root_container: Element<Message> = container(overlay_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        root_container
    }

    // 用 Iced 的「订阅」每帧触发一次 Tick，实现 60 FPS 刷新
    fn subscription(&self) -> iced::Subscription<Message> {
        if self.is_playing {
            iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

fn parse_lines(content: &str) -> Vec<Word> {
    let words = content.lines().flat_map(|line| parse_line(line)).collect();
    words
}

fn parse_line(line: &str) -> Vec<Word> {
    let time_re = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{3})\]").unwrap();
    let caps: Vec<_> = time_re.captures_iter(line).collect();
    let mut words = Vec::new();

    for (i, cap) in caps.iter().enumerate() {
        let mm = cap[1].parse::<u64>().unwrap();
        let ss = cap[2].parse::<u64>().unwrap();
        let zzz = cap[3].parse::<u64>().unwrap();
        let start = mm * 60_000 + ss * 1_000 + zzz;

        // 两个时间戳之间的文字就是“字”
        let end_of_time = cap.get(0).unwrap().end();
        let next_start = caps
            .get(i + 1)
            .map(|c| c.get(0).unwrap().start())
            .unwrap_or(line.len());
        let txt = &line[end_of_time..next_start];
        // 去掉空格，支持多字符英文如 "JJ" "Lin"
        for ch in txt.chars().filter(|c| !c.is_whitespace()) {
            words.push(Word { start, end: 0, ch });
        }
    }
    // 填 end：每个字的 end 用下一个字的 start
    if !words.is_empty() {
        for i in 0..words.len() - 1 {
            words[i].end = words[i + 1].start;
        }
    }
    if let Some(last) = words.last_mut() {
        last.end = last.start + 500; // 最后一个字多留 500 ms
    }
    words
}
