use std::collections::HashMap;

use argh::FromArgs;
use rust_decimal_macros::dec;

use toy_trx_engine::{Args, process};

#[test]
fn amt_formats() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &["tests/samples/i_amt_formats.csv"]
    ).expect("correxct command line");
    let mut accounts = HashMap::new();
    let rec = process(&arg, &mut accounts).expect("success");
    assert_eq!(rec, 5);
    let total = accounts.get(&2).expect("client 2 in test file").total();
    assert_eq!(total, dec!(5.4321));
}

#[test]
fn no_headers() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/i_no_headers.csv",
            "--no-header"
        ]
    ).expect("correxct command line");
    let mut accounts = HashMap::new();
    let rec = process(&arg, &mut accounts).expect("success");
    assert!(rec > 0);
}

#[test]
fn comments() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/i_comments.csv",
            "--comments"
        ]
    ).expect("correxct command line");
    let mut accounts = HashMap::new();
    let rec = process(&arg, &mut accounts).expect("success");
    assert_eq!(rec, 5);
}

#[test]
fn spaces_tabs() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &["tests/samples/i_spaces_tabs.csv"]
    ).expect("correxct command line");
    let mut accounts = HashMap::new();
    let rec = process(&arg, &mut accounts).expect("success");
    assert_eq!(rec, 5);
}

#[test]
fn out_of_order() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &["tests/samples/i_ooo.csv"]
    ).expect("correxct command line");
    let mut accounts = HashMap::new();
    let rec = process(&arg, &mut accounts).expect("success");
    assert_eq!(rec, 5);
    let acct_no = accounts.len();
    assert_eq!(acct_no, 5);
}
