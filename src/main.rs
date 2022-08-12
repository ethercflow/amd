use clap::Parser;
use scraper::{Html, Selector};
use std::env;
use std::fs::{create_dir_all, File};
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;
use std::str;
use std::{thread, time};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(parse(from_os_str))]
    urls: std::path::PathBuf,
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

async fn get_html_page(url: &str) -> String {
    let mut i = 0;
    loop {
        let body = reqwest::get(url).await;
        if body.is_err() {
            i += 1;
            eprintln!(
                "get_html_page from {} failed: {:?} retry: {} times",
                url,
                body.err().unwrap(),
                i
            );
            continue;
        }
        let body = body.unwrap().text().await;

        match body {
            Ok(body) => return body,
            Err(e) => eprintln!("get_html_page from {} get text failed: {:?}", url, e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Ok(lines) = read_lines(args.urls) {
        for line in lines {
            if let Ok(url) = line {
                let body = get_html_page(&url).await;
                let doc = Html::parse_document(&body);
                let selector = Selector::parse(r#"meta[property="music:song"]"#).unwrap();
                let mut i = 0;
                for element in doc.select(&selector) {
                    i += 1;
                    println!("song: {}", i);
                    let song_url = element.value().attr("content").unwrap();
                    let song_id: Vec<_> = song_url.split('=').collect();
                    let song_id = song_id[1];
                    // println!("{}:{}", song_url, song_id);
                    let body = get_html_page(song_url).await;
                    let doc = Html::parse_document(&body);
                    let selector = Selector::parse(r#"meta[name="twitter:image"]"#).unwrap();
                    let image_url = doc
                        .select(&selector)
                        .next()
                        .unwrap()
                        .value()
                        .attr("content")
                        .unwrap();
                    let image_url = image_url.replace("600x600", "1000x1000");
                    // println!("image_url: {}", image_url);
                    let selector = Selector::parse(r#"meta[name="keywords"]"#).unwrap();
                    let keywords: Vec<&str> = doc
                        .select(&selector)
                        .next()
                        .unwrap()
                        .value()
                        .attr("content")
                        .unwrap()
                        .split(", ")
                        .collect();
                    // println!("{:?}", keywords);
                    let album_name = keywords[1];
                    let singer = keywords[2];
                    // println!("{}", album_name);
                    let album_dir = album_name.to_owned() + " - " + singer;

                    create_dir_all(&album_dir)?;

                    let original_pwd = env::current_dir().unwrap();
                    let mut new_pwd = original_pwd.clone();
                    new_pwd.push(album_dir);
                    env::set_current_dir(new_pwd).unwrap();

                    let _res1 = Command::new("bash")
                        .arg("-c")
                        .arg(
                            "curl -u dzshgc:a123456. https://aplossless.decrypt.site/us/"
                                .to_owned()
                                + song_id,
                        )
                        .status()
                        .unwrap();
                    // println!("res1: {:?}", str::from_utf8(&_res1.stderr).unwrap());

                    thread::sleep(time::Duration::from_secs(10));

                    let _res2 = Command::new("bash")
                        .arg("-c")
                        .arg(
                            "curl -u dzshgc:a123456. -J -O https://aplossless.decrypt.site/us/"
                                .to_owned()
                                + song_id,
                        )
                        .status()
                        .unwrap();
                    // println!("res2: {:?}", str::from_utf8(&_res2.stderr).unwrap());

                    let _res3 = Command::new("bash")
                        .arg("-c")
                        .arg(
                            "curl --output ".to_owned()
                                + "\""
                                + album_name
                                + "\".jpg "
                                + &image_url,
                        )
                        .status()
                        .unwrap();
                    // println!("res3: {:?}", str::from_utf8(&_res3.stderr).unwrap());

                    thread::sleep(time::Duration::from_secs(1));

                    env::set_current_dir(original_pwd).unwrap();
                }
            }
        }
    }

    Ok(())
}
