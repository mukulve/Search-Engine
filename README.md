# Search Engine

A simple search engine made with Rust

<img src="https://raw.githubusercontent.com/mukulve/Search-Engine/main/photos/1.png" />
<img src="https://raw.githubusercontent.com/mukulve/Search-Engine/main/photos/2.png" />

## How does it work ?

- The program will parse websites and store all the words into a sqlite databse
- Then you can send http requests to the rust api to find matches given a string

## Tech Stack

- Rust
- petite-vue

## Usage

- To parse a file with a url on each line : `cargo run -- --parse <file>`
- To start the api server : `cargo run`

## Roadmap

- Use dictionary class to ensure only valid words are saved to db
- Update to gracefully handle errors
