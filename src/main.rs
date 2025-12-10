use iced::alignment::Horizontal::Left;
use iced::border::width;
use iced::widget::{Checkbox, Row, button, column, container, row, text};
use iced::{Application, Length, run};
use iced::{Color, Element, Settings, alignment};
use regex::Regex;
use std::time::Duration;
use std::time::Instant;
use std::vec;
pub fn main() -> iced::Result {
    // iced::application(App::new, App::update, App::view).run()
    iced::application(Karaoke::new, Karaoke::update, Karaoke::view).run()
}

#[derive(Debug, Clone)]
struct Word {
    start: u64, // ms
    end: u64,
    ch: char,
}

struct Karaoke {
    words: Vec<Word>, // 第一行字事件流
    start: Instant,   // 伪播放起点
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

// struct App {
//     is_prev_playing: bool,
//     mode: ViewMode,
//     // config: Vec<Config>,
//     current_pos: usize,
//     // current_source: audio::TheSource,
//     slider_value: f32,
//     time: Duration,
//     tick_secs: f32,
// }

// #[derive(Debug, Clone)]
// enum Message {}
impl Karaoke {
    fn new() -> Self {
        let lrc = include_str!("lrc.txt"); // 把整段 LRC 放同目录 lrc.txt
        let words = parse_lines(&lrc);
        Self {
            words,
            start: Instant::now(),
        }
    }

    fn title(&self) -> String {
        String::from("Iced 逐字歌词")
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::Tick => {} // 什么都不用做，视图会重新取当前时间
        }
    }

    fn view(&self) -> Element<Message> {
        let now = self.start.elapsed().as_millis() as u64;
        let mut line = iced::widget::row![];
        for (w) in self.words.iter() {
            let active = now >= w.start && now < w.end;
            let size = if active { 28 } else { 22 };
            let color = if active {
                Color::from_rgb(1.0, 0.3, 0.5)
            } else {
                Color::WHITE
            };
            line = line.push(
                text(w.ch.to_string()).size(size), // .style(color)
                                                   // .horizontal_alignment(alignment::Horizontal::Center),
            );
        }

        container(line)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    // 用 Iced 的「订阅」每帧触发一次 Tick，实现 60 FPS 刷新
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
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
#[cfg(test)]

mod tests {

    use super::*;

    #[test]

    fn test_parse_first_line_basic() {
        let content = "[01:23.456]Hello[01:24.000]World";

        let words = parse_lines(content);

        let expected_starts: Vec<u64> = vec![83456; 5].into_iter().chain(vec![84000; 5]).collect();

        let expected_chars: Vec<char> = "HelloWorld".chars().collect();

        for (i, word) in words.iter().enumerate() {
            println!("{}", word.ch);
            assert_eq!(word.start, expected_starts[i]);

            assert_eq!(word.ch, expected_chars[i]);
        }

        // 检查 end 字段（除最后一个外，end = 下一个 start）

        for i in 0..9 {
            assert_eq!(words[i].end, words[i + 1].start);
        }
    }

    // #[test]
    //
    // fn test_parse_first_line_no_match() {
    //     let content = "This line has no timestamps.";
    //
    //     let words = parse_first_line(content);
    //
    //     assert_eq!(words.len(), 0);
    // }
    //
    // #[test]
    //
    // fn test_parse_first_line_empty_input() {
    //     let content = "";
    //
    //     let words = parse_first_line(content);
    //
    //     assert_eq!(words.len(), 0);
    // }
    //
    // #[test]
    //
    // fn test_parse_first_line_whitespace_only() {
    //     let content = "[00:00.000]   ";
    //
    //     let words = parse_first_line(content);
    //
    //     assert_eq!(words.len(), 0); // 空格被过滤
    // }
    //
    // #[test]
    //
    // fn test_parse_first_line_single_char() {
    //     let content = "[00:00.000]A";
    //
    //     let words = parse_first_line(content);
    //
    //     assert_eq!(words.len(), 1);
    //
    //     assert_eq!(words[0].ch, 'A');
    //
    //     assert_eq!(words[0].start, 0);
    //
    //     assert_eq!(words[0].end, 500); // 最后一个字 end = start + 500
    // }
}
// impl App {
//     fn new() {}
//     fn update(&mut self, message: Message) {
//         match message {
//             _ => print!("_ matched"),
//         }
//     }
//     fn view(&self) {}
// }
//
// struct Addon {
//     addon_name: String,
//     addon_id: i64,
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum ViewMode {
//     Desktop,
// }
