use fantoccini::{Client, Locator};
use serde::Deserialize;
use std::env;
use std::net::SocketAddr;
use std::process::Command;
use tokio::fs::File;
use tokio::io::Result as IoResult;
use tokio::prelude::*;
use uuid::Uuid;
use warp::{http, Filter};

const WKHTMLTOPDF_CMD: &str = "wkhtmltopdf";

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
    fn new(html_file_name: String, pdf_file_name: String) -> Self {
        Self {
            html_file_name,
            pdf_file_name,
        }
    }

    async fn generate_pdf(&self, pdf_request: PdfRequest) -> IoResult<Vec<u8>> {
        match pdf_request.html {
            Some(html_body) => {
                let contents = self.build_pdf_from_html(html_body).await?;
                Ok(contents)
            }
            None => match pdf_request.url {
                Some(url) => {
                    let contents = self.build_pdf_from_url(url).await?;
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
        &self.create_file(html_body, FileType::Html).await?;
        let contents = self.generate_pdf_from_html().await?;
        Ok(contents)
    }

    async fn build_pdf_from_url(&self, url: String) -> IoResult<Vec<u8>> {
        let contents = self.generate_pdf_from_url(url).await?;
        Ok(contents)
    }

    async fn create_file(&self, content: String, file_type: FileType) -> IoResult<()> {
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

    async fn read_file(&self, file_type: &FileType) -> IoResult<Vec<u8>> {
        match file_type {
            FileType::Html => {
                let mut pdf_file = File::open(&self.html_file_name).await.unwrap();
                let mut contents = vec![];
                pdf_file.read_to_end(&mut contents).await.unwrap();
                Ok(contents)
            }
            FileType::Pdf => {
                let mut pdf_file = File::open(&self.pdf_file_name).await.unwrap();
                let mut contents = vec![];
                pdf_file.read_to_end(&mut contents).await.unwrap();
                Ok(contents)
            }
        }
    }

    async fn generate_pdf_from_url(&self, url: String) -> IoResult<Vec<u8>> {
        // Create a client connected to web-driver on host:port
        let mut client = Client::new("http://localhost:4444")
            .await
            .expect("failed to connect to WebDriver");
        client.goto(&url.as_str()).await.unwrap();
        // Wait for specific condition
        client
            .wait_for_find(Locator::Css(".lineofsight__node"))
            .await
            .unwrap();
        // Upload html content
        let beneficiaries_html = client
            .find(Locator::Css(".wfp--wrapper--background--lighter"))
            .await
            .unwrap()
            .html(true)
            .await
            .unwrap();
        client.close().await.unwrap();
        self.create_file(beneficiaries_html, FileType::Html).await?;
        let content = self.generate_pdf_from_html().await?;
        Ok(content)
    }

    async fn generate_pdf_from_html(&self) -> IoResult<Vec<u8>> {
        let res = Command::new(WKHTMLTOPDF_CMD)
            .arg(&self.html_file_name)
            .arg(&self.pdf_file_name)
            .output()
            .expect("wkhtmltopdf post command failed to start");
        dbg!(res);
        let content = self.read_file(&FileType::Pdf).await?;
        Ok(content)
    }
}

async fn generate(pdf_request: PdfRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let unique_id = Uuid::new_v4();
    let builder = FileBuilder::new(
        String::from(format!("./media/{}.html", unique_id)),
        String::from(format!("./media/{}.pdf", unique_id)),
    );
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
