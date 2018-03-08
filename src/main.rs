extern crate web_crawler;
extern crate clap;

use web_crawler::*;
use std::fs;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("web_crawler for www.80s.tw")
        .version("1.0")
        .author("Cheng shuchu <284709244@qq.com>")
        .about("get data from www.80s.tw")
        .arg(Arg::with_name("convert"))
        .subcommand(
            SubCommand::with_name("get")
                .help(
                    "get data from website   --pages  --items --details --covers",
                )
                .arg(Arg::with_name("pages").short("p").long("pages").help(
                    "get page lists",
                ))
                .arg(Arg::with_name("items").short("i").long("items").help(
                    "get  items list",
                ))
                .arg(Arg::with_name("details").short("d").long("details").help(
                    "get  details ",
                ))
                .arg(Arg::with_name("covers").short("c").long("covers").help(
                    "get cover images",
                )),
        )
        .subcommand(
            SubCommand::with_name("parse")
                .help("parse  data from local file  --items --details")
                .arg(Arg::with_name("items").short("i").long("items").help(
                    "parse items list",
                ))
                .arg(Arg::with_name("details").short("d").long("details").help(
                    "parse details",
                )),
        )
        .get_matches();

    if let Some(o) = matches.value_of("convert") {
        println!("convert data.csv to data.finished.csv");
        convert();
        return;
    }

    if let Some(matches) = matches.subcommand_matches("get") {
        if matches.is_present("pages") {
            println!("get pages...");
            get_pages();
            return;
        } else if matches.is_present("items") {
            println!("get items...");
            get_items_html();
            return;
        } else if matches.is_present("details") {
            println!("get details...");
            get_details_html();
            return;
        } else if matches.is_present("covers") {
            println!("get covers...");
            get_images();
            return;
        } else {
            println!("something wrong... Please press --help");
            return;
        }
    }
    if let Some(matches) = matches.subcommand_matches("parse") {
        if matches.is_present("items") {
            println!("parse items...");
            parse_items_html();
            return;
        } else if matches.is_present("details") {
            println!("parse details...");
            parse_details_html();
            return;
        } else {
            println!("something wrong... Please press --help");
            return;
        }
    }


    //fs::remove_dir_all("./data");
    //fs::remove_dir_all("./logs");
    println!("get all pages");
    get_pages();
    println!("get all items html");
    get_items_html();
    println!("parse all items html");
    parse_items_html();
    println!("get all detail html");
    get_details_html();
    println!("parse all details html");
    parse_details_html();
    println!("convert data.csv to data.finished.csv");
    convert();

    println!("get cover images");
    get_images();

    println!("finished");
    return;
}
