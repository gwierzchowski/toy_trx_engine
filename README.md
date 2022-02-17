# Toy Transaction Engine

This is small transaction engine written as exercise in Rust language. It supports 5 types of transactions:

- Deposit
- Withdrawal
- Dispute (transaction questioning)
- Resolution 
- Chargeback 

More comprehensive requirements are described in separate document.

This repository contain 2 versions of program: `main` is normal, multi-threaded version, while `simple` branch contain simplified, single-thread version. This simple version was created because of two reasons: serve as reference for performance checks, and to compare results (as simpler less likely contain bugs).

The `toy_trx_engine` program reads transactions data from CSV file passed as first positional argument and prints account's balances to standard output. It supports several options. Run it with `--help` argument figure out all options.
In case of errors in particular transactions, program will print message on `stderr` and continue processing.

## Design decisions

My first idea was creating transaction state machine with transaction passing thru several states. I eventually abandoned this idea in charge of simpler solution, where all business logic is coded in well defined one place - `commit` trait method implementations. E.g. all 'deposit' transaction characteristic code is encoded in `deposit.rs` source file. This simplifies adding new transaction types, and as I believe is less error prone. E.g. it is easy to identify small portion of critical code to be carefully reviewed.

I also tried to keep dependencies under control, taking only those that are really needed or small and safe. Program does not contain any direct code using `unsafe` annotation. Dependencies were checked using audit cargo extension.

I used `rust_decimal` crate to support money calculation. It is quite widely used crate and utilized for money-specialized crate: `rusty-money` so I assume it passed some testing. This is acceptable for toy-tool. But for real production system, I would either write more extensive and comprehensive test suite to prove library calculates money properly (including performance tests) or use 128-bit integers internally to calculate money and only convert for i/o - it may be faster.

For transaction types dispatching I used enum-based dispatching supported by 3rd party crate `enum_dispatch` that limited boiler-plate code. The code would be maybe simpler if I use dynamic dispatching, but this would be at cost of some extra memory allocations and virtual methods calls, so I takes in my opinion fair performance / simplicity compromise.

In case off this particular tool major processing (transaction commits) is being done in memory without intensive system calls so I used mostly system threads to achieve congruency, where clients pool is partitioned between several shards, that are being processed by different workers, where each client is processed always by one worker, what simplifies processing - there is no need to wait / synchronize. Specification did not ordered to implement "transfer" transaction. In such case some synchronization mechanism would have to be implemented. __Real Transactional__ system typically performs a way more i/o and network calls, so system threads should be replaced in it with asynchronous tasks to achieve better performance and scalability.

I assumed that external transaction IDs (`tx`) are unique for particular client (it is weaker assumption then suggested in requirements). Checking global transaction ID uniqueness would bring additional cost - not strictly necessary for system correctness. There is one exception - if transaction is rejected I do not remember its id - i.e. I allow another transaction for the same client with the same ID to be later present. It is kind of compromise support for such scenario would need extra processing and memory, and lack of it may cause debugging harder (looking at input file we are not sure to which transaction reference applies). In my opinion implementing such check in real system would be recommended, but not necessarily for toy-like.

## Status / Remaining work

Provided points are in more-less ordered by priority.

- Exposing internal implementation detail - `HashMap` as Accounts interface is temporary. It would be reconnected to replace it with some generic trait.

- It might be sensible to provide alternative (not memory) implementation to be secured against: huge accounts number, application / service crashes during processing. Rough idea for toy project is to use <https://crates.io/crates/sled>.

- Following above point implement some kind of persistent check-pointing for transactions, so in case of crash restore application / system would know which transactions were already processed and reflected in state of accounts database.

- More reliable rustdoc descriptions. Documentation on crate / module level.

- Performance tests. Tests on huge data sets.

- Float calculations - write more extensive and complete tests - maybe use some arbitrary calculation crates (e.g. `num` (num_rational)) as reference for tests and use randomly generated numbers.

- Introduce logging thru `log` interface instead of printing to `stderr`.

- Printing output file: use more robust printing interface for accounts (Display, print to Writer etc.) instead of print!.

- `data: AsyncDeserializer<R>` argument of `processing_loop` is not ideal - should be more abstract stream-like data type.

- Memory allocation optimization - limit number of allocations per transaction (possibly to 1 or none).

## Other observations, task to do discovered during work on this project

- It looks like error handling in case when `trim` on CSV reader is not set and there are spaces between data is not good (rec, line, byte info is misleading), and also in case when `has_headers` is not properly set. To investigate and possibly open issue/PR for `csv-core` crate 

- `csv_async` should re-export futures/tokio::io::AsyncRead.

## License

This program is available under MIT license.  
Author: Grzegorz Wierzchowski (gwierzchowski@wp.pl)
