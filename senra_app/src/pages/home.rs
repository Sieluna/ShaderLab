use iced::widget::{button, column, container, horizontal_space, mouse_area, row, scrollable, text};
use iced::{Alignment, Element, Length, Task};

#[derive(Debug, Clone)]
pub enum Message {
    LoadNotebooks,
    SelectCategory(String),
    OpenNotebook(u32),
}

#[derive(Debug, Clone)]
struct NotebookCard {
    id: u32,
    title: String,
    author: String,
    likes: u32,
    preview: String,
    category: String,
}

impl NotebookCard {
    fn view(&self) -> Element<Message> {
        let card =container(
            column![
                // Preview image placeholder
                container(
                    row![]
                        .width(Length::Fixed(200.0))
                        .height(Length::Fixed(120.0))
                ),
                text(&self.title)
                    .size(16)
                    .width(Length::Fixed(200.0)),
                row![
                    text(&self.author)
                        .size(12),
                    horizontal_space(),
                    text(format!("❤️ {}", self.likes))
                        .size(12),
                ]
                .width(Length::Fixed(200.0))
            ]
            .spacing(8)
        )
        .padding(8);

        mouse_area(card)
            .on_press(Message::OpenNotebook(self.id))
            .into()
    }
}

pub struct HomePage {
    selected_category: String,
    categories: Vec<String>,
    notebooks: Vec<NotebookCard>,
}

impl HomePage {
    pub fn new() -> (Self, Task<Message>) {
        let notebooks = vec![
            NotebookCard {
                id: 1,
                title: "Amazing Shader Effects".to_string(),
                author: "JohnDoe".to_string(),
                likes: 1234,
                preview: "".to_string(),
                category: "Shader".to_string(),
            },
            NotebookCard {
                id: 2,
                title: "Advanced Lighting Techniques".to_string(),
                author: "ShaderMaster".to_string(),
                likes: 856,
                preview: "".to_string(),
                category: "Shader".to_string(),
            },
            NotebookCard {
                id: 3,
                title: "Getting Started with Shaders".to_string(),
                author: "BeginnerGuide".to_string(),
                likes: 432,
                preview: "".to_string(),
                category: "Shader".to_string(),
            },
            NotebookCard {
                id: 4,
                title: "Documentation Template".to_string(),
                author: "DocWriter".to_string(),
                likes: 321,
                preview: "".to_string(),
                category: "Markdown".to_string(),
            },
            NotebookCard {
                id: 5,
                title: "Project Notes".to_string(),
                author: "NoteTaker".to_string(),
                likes: 156,
                preview: "".to_string(),
                category: "Markdown".to_string(),
            },
        ];

        (
            Self {
                selected_category: "Featured".to_string(),
                categories: vec![
                    "Featured".to_string(),
                    "Popular".to_string(),
                    "Latest".to_string(),
                    "Shader".to_string(),
                    "Markdown".to_string(),
                ],
                notebooks,
            },
            Task::done(Message::LoadNotebooks),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadNotebooks => Task::none(),
            Message::SelectCategory(category) => {
                self.selected_category = category;
                Task::none()
            }
            Message::OpenNotebook(id) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        // Header
        let header = container(
            text("ShaderLab")
                .size(32)
        )
        .padding(20)
        .align_x(Alignment::Center);

        // Category bar
        let category_bar = row![]
            .spacing(20)
            .padding([10, 20])
            .align_y(Alignment::Center);

        let category_bar = self.categories.iter().fold(category_bar, |row, category| {
            row.push(
                button(text(category))
                    .padding([12, 24])
                    .style(if category == &self.selected_category {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .on_press(Message::SelectCategory(category.clone())),
            )
        });

        // Carousel
        let carousel = container(
            row![]
                .width(Length::Fill)
                .height(Length::Fixed(300.0))
                .align_y(Alignment::Center),
        )
        .padding(20);

        // Featured section
        let featured_title = container(
            row![
                text("Featured Content")
                    .size(24),
                horizontal_space(),
                button("View More")
                    .padding([8, 16])
                    .style(button::secondary)
            ]
            .align_y(Alignment::Center)
        )
        .padding([0, 20]);

        // Featured content grid
        let featured_grid = container(
            row![]
                .spacing(20)
                .padding(20)
        )
        .width(Length::Fill);

        // Category section
        let category_title = container(
            row![
                text(format!("{} Content", self.selected_category))
                    .size(24),
                horizontal_space(),
                button("View More")
                    .padding([8, 16])
                    .style(button::secondary)
            ]
            .align_y(Alignment::Center)
        )
        .padding([0, 20]);

        // Category content grid
        let category_notebooks: Vec<_> = self.notebooks
            .iter()
            .filter(|n| n.category == self.selected_category)
            .collect();

        let category_grid = row![]
            .spacing(20)
            .padding(20)
            .width(Length::Fill);

        let category_grid = category_notebooks.iter().fold(category_grid, |row, notebook| {
            row.push(notebook.view())
        });

        // Main content
        let content = column![
            header,
            category_bar,
            carousel,
            featured_title,
            featured_grid,
            category_title,
            category_grid
        ]
        .spacing(30)
        .padding(20);

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
