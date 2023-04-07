#[allow(unused_imports)]
use markup5ever::rcdom::Node;
use reqwest::blocking::Client;
use std::rc::Rc;
use std::sync::mpsc;
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
    link: String,
}

//
//
// dodaj učitavanje celog BOM fajla,
// koristeći standardnu interaktivnu petlju, dok se korisnik premišlja o odluci, redom učitavaj
// htmlove
//
//

//Dodaj opis prodavnica u neki json fajl, ružno je u main-u
fn main() {
    let mut parts: Vec<(String, i32)> = Vec::new();

    loop {
        println!("Unesi ime dela ili \"kraj kupovine\"");
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer).unwrap();
        if answer.contains("kraj kupovine") {
            break;
        }
        parts.push((answer.clone(), 0));
        println!("Unesi broj komada");
        let num = get_usize_from_input(1000) as i32;
        let len = parts.len();
        parts[len - 1].1 = num;
    }

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

    prodavnice.push(Prodavnica {
        name: "Proelekronik".to_string(),
        query_fn: query_proelektronik,
        korpa: Korpa {
            artikli: Vec::new(),
            ukupna_cena: 0.0,
        },
        url: (
            "http://www.proelectronic.rs/pretraga?cat=0&q=".to_string(),
            "".to_string(),
        ),
    });

    prodavnice.push(Prodavnica {
        name: "Interhit".to_string(),
        query_fn: query_interhit,
        korpa: Korpa {
            artikli: Vec::new(),
            ukupna_cena: 0.0,
        },
        url: (
            "http://www.interhit.rs/pretraga?orderby=position&orderway=desc&search_query="
                .to_string(),
            "&submit_search.x=0&submit_search.y=0".to_string(),
        ),
    });

    let (tx, rx): (
        std::sync::mpsc::Sender<Vec<String>>,
        std::sync::mpsc::Receiver<Vec<String>>,
    ) = mpsc::channel();

    spawn_download_thread(&parts, &prodavnice, tx);
    for part in parts.iter() {
        let mut to_be_removed = Vec::new();
        let mut htmls = rx.recv().unwrap();
        for (n, prodavnica) in prodavnice.iter_mut().enumerate() {
            if htmls[0].len() < 10 {
                println!("Prodavnica {} nije više pod istim domenom", prodavnica.name);
                to_be_removed.push(n);
                continue;
            }
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
        for prodavnica in prodavnice.iter_mut() {
            prodavnica.korpa.artikli.last_mut().unwrap().1 = part.1 as usize;
        }
    }

    println!("Kupovina gotova, lista je:");
    for prodavnica in prodavnice.iter_mut() {
        let name = trunc_padd(&prodavnica.name, 25);
        print!("{}|", name);
    }
    println!("");
    for i in 0..prodavnice[0].korpa.artikli.len() {
        for prodavnica in prodavnice.iter_mut() {
            let name = trunc_padd(&prodavnica.korpa.artikli[i].0.name, 25);
            print!("{}|", name,);
        }
        println!("");
        for prodavnica in prodavnice.iter_mut() {
            let artikal = prodavnica.korpa.artikli[i].clone();
            let price = trunc_padd_start(&format!("{} RSD x {}", artikal.0.price, artikal.1), 25);

            print!("{}|", price,);
            prodavnica.korpa.ukupna_cena += artikal.0.price * artikal.1 as f32;
        }
        println!("");
    }

    for prodavnica in prodavnice.iter() {
        let ukupna_cena = trunc_padd_start(&format!("{} RSD", prodavnica.korpa.ukupna_cena), 25);
        print!("{}|", ukupna_cena);
    }
    println!("");

    for prodavnica in prodavnice.iter() {
        println!("{}", prodavnica.name);
        for part in prodavnica.korpa.artikli.iter() {
            println!("{} -> {}", trunc_padd(&part.0.name, 25), part.0.link);
        }
    }
}

fn spawn_download_thread(
    parts: &Vec<(String, i32)>,
    prodavnice: &Vec<Prodavnica>,
    tx: std::sync::mpsc::Sender<Vec<String>>,
) {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0(X11;Linux x86_64;rv10.0)")
        .build()
        .unwrap();

    let part_d = parts.clone();
    let pro_d = prodavnice.clone();
    thread::spawn(move || {
        let parts = part_d;
        let prodavnice = pro_d;

        for part in parts {
            let htmls = load_htmls(&part.0, &client, &prodavnice);
            tx.send(htmls);
        }
    });
}

fn trunc_padd(string: &str, n: usize) -> String {
    let mut out = String::from(string);
    if string.len() > n {
        out = string[0..n].to_string();
    } else {
        while out.len() != n {
            out.push_str(" ");
        }
    }
    out
}

fn trunc_padd_start(string: &str, n: usize) -> String {
    let mut out = String::from(string);
    if string.len() > n {
        out = string[0..n].to_string();
    } else {
        while out.len() != n {
            out.insert_str(0, " ");
        }
    }
    out
}

fn print_all_articles(
    n: usize,
    artikli: Option<Vec<Part>>,
    list: &mut Vec<(Part, usize)>,
) -> Option<usize> {
    match artikli {
        Some(artikli) => {
            for (n, artikl) in artikli.iter().enumerate() {
                let name = trunc_padd(&format!("{n}.{}", artikl.name), 30);
                let price = trunc_padd_start(&artikl.price.to_string(), 10);
                println!("{}{} RSD", name, price);
            }

            println!("Unesi rednu cifru gore navedenog artikala koji želiš da kupiš");
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
                        name: "None".to_string(),
                        price: 0.0,
                        description: "None".to_string(),
                        link: "".to_string(),
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
        let (name, link) = match find_by_class(&artikl_obj, "div", "pil_nameshort") {
            Some(n) => {
                let name = n.text();
                let name = name.split("| ").nth(1).unwrap();
                (
                    name.to_string(),
                    n.tag("a").find().unwrap().get("href").unwrap(),
                )
            }
            None => {
                println!("Ćorak");
                continue;
            }
        };

        let link = format!("http://www.kelco.rs{}", &link[2..link.len()]);

        let price = match find_by_class(&artikl_obj, "div", "svecene") {
            Some(n) => trim_whitespace(&n.text().lines().nth(1).unwrap())
                .split_whitespace()
                .nth(0)
                .unwrap()
                .replace(".", "")
                .replace(",", ".")
                .parse::<f32>()
                .unwrap(),

            None => {
                println!("Ćorak");
                continue;
            }
        };

        let artikl = Part {
            name,
            price,
            description: "".to_string(),
            link,
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
        let (name, link) = match find_by_class(&tr, "div", "text-block") {
            Some(n) => (
                trim_whitespace(&n.tag("a").find().unwrap().text()),
                n.tag("a").find().unwrap().get("href").unwrap(),
            ),
            None => continue,
        };

        let price = match find_by_class(&tr, "div", "price") {
            Some(n) => {
                let price_str = trim_whitespace(&n.text());
                price_str
                    .split_whitespace()
                    .nth(0)
                    .unwrap()
                    .replace(".", "")
                    .replace(",", ".")
                    .parse::<f32>()
                    .unwrap()
            }
            None => continue,
        };

        let artikl = Part {
            name,
            price,
            description: "".to_string(),
            link,
        };

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
        //podeli na komade
        let title_node = match find_by_class(&tr, "h4", "list-view__title") {
            Some(t) => t,
            None => continue,
        };

        let (name, link) = match title_node.tag("a").find() {
            Some(t) => (trim_whitespace(&t.text()), t.get("href").unwrap()),
            None => continue,
        };

        let link = format!("https://www.mgelectronic.rs/{link}");
        let price = match find_by_class(&tr, "td", "list-view__cell list-view__price") {
            Some(t) => trim_whitespace(&t.tag("li").find().unwrap().text())
                .split("(")
                .nth(0)
                .expect("MGELEKTRONIK, failed to split at (, promenili su šablon")
                .replace(".", "")
                .replace(",", ".")
                .parse::<f32>()
                .expect("MGELEKTRONIK, failed to split parse to float"),
            None => continue,
        };

        let artikl = Part {
            name,
            price,
            description: "".to_string(),
            link,
        };

        artikli.push(artikl);
    }

    Some(artikli)
}

fn query_proelektronik(html: String) -> Option<Vec<Part>> {
    let soup = Soup::new(&html);
    let search_div = match find_by_class(&soup, "div", "row row-fix-flex") {
        Some(a) => a,
        None => return None,
    };

    //trs je lista proizvoda
    let mut artikli = Vec::new();
    let trs = find_all_by_class(&search_div, "div", "col-lg-3 col-md-4 col-sm-6 col-xs-12");
    for tr in trs {
        let (name, link) = match find_by_class(&tr, "div", "xs-product-name") {
            Some(x) => match x.tag("a").find() {
                Some(a) => (trim_whitespace(&a.text()), a.get("href").unwrap()),
                None => continue,
            },
            None => continue,
        };

        let link = format!("http://www.proelectronic.rs/{}", link);

        let price = match find_by_class(&tr, "div", "xs-product-price") {
            Some(x) => trim_whitespace(&x.text()),
            None => continue,
        };

        let price = price
            .split_whitespace()
            .nth(0)
            .unwrap()
            .replace(",", "")
            .parse::<f32>()
            .unwrap();

        let artikl = Part {
            name,
            price,
            description: "".to_string(),
            link,
        };

        artikli.push(artikl);
    }

    if artikli.len() == 0 {
        return None;
    }
    Some(artikli)
}

fn query_interhit(html: String) -> Option<Vec<Part>> {
    let soup = Soup::new(&html);
    let search_div = match find_by_id(&soup, "ul", "product_list") {
        Some(a) => a,
        None => return None,
    };

    //trs je lista proizvoda
    let mut artikli = Vec::new();
    let trs = search_div.tag("li").find_all();
    for tr in trs {
        let name_div = match find_by_class(&tr, "div", "product-shop") {
            Some(d) => d,
            None => continue,
        };
        let name_div = match name_div.tag("a").find() {
            Some(f) => f,
            None => continue,
        };

        let price_div = match find_by_class(&tr, "span", "price") {
            Some(d) => d,
            None => continue,
        };

        let name = name_div.text();
        let link = name_div.get("href").unwrap();

        let price = price_div
            .text()
            .split_whitespace()
            .nth(0)
            .unwrap()
            .replace(".", "")
            .replace(",", ".")
            .parse::<f32>()
            .unwrap();

        let artikl = Part {
            name,
            price,
            description: "".to_string(),
            link,
        };

        artikli.push(artikl);
    }

    Some(artikli)
}

fn find_by_id<T: soup::QueryBuilderExt>(
    soup: &T,
    tag: &str,
    class: &str,
) -> Option<Rc<markup5ever::rcdom::Node>> {
    let divs = soup.tag(tag).find_all();

    for div in divs {
        let class_loc = match div.get("id") {
            Some(c) => c,
            None => continue,
        };

        if class_loc == class {
            return Some(div);
        }
    }

    None
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
        handles.push(thread::spawn(move || (n, client.get(&url).send())));
    }

    for handle in handles {
        let out = handle.join().unwrap();
        htmls[out.0] = match out.1 {
            Ok(v) => v.text().unwrap(),
            Err(_) => continue,
        };
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
        let index = input.lines().nth(0).unwrap().parse::<usize>();
        //        let index = input[0..input.len() - 1].parse::<usize>();
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
