use {
    askama::Template,
    ntex::web::{
        get,
        types::{Data, Query},
        HttpRequest, HttpResponse,
    },
    peace_performance::PpResult,
    serde_json::json,
    serde_json::Value::Null,
    std::time::Instant,
};

use crate::objects::{
    calculator::{self, CalcData},
    glob::Glob,
};

/// GET "/api"
#[get("")]
pub async fn index(glob: Data<Glob>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(glob.render_main_page.render().unwrap())
}

// calculate pp (used by peace)
#[get("/calc")]
pub async fn calculate_pp(req: HttpRequest, glob: Data<Glob>) -> HttpResponse {
    let get_raw = |result: &PpResult| {
        json!({
            "aim": result.raw.aim.unwrap_or(0.0),
            "spd": result.raw.spd.unwrap_or(0.0),
            "acc": result.raw.acc.unwrap_or(0.0),
            "str": result.raw.str.unwrap_or(0.0),
            "total": result.raw.total,
        })
    };
    let failed = |status, message| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(json!(
                {
                    "status": status,
                    "message": message,
                    "pp": null
                }
            ))
    };
    let start = Instant::now();

    // Parse query data
    let mut data = match Query::<CalcData>::from_query(&req.query_string()) {
        Ok(Query(q)) => q,
        Err(err) => {
            return failed(0, err.to_string().as_str());
        }
    };

    // We need any one of these
    if data.md5.is_none() && data.bid.is_none() && data.sid.is_none() {
        return failed(
            0,
            "invalid requests, we must have one of: (md5, bid, sid + filename)",
        );
    };

    let mut md5 = data.md5.clone();
    let bid = data.bid;

    // If we have md5 input
    if let Some(ref mut md5) = md5 {
        // Check md5
        if md5.len() != 32 {
            return failed(0, "invalid md5");
        }
        // Safe it
        *md5 = peace_utils::common::safe_string(md5.clone());
    };

    // get beatmap
    let beatmap =
        match calculator::get_beatmap(md5.clone(), bid, data.sid, data.file_name.clone(), &glob)
            .await
        {
            Some(b) => b,
            None => return failed(0, "cannot found beatmap"),
        };

    // Get it, calculate.
    let result = calculator::calculate_pp(&beatmap, &data).await;

    let mut value = json!({
        "status": 1,
        "message": "done",
        "mode": result.mode,
        "mods": result.mods,
        "pp": result.pp(),
        "stars": result.attributes.stars(),
        "acc_list": Null
    });

    // If need, calculate acc list..
    if data.acc_list.is_some() && data.acc_list.unwrap() > 0 {
        value["acc_list"] = calculator::calculate_acc_list(&beatmap, &data).await;
    };

    // If need, calculate no_miss
    if data.no_miss.is_some() && data.no_miss.unwrap() > 0 {
        data.miss = Some(0);
        let no_miss_result = calculator::calculate_pp(&beatmap, &data).await;
        let json = json!({
            "pp": no_miss_result.pp(),
            "raw": get_raw(&no_miss_result)
        });
        value["no_miss"] = json;
    };

    let end = start.elapsed();
    info!(
        "[calculate_pp] Beatmap {:?}({:?}) calculate done in: {:?}",
        md5, bid, end
    );

    if data.simple.is_some() && data.simple.unwrap() > 0 {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(value)
    } else {
        value["raw"] = get_raw(&result);
        HttpResponse::Ok()
            .content_type("application/json")
            .body(value)
    }
}
