use askama::Template;

#[derive(Template, Clone)]
#[template(path = "bancho_get.html")]
pub struct BanchoGet {
    pub server_name: String,
    pub server_front: String,
}
