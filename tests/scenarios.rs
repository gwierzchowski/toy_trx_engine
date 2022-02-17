use argh::FromArgs;
use rust_decimal_macros::dec;

use toy_trx_engine::{Args, process};

// TODO: Write and use function that count lines in test file.

#[async_std::test]
async fn dep_with() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &["tests/samples/s_dep_with.csv"]
    ).expect("correxct command line");
    let (rec, accounts) = process(&arg).await.expect("success");
    assert_eq!(rec, 3);
    let total1 = accounts.get(&1).expect("client 2 in test file").total();
    let total2 = accounts.get(&2).expect("client 2 in test file").total();
    assert_eq!(total1, dec!(4.5));
    assert_eq!(total2, dec!(10.0));
}

#[async_std::test]
async fn dep_dis_with_res() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/s_dep_dis_with_res.csv",
            "--comments"
        ]
    ).expect("correxct command line");
    let (rec, accounts) = process(&arg).await.expect("success");
    assert_eq!(rec, 5); // trx#4 and 7 should fail
    let total1 = accounts.get(&1).expect("client 2 in test file").total();
    let total2 = accounts.get(&2).expect("client 2 in test file").total();
    assert_eq!(total1, dec!(7.5));
    assert_eq!(total2, dec!(10.0));
}

#[async_std::test]
async fn dep_dis_with_chb() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/s_dep_dis_with_chb.csv",
            "--comments"
        ]
    ).expect("correxct command line");
    let (rec, accounts) = process(&arg).await.expect("success");
    assert_eq!(rec, 4); // trx#4 and 6 should fail
    let total1 = accounts.get(&1).expect("client 2 in test file").total();
    let total2 = accounts.get(&2).expect("client 2 in test file").total();
    assert_eq!(total1, dec!(0.0));
    assert_eq!(total2, dec!(10.0));
    assert!(accounts.get(&1).expect("client 2 in test file").locked);
}

#[async_std::test]
async fn dep_dis_res_chb() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/s_dep_dis_res_chb.csv",
            "--comments"
        ]
    ).expect("correxct command line");
    let (rec, accounts) = process(&arg).await.expect("success");
    assert_eq!(rec, 8); // all trx should succeeded
    let total = accounts.get(&1).expect("client 2 in test file").total();
    assert_eq!(total, dec!(26.0));
    assert!(accounts.get(&1).expect("client 2 in test file").locked);
}
