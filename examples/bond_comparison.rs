use std::fs::File;
use std::io::Read;
use finql::market::Market;
use finql::bond::Bond;
use finql::fixed_income::{FixedIncome, get_cash_flows_after};
use serde_json;
use chrono::NaiveDate;

fn main() {
    let mut file = File::open("./examples/Euroboden_deb_bond.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let today = NaiveDate::from_ymd(2019,12,11);
    let bond1: Bond = serde_json::from_str(&data).unwrap();
    let market = Market::new();
    let cfs1 = bond1.rollout_cash_flows(1., &market).unwrap();
    let cfs1 = get_cash_flows_after(&cfs1, today);

    let mut file = File::open("./examples/photon_energy_bond.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let bond2: Bond = serde_json::from_str(&data).unwrap();
    let cfs2 = bond2.rollout_cash_flows(1., &market).unwrap();
    let cfs2 = get_cash_flows_after(&cfs2, today);

    let max_len = std::cmp::max(cfs1.len(),cfs2.len());
    println!("\n\n    Future cash flows bond1      |    Future cash flows bond2");
    println!("===================================================================");
    for i in 0..max_len {
        if i < cfs1.len() && cfs1[i].date > today {
            print!("{}", cfs1[i]);
        } else {
            print!("                               ");
        }
        print!("  |  ");
        if i < cfs2.len() && cfs2[i].date > today {
            print!("{}", cfs2[i]);
        }
        println!("");
    }
}

