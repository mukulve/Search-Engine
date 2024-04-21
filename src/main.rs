mod database;
//mod dictionary;
use anyhow::{Ok, Result};
use database::Database;
//use dictionary::Dictionary;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header;
use std::{
    collections::HashMap,
    env::{self, args},
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
};

use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};

//create global variables
lazy_static! {
    // static ref DICTIONARY: Mutex<Dictionary> = Mutex::new(Dictionary::new());
    static ref DATABASE: Mutex<Database> = Mutex::new(Database::new());
    static ref WHITESPACEREGEX: Regex = Regex::new(r"[^a-zA-Z]+").unwrap();
    static ref REQWESTCLIENT: Client = Client::new();
    static ref HTMLINNERREGEX: Regex = Regex::new(r"<[^>]*>").unwrap();
}

#[derive(Deserialize, Debug)]
struct SearchGetRequest {
    words: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        if args[1] == "--help" {
            println!("To parse file : {} --parse <file>", args[0]);
            println!("To start the server : {}", args[0]);
            return Ok(());
        }
    }

    if args.len() == 3 {
        if args[1] == "--parse" {
            println!("Parsing the file {}", &args[2]);
            read_csv(&args[2])?;
        }
    }

    // initialize tracing
    tracing_subscriber::fmt::init();

    //setup cors
    let cors = CorsLayer::new()
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_origin(Any);

    // build our application with a route
    let app = Router::new().route("/", get(root)).layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root(query: Query<SearchGetRequest>) -> String {
    //search the database for the words
    let search_result = DATABASE
        .lock()
        .unwrap()
        .search(query.words.as_str())
        .unwrap_or_default();

    //return the search result as json string
    format!("{:?}", search_result)
}

fn read_csv(path: &str) -> Result<()> {
    //read file line by line parallelly
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().par_bridge().for_each(|line| {
        scrape_url(format!("https://{}", line.unwrap())).unwrap();
    });
    Ok(())
}

fn scrape_url(url: String) -> Result<()> {
    let response = REQWESTCLIENT
        .get(&url)
        .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36 Edg/123.0.0.0")
        .send()?;

    let content_type = response.headers().get("content-type");

    //check if the response is not html
    if content_type.unwrap().to_str()?.find("text/html").is_none() {
        return Ok(());
    }

    //get body of the response
    let body = response.text()?;

    //get all the text inside the html tags
    let inner_text = HTMLINNERREGEX.split(&body).collect::<Vec<&str>>().join(" ");

    //split the body into words
    let words = inner_text.split_whitespace().collect::<Vec<&str>>();

    //create a dictionary for each word with its count
    let word_count = Arc::new(Mutex::new(HashMap::new()));

    words.par_iter().for_each(|word| {
        let mut word_count_mutex = word_count.lock().unwrap();
        let count = word_count_mutex.entry(word.to_lowercase()).or_insert(0);
        *count += 1;
    });

    //insert the website and its words into the database
    DATABASE.lock().unwrap().insert_website(&url)?;
    let website_id = DATABASE.lock().unwrap().get_website_id(&url)?;
    let word_count_mutex = word_count.lock().unwrap();
    word_count_mutex
        .par_iter()
        .filter(|(_, &c)| c > 2)
        .filter(|(word, _)| word.len() > 2)
        .for_each(|(word, count)| {
            DATABASE
                .lock()
                .unwrap()
                .insert_word(word, website_id, *count as i32)
                .unwrap();
        });

    Ok(())
}
