use std::collections::HashMap;

use argh::FromArgs;

use toy_trx_engine::{Args, process};

// TODO: Write and use function that count lines in test file.

#[test]
fn dep_with() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &["tests/samples/s_dep_with.csv"]
    ).expect("correxct command line");
    let mut accounts = HashMap::new();
    let rec = process(&arg, &mut accounts).expect("success");
    assert_eq!(rec, 3);
    let total1 = accounts.get(&1).expect("client 2 in test file").total();
    let total2 = accounts.get(&2).expect("client 2 in test file").total();
    assert_eq!(total1, 4.5);
    assert_eq!(total2, 10.0);
}

#[test]
fn dep_dis_with_res() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/s_dep_dis_with_res.csv",
            "--comments"
        ]
    ).expect("correxct command line");
    let mut accounts = HashMap::new();
    let rec = process(&arg, &mut accounts).expect("success");
    assert_eq!(rec, 5); // trx#4 and 7 should fail
    let total1 = accounts.get(&1).expect("client 2 in test file").total();
    let total2 = accounts.get(&2).expect("client 2 in test file").total();
    assert_eq!(total1, 7.5);
    assert_eq!(total2, 10.0);
}

#[test]
fn dep_dis_with_chb() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/s_dep_dis_with_chb.csv",
            "--comments"
        ]
    ).expect("correxct command line");
    let mut accounts = HashMap::new();
    let rec = process(&arg, &mut accounts).expect("success");
    assert_eq!(rec, 4); // trx#4 and 6 should fail
    let total1 = accounts.get(&1).expect("client 2 in test file").total();
    let total2 = accounts.get(&2).expect("client 2 in test file").total();
    assert_eq!(total1, 0.0);
    assert_eq!(total2, 10.0);
    assert!(accounts.get(&1).expect("client 2 in test file").locked);
}
