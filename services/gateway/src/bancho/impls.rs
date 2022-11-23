pub mod client {
    use axum::{
        extract::Path,
        response::{IntoResponse, Response},
    };

    pub async fn bancho_post() -> Response {
        unimplemented!()
    }

    pub async fn download_beatmapset(
        Path(beatmapset_id): Path<i32>,
    ) -> Response {
        format!("got beatmapset_id: {}", beatmapset_id).into_response()
    }

    pub async fn client_register() -> Response {
        unimplemented!()
    }

    pub async fn ask_peppy() -> Response {
        unimplemented!()
    }

    pub async fn difficulty_rating() -> Response {
        unimplemented!()
    }

    pub mod web {
        use axum::response::Response;

        pub async fn osu_error() -> Response {
            unimplemented!()
        }

        pub async fn osu_screenshot() -> Response {
            unimplemented!()
        }

        pub async fn osu_getfriends() -> Response {
            unimplemented!()
        }

        pub async fn osu_getbeatmapinfo() -> Response {
            unimplemented!()
        }

        pub async fn osu_getfavourites() -> Response {
            unimplemented!()
        }

        pub async fn osu_addfavourite() -> Response {
            unimplemented!()
        }

        pub async fn lastfm() -> Response {
            unimplemented!()
        }

        pub async fn osu_search() -> Response {
            unimplemented!()
        }

        pub async fn osu_search_set() -> Response {
            unimplemented!()
        }

        pub async fn osu_submit_modular_selector() -> Response {
            unimplemented!()
        }

        pub async fn osu_getreplay() -> Response {
            unimplemented!()
        }

        pub async fn osu_rate() -> Response {
            unimplemented!()
        }

        pub async fn osu_osz2_getscores() -> Response {
            unimplemented!()
        }

        pub async fn osu_comment() -> Response {
            unimplemented!()
        }

        pub async fn osu_markasread() -> Response {
            unimplemented!()
        }

        pub async fn osu_getseasonal() -> Response {
            unimplemented!()
        }

        pub async fn bancho_connect() -> Response {
            unimplemented!()
        }

        pub async fn check_updates() -> Response {
            unimplemented!()
        }

        pub async fn update_beatmap() -> Response {
            unimplemented!()
        }

        pub async fn get_screenshot() -> Response {
            unimplemented!()
        }
    }
}
