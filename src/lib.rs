extern crate reqwest;
extern crate victoria_dom;
extern crate rayon;
extern crate csv;

use std::fmt;
use std::borrow::Cow;
use victoria_dom::DOM;
use std::io::{self, Write};
use std::str;
use std::result::Result;
use std::str::FromStr;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::Path;
use std::error::Error;
use std::time::Duration;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct Info {
    pub category: String,
    pub tile: String,
    pub year: String,
    pub area: String,
    pub types: String,
    pub language: String,
    pub length: String,
    pub director: String,
    pub actor: String,
    pub duban: String,
    pub description: String,
    pub season: String,
    pub image_path: String,
    pub image_url: String,
    pub file_path: String,
    pub file_url: String,
    pub url: String,
}
fn to_string(d: &Vec<String>) -> String {
    let mut s = String::new();
    let len = d.len();
    for (ii, i) in d.iter().enumerate() {
        if ii == len - 1 {
            s = s + i;
        } else {
            s = s + i + ",";
        }
    }
    s
}

fn to_string1(d: &Vec<&str>) -> String {
    let mut s = String::new();
    for i in d {
        s = s + i + ",";
    }
    s
}


fn get_image(url: &str) -> Result<Vec<u8>, String> {
    let mut client = reqwest::ClientBuilder::new()
        .danger_disable_certificate_validation_entirely()
        .build()
        .unwrap();
    let mut req: reqwest::Response = match client.get(url).send() {
        Ok(mut req) => req,
        Err(err) => {
            println!("{:?}", err.to_string());
            return Err(err.to_string());
        }
    };
    let mut content = Vec::new();
    req.read_to_end(&mut content);
    Ok(content)

}


fn get_content(url: &str) -> Result<String, String> {
    let mut client = reqwest::ClientBuilder::new()
        .danger_disable_certificate_validation_entirely()
        .build()
        .unwrap();
    let mut req: reqwest::Response = match client.get(url).send() {
        Ok(mut req) => req,
        Err(err) => {
            println!("{:?}", err.to_string());
            return Err(err.to_string());
        }
    };
    let mut content = String::new();
    req.read_to_string(&mut content);
    Ok(content)

}

pub fn get_pages() {
    let ref all = vec!["course", "video", "dm", "zy", "ju", "movie"];
    let len = all.len() as i32;
    let mut count = 0;
    let mut retries = 0;
    while (len != count) {
        count = 0;
        for i in all {
            match get_pages_first_page(i) {
                Ok(ps) => {
                    let items = get_chanel_pages(&ps);
                    if items.is_ok() {
                        count += 1;
                    } else {
                        println!("get page {:?} error ", i);
                    }

                }
                Err(err) => {
                    println!("get page {:?} error {:?} ", i, err.to_string());
                    continue;
                }
            }
        }
        if retries == 2 {
            break;
        }
        println!("get pages retries {:?}  count {:?}", retries, count);
        retries += 1;
    }
}



pub fn get_pages_first_page(chanel: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let mut mpages: Vec<String> = Vec::new();
    let mut mfirst_page: Vec<String> = Vec::new();

    let root = String::from("https://www.80s.tw/");

    let mut url = root + chanel + "/list";
    println!("list url:{:?}", url);
    let mut content = match get_content(&url) {
        Ok(content) => content,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    content = content.replace("&", "");
    let dom = DOM::new(&content);
    let pager = match dom.at("[class=\"pager\"]") {
        Some(pager) => pager,
        None => {
            return Err(String::from("parse pager error"));
        }
    };
    let href = pager.find("a");
    let pages = href.iter()
        .map(|x| x.attr("href").unwrap())
        .collect::<Vec<_>>();

    for p in &pages {
        mpages.push(p.to_string());
    }
    if chanel == "video" {
        let dom = match dom.at("[class=me3 clearfix]") {
            Some(dom) => dom,
            None => {
                return Err(String::from("me3 not found"));
            }
        };
        let dom = dom.find("li > a");
        let first_page = dom.iter()
            .map(|x| x.attr("href").unwrap())
            .collect::<Vec<_>>();
        for i in first_page {
            mfirst_page.push(i.to_string());
        }


    } else {

        let dom = match dom.at("[class=me1 clearfix]") {
            Some(dom) => dom,
            None => {
                return Err(String::from("me1 not found"));
            }
        };
        let dom = dom.find("li > a");
        let first_page = dom.iter()
            .map(|x| x.attr("href").unwrap())
            .collect::<Vec<_>>();
        for i in first_page {
            mfirst_page.push(i.to_string());
        }
    }

    Ok((mpages, mfirst_page))
}

pub fn get_page_items(pages_fp: &(Vec<String>, Vec<String>)) -> Result<Vec<String>, String> {
    let mut items = Vec::new(); //pages_fp.1.clone();
    // let data: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let root = String::from("https://www.80s.tw/");
    let last = match &pages_fp.0.iter().last() {
        &Some(last) => last,
        &None => return Err(String::from("last is none")),

    };

    let last = match last.split('p').last() {
        Some(last) => last,
        None => {
            return Err(String::from("split p last none"));
        }
    };
    //println!("last :{:?}", last);
    let last = match i32::from_str(last) {
        Ok(last) => last + 1,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut first = match &pages_fp.0.iter().next() {
        &Some(first) => first,
        &None => {
            return Err(String::from("first page is none"));
        }
    };
    //println!("first: {:?}", first);
    let p = match first.find('p') {
        Some(p) => p + 1,
        None => {
            return Err(String::from("first page not find p"));
        }
    };
    let mut first = first.clone();
    let (first, _) = first.split_at(p);

    //println!("first: {:?}", first);

    let mut page_urls = Vec::new();
    //let list = data.clone();
    for i in (2..last) {
        //(2..last).into_par_iter().for_each(|i| {
        let url = root.clone() + &first + &i.to_string();
        save_list("logs/pages.csv", url.to_string());
        page_urls.push(url);
        //println!("url :{:?}", url);
    }
    for url in &page_urls {
        let mut content = match get_content(url) {
            Ok(content) => content,
            Err(err) => {
                //return;
                continue;
            }
        };
        save_list("logs/pages_finished.csv", url.clone());
        content = content.replace("&", "");
        let dom = DOM::new(&content);
        if url.contains("/video/") == true {
            let dom = match dom.at("[class=me3 clearfix]") {
                Some(dom) => dom,
                None => {
                    return Err(String::from("me3 not found"));
                }
            };
            let dom = dom.find("li > a");
            let first_page = dom.iter()
                .map(|x| x.attr("href").unwrap())
                .collect::<Vec<_>>();
            for ii in first_page {
                save_list("logs/items.csv", ii.to_string());
                items.push(ii.to_string());
                //list1.push(i.to_string());
            }


        } else {

            let dom = match dom.at("[class=me1 clearfix]") {
                Some(dom) => dom,
                None => {
                    return Err(String::from("me1 not found"));
                }
            };
            let dom = dom.find("li > a");
            let first_page = dom.iter()
                .map(|x| x.attr("href").unwrap())
                .collect::<Vec<_>>();
            for ii in first_page {
                save_list("logs/items.csv", ii.to_string());
                items.push(ii.to_string());
                //list1.push(i.to_string());
            }
        }
    }
    //items.extend_from_slice(&list.lock().unwrap());
    items.append(&mut pages_fp.1.clone());
    //  let ret = get_details(&pages_fp.1.clone());
    // save_data("/data/csc/page1",&ret);

    Ok(items)

}

pub fn get_chanel_pages(pages_fp: &(Vec<String>, Vec<String>)) -> Result<Vec<String>, String> {
    let root = String::from("https://www.80s.tw/");

    for fp in pages_fp.1.iter() {
        if is_finished("logs/items.csv", fp) == false {
            save_list("logs/items.csv", fp.to_string());
        }
    }

    let last = match &pages_fp.0.iter().last() {
        &Some(last) => last,
        &None => return Err(String::from("last is none")),

    };

    let last = match last.split('p').last() {
        Some(last) => last,
        None => {
            return Err(String::from("split p last none"));
        }
    };
    //println!("last :{:?}", last);
    let last = match i32::from_str(last) {
        Ok(last) => last + 1,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut first = match &pages_fp.0.iter().next() {
        &Some(first) => first,
        &None => {
            return Err(String::from("first page is none"));
        }
    };
    //println!("first: {:?}", first);
    let p = match first.find('p') {
        Some(p) => p + 1,
        None => {
            return Err(String::from("first page not find p"));
        }
    };
    let mut first = first.clone();
    let (first, _) = first.split_at(p);

    let mut page_urls = Vec::new();
    //let list = data.clone();
    for i in (2..last) {
        //(2..last).into_par_iter().for_each(|i| {
        let url = root.clone() + &first + &i.to_string();
        if is_finished("logs/pages.csv", &url) == true {
            continue;
        }
        save_list("logs/pages.csv", url.to_string());
        page_urls.push(url);
        //println!("url :{:?}", url);
    }
    Ok(page_urls)

}

pub fn get_items() -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    let mut rdr = csv::Reader::from_path("logs/pages.csv").unwrap();
    for result in rdr.records() {
        let record = result.unwrap();
        let url = &record[0];
        if is_finished("logs/pages_finished.csv", url) == true {
            continue;
        }

        let mut content = match get_content(&url.to_string()) {
            Ok(content) => content,
            Err(err) => {
                //return;
                continue;
            }
        };
        save_list("logs/pages_finished.csv", url.to_string());
        println!("item url {:?}", url.to_string());
        content = content.replace("&", "");
        let dom = DOM::new(&content);
        if url.contains("/video/") == true {
            let dom = match dom.at("[class=me3 clearfix]") {
                Some(dom) => dom,
                None => {
                    println!("me1 not found ");
                    return Err(String::from("me3 not found"));
                }
            };
            let dom = dom.find("li > a");
            let first_page = dom.iter()
                .map(|x| x.attr("href").unwrap())
                .collect::<Vec<_>>();
            for ii in first_page {
                println!("item   {:?} ", ii.to_string());
                save_list("logs/items.csv", ii.to_string());
                items.push(ii.to_string());
                //list1.push(i.to_string());
            }


        } else {

            let dom = match dom.at("[class=me1 clearfix]") {
                Some(dom) => dom,
                None => {
                    println!("me1 not found ");
                    return Err(String::from("me1 not found"));
                }
            };
            let dom = dom.find("li > a");
            let first_page = dom.iter()
                .map(|x| x.attr("href").unwrap())
                .collect::<Vec<_>>();
            for ii in first_page {
                println!("item   {:?} ", ii.to_string());
                save_list("logs/items.csv", ii.to_string());
                items.push(ii.to_string());
                //list1.push(i.to_string());
            }
        }

    }
    Ok(items)
}

pub fn get_images() -> Result<(), String> {
    //let mut items = Vec::new();
    let mut rdr = csv::Reader::from_path("data/data.csv").unwrap();

    let len = rdr.records().count();
    println!("len :{:?}", len);
    let mut count = 0;
    let mut retries = 0;
    let mut rdr = csv::Reader::from_path("data/data.csv").unwrap();
    while (len != count) {
        for result in rdr.records() {
            let record = result.unwrap();
            let url = String::from("https://") + &record[14];
            let file_name = String::from("data/poster/") + &record[13];


            println!("url :{:?}, file_name {:?}", url.to_string(), file_name);
            if is_finished("logs/images_finished.csv", &url) == true {
                count += 1;
                continue;
            }
            let mut content = match get_image(&url.to_string()) {
                Ok(content) => content,
                Err(err) => {
                    println!("{:?}", err.to_string());
                    continue;
                }
            };

            save_list("logs/images_finished.csv", url.to_string());
            count += 1;
            //"http://www.80s.tw", "data/"
            //content = content.into_bytes();
            let file_path = Path::new(&file_name);
            let parent = file_path.parent().unwrap();

            if parent.exists() == false {
                fs::create_dir_all(parent);
            }
            let mut f = fs::File::create(&file_name).unwrap();
            f.write_all(&content);
        }
        if retries == 50 {
            break;
        }
        retries += 1;
        println!("retries :{:?}", retries);

    }
    Ok(())
}


pub fn get_items_html() -> Result<(), String> {
    //let mut items = Vec::new();
    let mut rdr = csv::Reader::from_path("logs/pages.csv").unwrap();

    let len = rdr.records().count();
    println!("len :{:?}", len);
    let mut count = 0;
    let mut retries = 0;
    while (len != count) {
        count = 0;
        let mut rdr = csv::Reader::from_path("logs/pages.csv").unwrap();
        for result in rdr.records() {
            let record = result.unwrap();
            let url = &record[0];

            if is_finished("logs/pages_finished.csv", url) == true {
                count += 1;
                continue;
            }
            let mut content = match get_content(&url.to_string()) {
                Ok(content) => {
                    println!("finished url :{:?}  {:?}", count, url.to_string());
                    content
                }
                Err(err) => {
                    //return;
                    println!("url :{:?}  {:?}", url.to_string(), err.to_string());
                    continue;
                }
            };

            save_list("logs/pages_finished.csv", url.to_string());
            count += 1;
            //"http://www.80s.tw", "data/"
            let mut file_name = url.to_string().replace("https://www.80s.tw", "data/items/");
            file_name = file_name.replace("/-", "/aaaa");
            content = content.replace("&", "");
            let file_path = Path::new(&file_name);
            let parent = file_path.parent().unwrap();

            if parent.exists() == false {
                fs::create_dir_all(parent);
            }
            let mut f = fs::File::create(&file_name).unwrap();
            f.write_all(content.as_bytes());
        }
        if retries == 500 {
            break;
        }
        retries += 1;
        println!("retries :{:?}   count: {:?}", retries, count);

    }
    Ok(())
}

pub fn parse_items_html() {
    let mut rdr = csv::Reader::from_path("logs/pages_finished.csv").unwrap();
    let mut files = Vec::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let url = &record[0];
        files.push(String::from(url));
    }

    files.into_par_iter().for_each(|url| if is_finished(
        "logs/pages_parsed.csv",
        &url,
    ) == false
    {
        save_list("logs/pages_parsed.csv", url.clone());
        let mut file_name = url.replace("https://www.80s.tw", "data/items/");
        file_name = file_name.replace("/-", "/aaaa");
        println!("file_name :{:?}", file_name);

        let mut content = String::new();
        let f = fs::File::open(&file_name);
        if f.is_err() {
            return;
        }
        let mut f = f.unwrap();
        f.read_to_string(&mut content);

        let dom = DOM::new(&content);
        if url.contains("/video/") == true {
            let dom = dom.at("[class=me3 clearfix]");
            if dom.is_some() == true {
                let dom = dom.unwrap();
                let dom = dom.find("li > a");
                let first_page = dom.iter()
                    .map(|x| x.attr("href").unwrap())
                    .collect::<Vec<_>>();
                for ii in first_page {
                    if is_finished("logs/items.csv", ii) == false {
                        save_list("logs/items.csv", ii.to_string());
                    }
                }
            }


        } else {

            let dom = dom.at("[class=me1 clearfix]");
            if dom.is_some() == true {
                let dom = dom.unwrap();
                let dom = dom.find("li > a");
                let first_page = dom.iter()
                    .map(|x| x.attr("href").unwrap())
                    .collect::<Vec<_>>();
                for ii in first_page {
                    if is_finished("logs/items.csv", ii) == false {
                        save_list("logs/items.csv", ii.to_string());
                    }
                }
            }
        }
    });

}

pub fn get_details_html() {
    let mut rdr = csv::Reader::from_path("logs/items.csv").unwrap();
    let len = rdr.records().count();
    let mut retries = 0;
    let mut count = 0;
    while (len != count) {
        let mut rdr = csv::Reader::from_path("logs/items.csv").unwrap();
        count = 0;
        for result in rdr.records() {
            let record = result.unwrap();
            let url = &record[0];
            let file_name = String::from("data/") + url;
            let url = String::from("https://www.80s.tw/") + url;
            if is_finished("logs/items_finished.csv", &url) == true {
                count += 1;
                continue;
            }

            let mut content = match get_content(&url) {
                Ok(content) => content,
                Err(err) => {
                    println!("error :{:?}  {:?}", url, err.to_string());
                    continue; // Err(err.to_string());
                }
            };
            println!("get detail from {:?}", url);
            save_list("logs/items_finished.csv", url.to_string());
            count += 1;
            let file_path = Path::new(&file_name);
            let parent = file_path.parent().unwrap();

            if parent.exists() == false {
                fs::create_dir_all(parent);
            }
            let mut f = fs::File::create(&file_name).unwrap();
            f.write_all(content.as_bytes());
            let ten_millis = std::time::Duration::from_millis(1000);
            let now = std::time::Instant::now();

            std::thread::sleep(ten_millis);
        }
        if retries == 50 {
            break;
        }
        retries += 1;
        println!("get detail retries :{:?}", retries);
    }
}
pub fn parse_details_html() {
    let mut rdr = csv::Reader::from_path("logs/items_finished.csv").unwrap();

    let mut urls = Vec::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let url = &record[0];
        urls.push(String::from(url));
        //let url = String::from("http://www.80s.tw/") + url;
    }
    urls.into_par_iter().for_each(|url| {
        println!("parse file {:?}", url);
        if is_finished("data/data.csv", &url) == false {
            println!(" not in data.csv{:?}", url);
            match get_details_page(&url, 1) {
                Ok(re) => {
                    //save_list("logs/items_finished.csv",url.to_string());
                    //if is_finished("data/data.csv", &url) == false {
                    save_data1("data/data.csv", &re);
                    // }
                }
                Err(err) => {
                    println!("parse {:?}   file error {:?}", &url, err.to_string());
                    // continue;
                }
            }
        } else {
            println!(" in data.csv{:?}", url);
        }
    });
}



fn is_finished(file_name: &str, item: &str) -> bool {
    if Path::new(file_name).exists() == false {
        return false;
    }
    let mut rdr = csv::Reader::from_path(file_name).unwrap();
    for result in rdr.records() {
        // rdr.records().into_par_iter().for_each(|result|{
        let record = result.unwrap();
        let url = &record[0];
        //println!("data url :{:?}",url);
        if url == item {
            return true;
        }
    }
    false
}
pub fn convert() {
    let mut rdr = csv::Reader::from_path("./data/data.csv").unwrap();
    let mut all: Vec<Info> = Vec::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let file_paths = &record[15];
        let file_paths: Vec<&str> = file_paths.split(",").collect();
        let file_links = &record[16];
        let file_links: Vec<&str> = file_links.split(",").collect();
        //let file_links = &record[16].split(",").collect();
        //println!("{:?}  {:?}",file_paths.len(),file_links.len());
        let file_paths_len = file_paths.len();
        let file_links_len = file_links.len();
        let len = usize::min(file_paths_len, file_links_len);
        let mut cat = &record[1];
        if &record[1] == "电影" || &record[1] == "电视剧" {
            cat = "影视";
        }
        if &record[1] == "视频短剧" || &record[1] == "公开课" {
            cat = "短片";
        }

        for i in (0..len) {
            let info = Info {
                url: record[0].to_string(),
                category: cat.to_string(), //record[1].to_string(),
                tile: record[2].to_string(),
                year: record[3].to_string(),
                area: record[4].to_string(),
                types: record[5].to_string(),
                language: record[6].to_string(),
                length: record[7].to_string(),
                director: record[8].to_string(),
                actor: record[9].to_string(),
                duban: record[10].to_string(),
                description: record[11].to_string(),
                season: record[12].to_string(),
                image_path: record[13].to_string(),
                image_url: record[14].to_string(),
                file_path: file_paths[i].to_string(),
                file_url: file_links[i].to_string(),
            };
            all.push(info);

        }
    }
    //println!("{:?}",all.len());
    save_data("./data/data.finished.csv", &all);
}



fn save_data(file_path: &str, data: &Vec<Info>) -> Result<(), Box<Error>> {
    let file_path = Path::new(&file_path);
    let parent = file_path.parent().unwrap();

    if parent.exists() == false {
        fs::create_dir_all(parent);
    }

    let mut wtr = csv::Writer::from_path(file_path)?;

    wtr.write_record(
        &[
            "url",
            "category",
            "title",
            "year",
            "area",
            "type",
            "language",
            "length",
            "director",
            "actor",
            "duban",
            "description",
            "season",
            "image_path",
            "image_url",
            "file_path",
            "file_url",
        ],
    )?;

    for d in data {
        wtr.write_record(
            [
                &d.url,
                &d.category,
                &d.tile,
                &d.year,
                &d.area,
                &d.types,
                &d.language,
                &d.length,
                &d.director,
                &d.actor,
                &d.duban,
                &d.description,
                &d.season,
                &d.image_path,
                &d.image_url,
                &d.file_path,
                &d.file_url,
            ].iter(),
        )?;
    }
    wtr.flush()?;
    Ok(())



}
fn save_data1(file_path: &str, d: &Info) -> Result<(), Box<Error>> {
    let file_path = Path::new(&file_path);

    let file_ex = file_path.exists();
    let parent = file_path.parent().unwrap();

    if parent.exists() == false {
        fs::create_dir_all(parent);
    }
    let file = fs::OpenOptions::new().create(true).append(true).open(
        file_path,
    )?;

    let mut wtr = csv::Writer::from_writer(file);
    if file_ex == false {
        wtr.write_record(
            &[
                "url",
                "category",
                "title",
                "year",
                "area",
                "type",
                "language",
                "length",
                "director",
                "actor",
                "duban",
                "description",
                "season",
                "image_path",
                "image_url",
                "file_path",
                "file_url",
            ],
        )?;
    }
    wtr.write_record(
        [
            &d.url,
            &d.category,
            &d.tile,
            &d.year,
            &d.area,
            &d.types,
            &d.language,
            &d.length,
            &d.director,
            &d.actor,
            &d.duban,
            &d.description,
            &d.season,
            &d.image_path,
            &d.image_url,
            &d.file_path,
            &d.file_url,
        ].iter(),
    )?;
    wtr.flush()?;
    Ok(())



}
fn save_list(file_path: &str, d: String) -> Result<(), Box<Error>> {
    let file_path = Path::new(&file_path);
    let parent = file_path.parent().unwrap();

    if parent.exists() == false {
        fs::create_dir_all(parent);
    }
    let file = fs::OpenOptions::new().create(true).append(true).open(
        file_path,
    )?;

    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record([&d].iter())?;
    wtr.flush()?;
    Ok(())

}


pub fn get_details(item_list: &Vec<String>) -> Vec<Info> {
    let mut details = Vec::new();
    let data: Arc<Mutex<Vec<Info>>> = Arc::new(Mutex::new(Vec::new()));
    let details1 = data.clone();
    //   item_list.into_par_iter().for_each(|i| {
    for i in item_list {
        let url = String::from("https://www.80s.tw/") + i;

        match get_details_page(&url, 0) {
            Ok(re) => save_data1("data/data.csv", &re),
            Err(_) => {
                println!("get {:?}   time out ", &url);
                continue;
            }
        };
    }
    details.extend_from_slice(&details1.lock().unwrap());
    details
}


fn split_str(ss: &str) -> Vec<String> {
    let mut ss = ss.to_string();
    let url = ss.replace("//", "");
    let mut a = String::new();
    if url.contains("upload") == true {
        a = url.split("upload").last().unwrap().to_string();
    } else {
        a = url.split("/").last().unwrap().to_string();
    }
    let mut sss: Vec<String> = vec![String::new(), String::new()];
    sss[0] = url.clone();
    sss[1] = a;
    sss
}

fn get_ji(s: &str) -> String {
    let mut ret = String::new();
    if s.contains("[第") && s.contains("季]") {
        let f: Vec<&str> = s.split("[第").collect();
        let e: Vec<&str> = f[1].split("季]").collect();
        ret = e[0].to_string();
    }
    ret
}



pub fn get_details_page(url: &str, tt: i32) -> Result<Info, String> {
    let mut category = String::new();
    let t = {
        if url.contains("/zy/") {
            category = String::from("综艺");
            "zy"
        } else if url.contains("/movie/") {
            category = String::from("电影");
            "movie"
        } else if url.contains("/ju/") {
            category = String::from("电视剧");
            "ju"
        } else if url.contains("/dm/") {
            category = String::from("动漫");
            "dm"
        } else if url.contains("/video/") {
            category = String::from("视频短剧");
            "video"
        } else if url.contains("/course/") {
            category = String::from("公开课");
            "course"
        } else {
            return Err(format!("没有次类型 :{:?}", url));
        }

    };

    println!("get details :{:?}", url);
    let mut content = String::from("");
    if tt == 0 {
        content = match get_content(&url) {
            Ok(content) => content,
            Err(err) => {
                return Err(err.to_string());
            }
        };
    } else {
        let file_name = url.replace("https://www.80s.tw", "data/");
        println!("file_name :{:?}", file_name);
        let mut f = fs::File::open(&file_name).unwrap();
        f.read_to_string(&mut content);

    }

    //println!("{:?}", content);
    content = content.replace("&", "");
    let dom = DOM::new(&content);
    println!("dom");
    if dom.at("#minfo").is_none() {
        println!("minfo is none");
        return Err(String::from("minfo is none"));
    }
    let minfo = dom.at("#minfo").unwrap();
    let mut image = String::new();
    let mut images = Vec::new();
    match minfo.at("[class=img]") {
        Some(image1) => {
            let image1 = &image1.childs(None)[0];
            if image1.attr("src").is_none() {
                println!("image src is none");
            } else {

                let image1 = image1.attr("src").unwrap().clone();
                image = image1.to_string();

                images = split_str(&image);
                println!("image :{:?}", image);
            }
        }

        None => {
            println!("image not found");
        }
    };

    if minfo.at("[class=info]").is_none() {
        println!("info is none");
        return Err(String::from("info is none"));
    }

    let info = minfo.at("[class=info]").unwrap();
    /*  if info.at("h").is_none() {
        println!("h is none");
    }
  */
    let tile = match info.at("h1") {
        Some(tile) => tile.text(),
        None => String::from("未知"),
    };
    let mut year = String::from("未知");
    let mut area = String::from("未知");
    let mut types = String::from("未知");
    let mut language = String::from("未知");
    let mut director = String::from("未知");
    let mut length = String::from("未知");
    let mut actor = String::from("未知");
    let mut duban = String::from("未知");

    println!("tile: {:?}", tile);
    let mut season = get_ji(&tile);
    let actor1 = info.find("span");
    for a in actor1.iter() {
        if a.text().starts_with("演员") {
            let ss = a.following(None);
            let mut tmp = Vec::new();
            for s in &ss {
                tmp.push(s.text());
            }
            actor = to_string(&tmp);
            break;
        }
    }
    println!("actors :{:?}", actor);

    let span = info.find("span[class=span_block]");


    let len = span.len();
    for i in (0..len) {
        if span.get(i).is_none() {
            println!("span is not found");
            continue;
        }
        let span = span.get(i).unwrap();

        let mut field = String::new();
        let types1 = span.find("a");
        let len1 = types1.len();
        if len1 > 0 {
            let types1 = types1.iter().map(|x| x.text()).collect::<Vec<_>>();
            field = to_string(&types1);
            field = field.replace("nbsp;", "");
        //println!("aaaaaa{:?}", types1);
        } else {
            field = span.text();
            field = field.replace("nbsp;", "");
            // println!("aaaaaa{:?}", span.text());
        }
        let childs = span.childs(None);
        if childs.len() > 0 {
            let tt = childs[0].text();
            if tt.starts_with("类型") {
                types = field;

            } else if tt.starts_with("地区") {
                area = field;

            } else if tt.starts_with("语言") {
                language = field;

            } else if tt.starts_with("年代") || tt.starts_with("上映日期") {
                year = field;

            } else if tt.starts_with("片长") {
                length = field;

            } else if tt.starts_with("豆瓣评分") {
                duban = field;

            } else if tt.starts_with("导演") {
                director = field;

            }
        }


    }


    let movie_content = match info.at("#movie_content") {
        Some(movie_content) => movie_content.text(),
        None => String::from(""),
    };
    println!("content :{:?}", movie_content);


    let mut addresses = vec![];
    if dom.at("#cpage").is_some() {
        let ul = dom.at("#cpage").unwrap();

        for li in ul.childs(None).iter() {
            let ad = li.find("span");
            let mut v = ad.iter().map(|x| x.text()).collect::<Vec<_>>();
            addresses.append(&mut v);
        }
    }
    let mut add_url = url.to_string();
    if tt != 0 {
        add_url = add_url.replace("data/", "https://www.80s.tw/");
    }
    let alist = vec!["/bd-1", "/hd-1", "/mp4-1", "/bt-1"];

    fn get_a(aa: &Vec<String>, t: &str) -> bool {
        for a in aa {
            if a.starts_with(t) {
                return true;
            }
        }
        false
    }

    if get_a(&addresses, "平板") == true {
        add_url = add_url + alist[0];
    } else if get_a(&addresses, "手机") == true {

        add_url = add_url + alist[1];
    } else if get_a(&addresses, "小MP4") == true {

        add_url = add_url + alist[2];
    } else if get_a(&addresses, "电视") == true {

        add_url = add_url + alist[3];
    }

    //let add_url = "http://www.80s.tw/movie/4929";
    println!("add_url:{:?}", add_url);
    let mut content = match get_content(&add_url) {
        Ok(content) => content,
        Err(err) => {
            return Err(format!("get content err {:?}", add_url));
        }
    };

    content = content.replace("&", "999999999");
    // println!("---------------------------------------- {:?}", content);


    let mut link = String::new();
    let mut text = String::new();
    let mut texts: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();

    let dom = DOM::new(&content);
    if dom.at("[class=dllist1]").is_some() {
        let ul = dom.at("[class=dllist1]").unwrap();
        for li in ul.childs(None).iter() {
            let dl_name = match li.at("[class=dlname nm]") {
                Some(dl_name) => dl_name,
                None => {
                    println!("dlname nm not found");
                    continue;
                }
            };
            let link1 = dl_name.find("a");
            let text1 = link1.iter().map(|x| x.text()).collect::<Vec<_>>();
            let href = link1
                .iter()
                .map(|x| x.attr("href").unwrap())
                .collect::<Vec<_>>();
            //println!("link:{:?}", text);
            //println!("link:{:?}", href);
            for t in text1 {
                let tmp = t.replace("999999999", "&");

                texts.push(tmp.to_string());
            }
            //links.push(href.to_string());
            for h in &href {
                let tmp = h.replace("999999999", "&");
                println!("hhhhhhhhhhhhhhhh {:?}", tmp);
                links.push(tmp.to_string());
            }

        }
        link = to_string(&links);
        text = to_string(&texts);

    }
    Ok(Info {
        url: url.to_string(),
        category: category,
        tile: tile,
        year: year,
        area: area,
        types: types,
        language: language,
        length: length,
        director: director,
        actor: actor,
        duban: duban,
        description: movie_content,
        season: season,
        image_path: images[1].clone(),
        image_url: images[0].clone(),
        file_path: text,
        file_url: link,
    })
}
