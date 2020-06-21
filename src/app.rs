use serde::Deserialize;
use std::env;
use std::net::SocketAddr;
use std::process::Command;
use std::str;
use std::time::SystemTime;
use tokio::fs::{self, File};
use tokio::io::Result as IoResult;
use tokio::prelude::*;
use warp::{http, Filter};

const WKHTMLTOPDF_CMD: &str = "wkhtmltopdf";
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

enum FileType {
    Html,
    Pdf,
}

#[derive(Deserialize)]
struct PdfRequest {
    // Html body for pdf generation
    html: Option<String>,
    // Url for pdf generation
    url: Option<String>,
}

struct FileBuilder {
    html_file_name: String,
    pdf_file_name: String,
}

impl FileBuilder {
    fn new(secs: u64) -> Self {
        Self {
            html_file_name: String::from(format!("./{}.html", secs)),
            pdf_file_name: String::from(format!("./{}.pdf", secs)),
        }
    }

    async fn cleanup(&self) -> IoResult<()> {
        fs::remove_file(&self.pdf_file_name).await?;
        fs::remove_file(&self.html_file_name).await?;
        Ok(())
    }

    async fn generate_pdf(&self, pdf_request: PdfRequest) -> IoResult<Vec<u8>> {
        match pdf_request.html {
            Some(html_body) => {
                let contents = self.build_pdf_from_html(html_body).await?;
                self.cleanup().await?;
                Ok(contents)
            }
            None => match pdf_request.url {
                Some(url) => {
                    let contents = self.build_pdf_from_url(url).await?;
                    // self.cleanup().await?;
                    Ok(contents)
                }
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No url or html body found",
                )),
            },
        }
    }

    async fn build_pdf_from_html(&self, html_body: String) -> IoResult<Vec<u8>> {
        self.create_file(html_body, FileType::Html)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let contents = self.generate_pdf_from_html().await?;
        Ok(contents)
    }

    async fn build_pdf_from_url(&self, url: String) -> IoResult<Vec<u8>> {
        if url.contains("export") {
            let contents = self.generate_pdf_from_url(url).await?;
            return Ok(contents);
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "comp or export page url paramter not found",
            ))
        }
    }

    async fn create_file(&self, content: String, file_type: FileType) -> Result<(), Error> {
        match file_type {
            // Create html file from existing body
            FileType::Html => {
                let mut file = File::create(&self.html_file_name).await?;
                file.write_all(content.as_bytes()).await?;
                Ok(())
            }
            // Create pdf file from existing html file
            FileType::Pdf => {
                let mut file = File::create(&self.pdf_file_name).await?;
                file.write_all(content.as_bytes()).await?;
                Ok(())
            }
        }
    }

    async fn read_file(&self, file_type: FileType) -> Result<Vec<u8>, Error> {
        match file_type {
            FileType::Html => {
                let mut html_file = File::open(&self.html_file_name).await?;
                let mut contents = vec![];
                html_file.read_to_end(&mut contents).await?;
                Ok(contents)
            }
            FileType::Pdf => {
                let mut pdf_file = File::open(&self.pdf_file_name).await?;
                let mut contents = vec![];
                pdf_file.read_to_end(&mut contents).await?;
                Ok(contents)
            }
        }
    }

    async fn generate_pdf_from_url(&self, url: String) -> IoResult<Vec<u8>> {
        let res = Command::new(WKHTMLTOPDF_CMD)
            .args(&[
                "--javascript-delay",
                "40000",
                url.as_str(),
                &self.pdf_file_name,
            ])
            .status()
            .expect("wkhtmltopdf get url command failed to start");
        dbg!(res);
        let content = self
            .read_file(FileType::Pdf)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(content)
    }

    async fn generate_pdf_from_html(&self) -> IoResult<Vec<u8>> {
        let res = Command::new(WKHTMLTOPDF_CMD)
            .arg(&self.html_file_name)
            .arg(&self.pdf_file_name)
            .output()
            .expect("wkhtmltopdf post command failed to start");
        dbg!(res);
        let content = self
            .read_file(FileType::Pdf)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(content)
    }
}

async fn generate(pdf_request: PdfRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let builder = FileBuilder::new(now.as_secs());
    match builder.generate_pdf(pdf_request).await {
        Ok(contents) => Ok(warp::reply::with_status(
            contents,
            http::StatusCode::CREATED,
        )),
        Err(err) => Ok(warp::reply::with_status(
            format!("{}", err).as_bytes().to_vec(),
            http::StatusCode::BAD_REQUEST,
        )),
    }
}

pub async fn start() {
    let log = warp::log("pdf-generator::api");
    let bind_address: SocketAddr = env::var("BIND_ADDRESS")
        .expect("BIND_ADDRESS is not set")
        .parse()
        .expect("BIND_ADDRESS is invalid");

    println!("You can access the server at {}", bind_address);

    let pdf_builder_routes = warp::post()
        .and(warp::path("render"))
        .and(warp::body::json())
        .and_then(generate)
        .with(log);

    warp::serve(pdf_builder_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

#[tokio::test]
async fn test_build_html() {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let builder = FileBuilder::new(now.as_secs());
    builder
        .create_file(String::from("<p>TEST</p>"), FileType::Html)
        .await
        .unwrap();
    let content = builder.read_file(FileType::Html).await.unwrap();
    let s = match str::from_utf8(&content[..]) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    assert_eq!(s, "<p>TEST</p>");
}
