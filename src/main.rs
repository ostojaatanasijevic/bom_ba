use std::time::Instant;

use soup::prelude::*;
use soup::NodeExt;

struct Part {
    name: String,
    price: String,
    description: String,
}

fn main() {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0(X11;Linux x86_64;rv10.0)")
        .build()
        .unwrap();

    loop {
        println!("Unesi ime komponente:");

        let mut query = String::new();
        std::io::stdin().read_line(&mut query).unwrap();
        let instant = std::time::Instant::now();

        let artikli = query_mikro_princ(&client, &query);
        println!("MIKROPRINC: \n");
        for (n, artikl) in artikli.iter().enumerate() {
            println!("{n}.{} :: {}", artikl.name, artikl.price);
        }

        let index = loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let index = input[0..input.len() - 1].parse::<usize>();
            match index {
                Ok(val) => {
                    if val < artikli.len() {
                        break val;
                    } else {
                        println!("Nije u opsegu");
                    }
                }
                Err(e) => println!("{e}"),
            }
        };

        println!(
            "{index}.{} :: {}",
            artikli[index].name, artikli[index].price
        );

        println!("MGELECTRINIC: \n");
        let artikli = query_mg_electronic(&client, &query);
        match artikli {
            Some(artikli) => {
                for artikl in artikli {
                    println!("{} :: {}", artikl.name, artikl.price);
                }
            }
            None => (),
        }

        println!("Response time: {:.2}", instant.elapsed().as_secs_f32());
    }
}

fn query_mikro_princ(client: &reqwest::blocking::Client, part_name: &str) -> Vec<Part> {
    let url = format!(
        "https://www.mikroprinc.com/sr/pretraga?phrase={}&min_price=0.00&max_price=1170833.32&limit=80&sort%5Bprice%5D=1",
        part_name
        );
    let returned_page = client.get(url).send().expect("PHFUCK!").text().unwrap();
    let soup = Soup::new(&returned_page);

    let divs = soup.tag("div").find_all();

    //biće to fn jedan dan, možda sa genericima
    let mut search_div = None;
    for div in divs {
        let class = match div.get("class") {
            Some(c) => c,
            None => continue,
        };

        if class == "products-table" {
            search_div = Some(div);
        }
    }

    let out = search_div.unwrap();
    let trs = out.tag("tr").find_all().skip(1);

    let mut artikli = Vec::new();
    //trs je lista proizvoda
    for tr in trs {
        let mut artikl = Part {
            name: "".to_string(),
            price: "".to_string(),
            description: "".to_string(),
        };
        //podeli na komade
        let title_node = tr.tag("div").find_all();
        for n in title_node {
            if n.get("class").unwrap() == "text-block" {
                artikl.name = trim_whitespace(&n.tag("a").find().unwrap().text());
            }
        }
        let price_node = tr.tag("div").find_all();
        for n in price_node {
            if n.get("class").unwrap() == "price" {
                artikl.price = trim_whitespace(&n.text());
            }
        }

        artikli.push(artikl);
    }

    artikli
}

fn query_mg_electronic(client: &reqwest::blocking::Client, part_name: &str) -> Option<Vec<Part>> {
    let url = format!(
        "https://www.mgelectronic.rs/search?Cid=0&As=true&Isc=true&Sid=true&q={}&AsUI=false&sos=false&orderby=10&pagesize=100&viewmode=list",
        part_name
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0(X11;Linux x86_64;rv10.0)")
        .build()
        .unwrap();

    let returned_page = client.get(url).send().expect("PHFUCK!").text().unwrap();
    let soup = Soup::new(&returned_page);

    let divs = soup.tag("div").find_all();

    //biće to fn jedan dan, možda sa genericima
    let mut search_div = None;
    for div in divs {
        let class = match div.get("class") {
            Some(c) => c,
            None => continue,
        };

        if class == "search-results" {
            search_div = Some(div);
        }
    }

    let out = match search_div {
        Some(sm) => sm.tag("div").find().unwrap(),
        None => return None,
    };

    // table list-view
    let table = soup.tag("table").find_all();

    //biće to fn jedan dan, možda sa genericima
    let mut search_div = None;
    for div in table {
        let class = match div.get("class") {
            Some(c) => c,
            None => continue,
        };

        if class == "list-view" {
            search_div = Some(div);
        }
    }

    let trs = match search_div {
        Some(sm) => sm.tag("tr").find_all().skip(1),
        None => return None,
    };

    let mut artikli = Vec::new();
    //trs je lista proizvoda
    for tr in trs {
        let mut artikl = Part {
            name: "".to_string(),
            price: "".to_string(),
            description: "".to_string(),
        };
        //podeli na komade
        let title_node = tr.tag("h4").find_all();
        for n in title_node {
            if n.get("class").unwrap() == "list-view__title" {
                artikl.name = trim_whitespace(&n.tag("a").find().unwrap().text());
            }
        }
        let price_node = tr.tag("td").find_all();
        for n in price_node {
            if n.get("class").unwrap() == "list-view__cell list-view__price" {
                artikl.price = trim_whitespace(&n.tag("li").find().unwrap().text());
            }
        }

        artikli.push(artikl);
    }

    Some(artikli)
}
/*

fn find_by_class<T: soup::QueryBuilderExt>(soup: T, tag: &str, class: &str) -> Option<Rc<T>> {
    let divs = soup.tag(tag).find_all();

    let mut search_div = None;
    for div in divs {
        let class_loc = match div.get("class") {
            Some(c) => c,
            None => continue,
        };

        if class_loc == class {
            println!("{}", div.display());
            search_div = Some(div);
        }
    }

    search_div
}
*/

pub fn trim_whitespace(s: &str) -> String {
    // first attempt: allocates a vector and a string
    let words: Vec<_> = s.split_whitespace().filter(|x| x.len() > 1).collect();

    words.join(" ")
}
