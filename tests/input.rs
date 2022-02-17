use argh::FromArgs;
use rust_decimal_macros::dec;

use toy_trx_engine::{Args, process};

#[async_std::test]
async fn amt_formats() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &["tests/samples/i_amt_formats.csv"]
    ).expect("correxct command line");
    let (rec, accounts) = process(&arg).await.expect("success");
    assert_eq!(rec, 5);
    let total = accounts.get(&2).expect("client 2 in test file").total();
    assert_eq!(total, dec!(5.4321));
}

#[async_std::test]
async fn no_headers() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/i_no_headers.csv",
            "--no-header"
        ]
    ).expect("correxct command line");
    let (rec, _) = process(&arg).await.expect("success");
    assert!(rec > 0);
}

#[async_std::test]
async fn comments() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &[
            "tests/samples/i_comments.csv",
            "--comments"
        ]
    ).expect("correxct command line");
    let (rec, _) = process(&arg).await.expect("success");
    assert_eq!(rec, 5);
}

#[async_std::test]
async fn spaces_tabs() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &["tests/samples/i_spaces_tabs.csv"]
    ).expect("correxct command line");
    let (rec, _) = process(&arg).await.expect("success");
    assert_eq!(rec, 5);
}

#[async_std::test]
async fn out_of_order() {
    let arg0 = std::env::args().next().unwrap();
    let arg = Args::from_args(
        &[&arg0],
        &["tests/samples/i_ooo.csv"]
    ).expect("correxct command line");
    let (rec, accounts) = process(&arg).await.expect("success");
    assert_eq!(rec, 5);
    let acct_no = accounts.len();
    assert_eq!(acct_no, 5);
}
