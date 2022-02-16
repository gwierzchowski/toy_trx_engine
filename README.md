# Toy Transaction Engine - simple variant

This is small transaction engine written as exercise in Rust language. It supports 5 types of transactions:

- Deposit
- Withdrawal
- Dispute (transaction questioning)
- Resolution 
- Chargeback 

More comprehensive requirements are described in separate document.

This branch (`simple`) contain the simplest, single-threaded and synchronous version, while `main` branch has some changes towards concurrency. At this moment the `main` branch is not ready yet. The reason for creating _simple_ branch was to stabilize interfaces, and to have simple (less likely having bugs) system as reference for testing and also as reference for future performance tunning.

The `toy_trx_engine` program reads transactions data from CSV file passes as first positional argument and prints account's balances to standard output. It supports several options. Run it with `--help` option figure out all options.
In case of errors in particular transactions, program will print message on `stderr` and continue processing.

## Design decisions

My first idea was creating transaction state machine with transaction passing thru several states. I eventually abandoned this idea in charge of simpler solution, where all business logic is coded in well defined one place - `commit` trait method implementations. E.g. all 'deposit' transaction characteristic code is encoded in `deposit.rs` source file. This simplifies adding new transaction types, and as I believe less error prone. E.g. I did not implemented methods like `deposit` on Account abject to avoid distribution of logic - even at cost of making Account fields public. 

I also tried to keep dependencies under control, taking only those that are really needed or small and safe. Program does not contain any direct code using `unsafe` annotation. Dependencies were checked using audit cargo extension.

I used `rust_decimal` crate to support money calculation. It is quite widely used crate and utilized for money-specialized crate: `rusty-money` so I assume it passed some testing. This is acceptable for toy-tool. But for real production system, I would better either write more extensive and comprehensive test suite to prove library calculates properly (including performance tests) or use 128-bit integers internally to calculate money and only convert for i/o - it may be faster.

For transaction types dispatching I used enum-based dispatching supported by 3rd party crate `enum_dispatch` that limited boiler-plate code. The code would be maybe simpler if I use dynamic dispatching, but this would be at cost of some extra memory allocations and virtual methods calls, so I takes in my opinion fair performance / simplicity compromise.

I assumed that external transaction IDs (`tx`) are unique for particular client (it is weaker assumption then suggested in requirements). Checking global transaction ID uniqueness would bring additional cost - not strictly necessary for system correctness. There is one exception - if transaction is rejected I do not remember its id - i.e. I allow another transaction for the same client with the same ID to be later present. It is kind of compromise support for such scenario would need extra processing and memory, and lack of it may cause debugging harder (looking at input file we are not sure to which transaction reference applies). In my opinion implementing such check in real system would be recommended, but not necessarily for toy-like.

## Status / Remaining work

Provided points are in more-less ordered by priority.

- Make program use of multi-threading and async programming. Currently accounts are pretty much isolated each other, every transaction refers to previous ones done on the same account. This can be utilized by implementing possibly lock-free concurrent algorithms. Initial idea would be to shard accounts pool e.g. by dividing client_id modulo number of workers and dispatch work according to reminder to separate threads or tasks. Approach with creating separate task per each transaction is also possible but more complicated - to ensure processing transactions in order we would have to assign sequential serial number (e.g. record number) and store it into transaction, and then synchronize tasks (for the same customer) to ensure sequential processing. In the future if system would have to be extended for cross accounts transactions (e.g. transfer), some such mechanisms would have to be implemented anyway.

- Following above passing all accounts object to every transaction `commit` method is probably not appropriate and may be treated as security issue. Consider passing only account related to given transaction (or some slice of clients for extensibility - e.g potential transfer transaction).

- Exposing internal implementation detail - `HashMap` as Accounts interface is temporary. It would be reconnected to replace it with some generic trait.

- It might be sensible to provide alternative (not memory) implementation to be secure against: huge accounts number, application / service crashes during processing. Rough idea for toy project is to use <https://crates.io/crates/sled>.

- Following above point implement some king of persistent check-pointing for transactions, so in case of crash restore application / system would know which transactions were already processed and reflected in state of accounts database.

- More reliable rustdoc descriptions. Documentation on crate / module level.

- Performance tests. Tests on huge data sets.

- Float calculations - write more extensive and complete tests - maybe use some arbitrary calculation crates (e.g. `num` (num_rational)) as reference and use randomly generated numbers.

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
