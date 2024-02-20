use anyhow::Result as AnyResult;
use actix_web::{
    get, http::header::{ContentDisposition, DispositionType}, web, App, HttpRequest, HttpResponse, HttpServer, Responder
};
use index::IndexTmpl;  
use std::path::PathBuf;
use actix_files as afs;
use sailfish::TemplateOnce;
use listenfd::ListenFd;

mod index;

#[get("/{filename:.*}")]
async fn main_index(req: HttpRequest) -> AnyResult<afs::NamedFile, actix_web::Error> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let path_clone = path.clone();
    let mut path_str = path_clone.to_str().unwrap().to_string();
    for pth in path.components().enumerate() {
        let k = pth.1;
        let p = k.as_os_str().to_str().unwrap();

        if p == "dist" || p == "plugins" {
            break;
        }

        path_str = path_str.replace(format!("{}/", p).as_str(), "");
    }
    let path = format!("assets/{}", path_str);
    let file = afs::NamedFile::open(path);
    match file {
        Ok(res) => Ok(res.use_last_modified(true).set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        })),
        Err(e) => {
            Err(actix_web::Error::from(e))
        }
    }
}

#[actix_web::main]
async fn main() -> AnyResult<()> {
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
        .route("/", web::get().to(req_index))
        .service(main_index)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(format!("{}:{}", "127.0.0.1", 1414))?,
    };

    server.run().await?;

    Ok(())
}


async fn req_index() -> impl Responder {
    let tmpl = IndexTmpl {};
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(tmpl.render_once().unwrap())
}