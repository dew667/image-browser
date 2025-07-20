use std::path::{self, PathBuf};

use iced::{alignment::Horizontal, color, overlay::menu::Style, widget::{button::{self, Button, Status}, column, container, image, row, scrollable, text, Column, Row}, Element, Length, Theme};
use rfd::FileDialog;

mod button_style;
use button_style::default;
use button_style::primary;

#[derive(Default)]
struct State {
    current_path: PathBuf,
    current_image: Option<PathBuf>,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeDirectory(PathBuf),
    SelectImage,
    NoOp,
}

impl State {

    fn update(&mut self, message: Message) {
        match message {
            Message::SelectImage => {
                let path = FileDialog::new()
                    .add_filter("image", &["png", "jpg", "jpeg", "gif"])
                    .set_directory("/")
                    .pick_file();
                self.current_image = path;
            },
            Message::ChangeDirectory(path) => {
                if path.is_dir() {
                    self.current_path = path;
                } else {
                    eprintln!("Error: {} is not a directory", path.display());
                }
            }, 
            Message::NoOp => {
                // No operation, can be used for future actions
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let top_bar = row![
            row![
                text("📷").size(20), // Camera icon as a placeholder logo
                text("Image Browser")
                    .size(20)
                    .align_x(Horizontal::Left)
            ]
            .spacing(5)
            .width(Length::FillPortion(2)),

            row![
                Button::new(text("Open"))
                    .on_press(Message::SelectImage)
                    .style(|theme, status| default(theme, status)),
                Button::new(text("Next"))
                    .on_press(Message::NoOp)
                    .style(|theme, status| default(theme, status)),
                Button::new(text("Previous"))
                    .on_press(Message::NoOp)
                    .style(|theme, status| default(theme, status)),
                Button::new(text("Zoom"))
                    .on_press(Message::NoOp)
                    .style(|theme, status| default(theme, status)),
                Button::new(text("Share"))
                    .on_press(Message::NoOp)
                    .style(|theme, status| default(theme, status)),
                container(
                    Button::new(text("Fullscreen"))
                        .on_press(Message::NoOp)
                        .style(|theme, status| primary(theme, status)),
                )
                .align_x(Horizontal::Right)
                .width(Length::Fill),
            ]
            .spacing(10)
            .width(Length::FillPortion(3))
        ].spacing(10);

        // File Tree Sidebar (Left)
        let file_tree = container(
            scrollable(
                column![
                    text("▶ Veen").size(16),
                    column![
                        text("  ▶ Orion").size(14),
                        text("    - Poisesr").size(12),
                        text("    - Poler").size(12),
                        text("    - Felippe").size(12),
                        text("  ▶ Prorien").size(14),
                        text("    - Songon").size(12),
                        text("    - Polt").size(12),
                        text("    - Opatial").size(12),
                    ].padding([20, 0]), // Indent sub-items
                    text("▶ Frosnite").size(16),
                    column![
                        text("  ▶ Floit").size(14),
                        text("    - Folior").size(12),
                        text("    - Corgin").size(12),
                        text("    - Eleis").size(12),
                        text("  ▶ Tiobe").size(14),
                        text("    - Ropsisner").size(12),
                        text("    - Kociney").size(12),
                        text("    - Lost").size(12),
                        text("    - Doite").size(12),
                        text("    - Lote").size(12),
                        text("    - Eliet").size(12),
                    ].padding([0, 20]),
                ]
                .spacing(5)
                .width(Length::Fill)
            )
        )
        .width(Length::FillPortion(1))
        .padding(10);

        let main_image_display = container(
            image(self.current_image.clone().unwrap_or_else(|| PathBuf::from("placeholder.png")))
                .width(Length::Fill)
                .height(Length::Fill)
        );

        let main_content = row![
            file_tree,
            column![
                main_image_display,
                // thumbnail_bar,
            ]
            .width(Length::FillPortion(4)) // This column takes the remaining space
            .height(Length::Fill) // Fill remaining height
        ]
        .width(Length::Fill)
        .height(Length::Fill);


        column![
            top_bar,
            main_content,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn main() -> iced::Result {
    println!("Hello, world!");
    iced::run("Image Browser", State::update, State::view)
}
