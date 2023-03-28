#[allow(unused_imports)]
use markup5ever::rcdom::Node;
use std::rc::Rc;
use std::thread;
use std::time::Instant;

use soup::prelude::*;
use soup::NodeExt;

#[derive(Clone)]
pub struct Prodavnica {
    name: String,
    query_fn: fn(String) -> Option<Vec<Part>>,
    korpa: Korpa,
    url: (String, String),
}

#[derive(Clone)]
struct Korpa {
    artikli: Vec<(Part, usize)>,
    ukupna_cena: f32,
}

#[derive(Clone)]
struct Part {
    name: String,
    price: f32,
    description: String,
}

fn main() {
    let mut prodavnice = Vec::new();

    prodavnice.push(Prodavnica {
        name: "Mikroprinc".to_string(),
        query_fn: query_mikro_princ,
        korpa: Korpa {
            artikli: Vec::new(),
            ukupna_cena: 0.0,
        },
        url: (
            "https://www.mikroprinc.com/sr/pretraga?phrase=".to_string(),
            "&min_price=0.00&max_price=1170833.32&limit=80&sort[price]=1".to_string(),
        ),
    });

    prodavnice.push(Prodavnica {
        name: "MG Elektronik".to_string(),
        query_fn: query_mg_electronic,
        korpa: Korpa {
            artikli: Vec::new(),
            ukupna_cena: 0.0,
        },
        url: (
            "https://www.mgelectronic.rs/search?Cid=0&As=true&Isc=true&Sid=true&q=".to_string(),
            "&AsUI=false&sos=false&orderby=10&pagesize=100&viewmode=list".to_string(),
        ),
    });

    prodavnice.push(Prodavnica {
        name: "Kelco".to_string(),
        query_fn: query_kelco,
        korpa: Korpa {
            artikli: Vec::new(),
            ukupna_cena: 0.0,
        },
        url: (
            "http://www.kelco.rs/katalog/komponente.php?q=".to_string(),
            "&search=".to_string(),
        ),
    });

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

        let mut to_be_removed = Vec::new();
        let mut htmls = load_htmls(&query, &client, &prodavnice);
        for (n, prodavnica) in prodavnice.iter_mut().enumerate() {
            let artikli = (prodavnica.query_fn)(htmls.remove(0));
            println!("{}\n", prodavnica.name);
            let to_remove = print_all_articles(n, artikli, &mut prodavnica.korpa.artikli);
            match to_remove {
                Some(index) => to_be_removed.push(index),
                None => (),
            }
        }

        let mut stare_prodavnice = prodavnice.clone();
        prodavnice = Vec::new();
        //pretakanje
        for (n, prodaja) in stare_prodavnice.iter_mut().enumerate() {
            if !to_be_removed.contains(&n) {
                prodavnice.push(prodaja.clone());
            }
        }

        println!("Unesi broj komada: ");
        let komada = get_usize_from_input(1000);

        for prodavnica in prodavnice.iter_mut() {
            //unsafe, možda nema ništa
            println!("{}", prodavnica.name);
            prodavnica.korpa.artikli.last_mut().unwrap().1 = komada;
        }
    }

    println!("Kupovina gotova, lista je:");
    for i in 0..prodavnice[0].korpa.artikli.len() {
        for prodavnica in prodavnice.iter_mut() {
            let artikal = prodavnica.korpa.artikli[i].clone();
            print!(
                "{} @ {} x {} \t",
                artikal.0.name, artikal.0.price, artikal.1
            );
            prodavnica.korpa.ukupna_cena += artikal.0.price * artikal.1 as f32;
        }
        println!("");
    }

    println!("Ukupno:");
    for prodavnica in prodavnice.iter() {
        print!("{} : {}\t", prodavnica.name, prodavnica.korpa.ukupna_cena);
    }
}

fn print_all_articles(
    n: usize,
    artikli: Option<Vec<Part>>,
    list: &mut Vec<(Part, usize)>,
) -> Option<usize> {
    match artikli {
        Some(artikli) => {
            for (n, artikl) in artikli.iter().enumerate() {
                println!("{n}.{} :: {}", artikl.name, artikl.price);
            }

            let index = get_usize_from_input(artikli.len());
            list.push((artikli[index].clone(), 0));
        }
        None => {
            println!("Tražena komponenta nije dostupna u ovoj prodavnici, deal breaker? y/n");

            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer).unwrap();
            if answer.contains("y") {
                return Some(n);
            } else {
                list.push((
                    Part {
                        name: "Ništa".to_string(),
                        price: 0.0,
                        description: "Ništa".to_string(),
                    },
                    0,
                ));
            }
        }
    }

    None
}

fn query_kelco(html: String) -> Option<Vec<Part>> {
    let soup = Soup::new(&html);
    //kelco ne vraća products_list ako je prazan, tkd , panic, uradi match
    let products_list = match find_by_class(&soup, "div", "products_list") {
        Some(p) => p,
        None => return None,
    };

    let products_list_row = match find_by_class(&products_list, "div", "row") {
        Some(p) => p,
        None => return None,
    };
    let artikli_obj = find_all_by_class(&products_list_row, "div", "asinItem");

    let mut artikli = Vec::new();
    for artikl_obj in artikli_obj {
        let mut artikl = Part {
            name: "".to_string(),
            price: 0.0,
            description: "".to_string(),
        };

        match find_by_class(&artikl_obj, "div", "pil_nameshort") {
            Some(n) => {
                let name = n.text();
                let name = name.split("| ").nth(1).unwrap();
                artikl.name = name.to_string();
            }
            None => {
                println!("Ćorak");
                continue;
            }
        };

        match find_by_class(&artikl_obj, "div", "svecene") {
            Some(n) => {
                let price = trim_whitespace(&n.text().lines().nth(1).unwrap())
                    .split_whitespace()
                    .nth(0)
                    .unwrap()
                    .replace(".", "")
                    .replace(",", ".")
                    .parse::<f32>()
                    .unwrap();
                artikl.price = price;
            }
            None => {
                println!("Ćorak");
                continue;
            }
        };
        artikli.push(artikl);
    }

    if artikli.len() == 0 {
        return None;
    }
    Some(artikli)
}

fn query_mikro_princ(html: String) -> Option<Vec<Part>> {
    let soup = Soup::new(&html);
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

                artikl.price = match price_str.replace(".", "").replace(",", ".").parse::<f32>() {
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

fn query_mg_electronic(html: String) -> Option<Vec<Part>> {
    let soup = Soup::new(&html);
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
                    .replace(".", "")
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

    for div in divs {
        let class_loc = match div.get("class") {
            Some(c) => c,
            None => continue,
        };

        if class_loc == class {
            return Some(div);
        }
    }

    None
}

fn find_all_by_class<T: soup::QueryBuilderExt>(
    soup: &T,
    tag: &str,
    class: &str,
) -> Vec<Rc<markup5ever::rcdom::Node>> {
    let divs = soup.tag(tag).find_all();
    let mut out = Vec::new();
    for div in divs {
        let class_loc = match div.get("class") {
            Some(c) => c,
            None => continue,
        };

        if class_loc == class {
            out.push(div);
        }
    }

    out
}

pub fn load_htmls(
    part_name: &str,
    client: &reqwest::blocking::Client,
    prodavnice: &Vec<Prodavnica>,
) -> Vec<String> {
    let mut urls = Vec::new();

    for prodavnica in prodavnice {
        urls.push(format!(
            "{}{}{}",
            prodavnica.url.0, part_name, prodavnica.url.1
        ));
    }

    let mut handles = Vec::new();
    let mut htmls = vec![String::new(); urls.len()];

    for (n, url) in urls.into_iter().enumerate() {
        let client = client.clone();
        handles.push(thread::spawn(move || {
            (n, client.get(&url).send().expect("PHFUCK!").text().unwrap())
        }));
    }

    for handle in handles {
        let out = handle.join().unwrap();
        htmls[out.0] = out.1;
    }

    htmls
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
