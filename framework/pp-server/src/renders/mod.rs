use askama::Template;

#[derive(Template, Clone)]
#[template(path = "main_page.html")]
pub struct MainPage {}

impl MainPage {
    pub fn new() -> Self {
        MainPage {}
    }
}
