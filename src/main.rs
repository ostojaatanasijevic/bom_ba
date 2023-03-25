#[allow(unused_imports)]
use markup5ever::rcdom::Node;
use std::rc::Rc;
use std::time::Instant;

use soup::prelude::*;
use soup::NodeExt;

struct Korpa {
    mg_artikli: Vec<(Part, usize)>,
    mikro_artikli: Vec<(Part, usize)>,
    mg_ukupno: f32,
    mikro_ukupno: f32,
}

#[derive(Clone)]
struct Part {
    name: String,
    price: f32,
    description: String,
}

fn main() {
    let mut korpa = Korpa {
        mg_artikli: Vec::new(),
        mikro_artikli: Vec::new(),
        mg_ukupno: 0.0,
        mikro_ukupno: 0.0,
    };
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0(X11;Linux x86_64;rv10.0)")
        .build()
        .unwrap();

    loop {
        println!("Unesi ime komponente ili \"kraj kupovine\"");

        let mut query = String::new();
        std::io::stdin().read_line(&mut query).unwrap();
        if query.contains("kraj kupovine") {
            break;
        }

        let artikli = query_mikro_princ(&client, &query);
        println!("MIKROPRINC: \n");
        append_new_article(artikli, &mut korpa.mikro_artikli);

        let artikli = query_mg_electronic(&client, &query);
        println!("MGELECTRINIC: \n");
        append_new_article(artikli, &mut korpa.mg_artikli);

        println!("Unesi broj komada: ");
        let komada = get_usize_from_input(1000);

        korpa.mg_artikli.last_mut().unwrap().1 = komada;
        korpa.mikro_artikli.last_mut().unwrap().1 = komada;
    }

    println!("Kupovina gotova, lista je:");
    for i in 0..korpa.mg_artikli.len() {
        let artikal = korpa.mg_artikli[i].clone();
        print!(
            "{} @ {} x {} \t",
            artikal.0.name, artikal.0.price, artikal.1
        );
        korpa.mg_ukupno += artikal.0.price * artikal.1 as f32;

        let artikal = korpa.mikro_artikli[i].clone();
        print!(
            "{} @ {} x {} \t",
            artikal.0.name, artikal.0.price, artikal.1
        );
        korpa.mikro_ukupno += artikal.0.price * artikal.1 as f32;

        println!("");
    }

    println!("Ukupno:");
    print!("MG : {}\t", korpa.mg_ukupno);
    print!("Mikroprinc : {}\t", korpa.mikro_ukupno);
}

fn append_new_article(artikli: Option<Vec<Part>>, list: &mut Vec<(Part, usize)>) {
    match artikli {
        Some(artikli) => {
            for (n, artikl) in artikli.iter().enumerate() {
                println!("{n}.{} :: {}", artikl.name, artikl.price);
            }

            let index = get_usize_from_input(artikli.len());
            list.push((artikli[index].clone(), 0));
        }
        None => (),
    }
}

fn query_mikro_princ(client: &reqwest::blocking::Client, part_name: &str) -> Option<Vec<Part>> {
    let url = format!(
        "https://www.mikroprinc.com/sr/pretraga?phrase={}&min_price=0.00&max_price=1170833.32&limit=80&sort[price]=1",
        part_name
        );
    let returned_page = client.get(url).send().expect("PHFUCK!").text().unwrap();
    let soup = Soup::new(&returned_page);

    let search_div = find_by_class(&soup, "div", "products-table");

    let out = search_div.unwrap();
    let trs = out.tag("tr").find_all().skip(1);

    let mut artikli = Vec::new();
    //trs je lista proizvoda
    for tr in trs {
        let mut artikl = Part {
            name: "".to_string(),
            price: 0.0,
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
                let price_str = trim_whitespace(&n.text());
                let price_str = price_str.split_whitespace().nth(0);
                let price_str = match price_str {
                    Some(pr) => pr,
                    None => {
                        println!("Mikroprinc, failed to split at whitespace, promenili su šablon");
                        return None;
                    }
                };

                artikl.price = match price_str.replace(",", ".").parse::<f32>() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Mikroprinc, failed to parse to f32, see: {e}");
                        return None;
                    }
                }
            }
        }

        artikli.push(artikl);
    }

    if artikli.len() == 0 {
        return None;
    }
    Some(artikli)
}

fn query_mg_electronic(client: &reqwest::blocking::Client, part_name: &str) -> Option<Vec<Part>> {
    let url = format!(
        "https://www.mgelectronic.rs/search?Cid=0&As=true&Isc=true&Sid=true&q={}&AsUI=false&sos=false&orderby=10&pagesize=100&viewmode=list",
        part_name
    );

    let returned_page = client.get(url).send().expect("PHFUCK!").text().unwrap();
    let soup = Soup::new(&returned_page);

    let search_div = find_by_class(&soup, "div", "search-results");
    let out = match search_div {
        Some(sm) => sm.tag("div").find().unwrap(),
        None => return None,
    };

    let search_div = find_by_class(&out, "table", "list-view");
    let trs = match search_div {
        Some(sm) => sm.tag("tr").find_all().skip(1),
        None => return None,
    };

    let mut artikli = Vec::new();
    //trs je lista proizvoda
    for tr in trs {
        let mut artikl = Part {
            name: "".to_string(),
            price: 0.0,
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
                artikl.price = trim_whitespace(&n.tag("li").find().unwrap().text())
                    .split("(")
                    .nth(0)
                    .expect("MGELEKTRONIK, failed to split at (, promenili su šablon")
                    .replace(",", ".")
                    .parse::<f32>()
                    .expect("MGELEKTRONIK, failed to split parse to float");
            }
        }

        artikli.push(artikl);
    }

    Some(artikli)
}

fn find_by_class<T: soup::QueryBuilderExt>(
    soup: &T,
    tag: &str,
    class: &str,
) -> Option<Rc<markup5ever::rcdom::Node>> {
    let divs = soup.tag(tag).find_all();

    let mut search_div = None;
    for div in divs {
        let class_loc = match div.get("class") {
            Some(c) => c,
            None => continue,
        };

        if class_loc == class {
            search_div = Some(div);
        }
    }

    search_div
}

pub fn trim_whitespace(s: &str) -> String {
    // first attempt: allocates a vector and a string
    let words: Vec<_> = s.split_whitespace().filter(|x| x.len() > 1).collect();
    words.join(" ")
}

pub fn get_usize_from_input(opseg: usize) -> usize {
    let index = loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let index = input[0..input.len() - 1].parse::<usize>();
        match index {
            Ok(val) => {
                if val < opseg {
                    break val;
                } else {
                    println!("Nije u opsegu");
                }
            }
            Err(e) => println!("{e}"),
        }
    };
    index
}
