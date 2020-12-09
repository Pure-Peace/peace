use super::depends::*;

pub async fn handler(req: HttpRequest, body: Bytes, counter: Data<IntCounterVec>) -> HttpResponse {
    counter
        .with_label_values(&["/bancho", "get", "start"])
        .inc();
    //println!("GET Body {:?}", &body);
    //println!("REQ {:?}\n--------------", req);
    //let contents = "Hello bancho!";
    HttpResponse::Ok().body("666")
}
